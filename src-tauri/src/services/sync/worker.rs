use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use imap::Session;
use native_tls::TlsStream;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig};
use crate::models::mail::{SyncProgress, SyncStatus};

use super::mail_parser;

/// Background sync worker for a single email account.
pub struct SyncWorker {
    pub account_id: String,
    pub account_config: AccountConfig,
    pub db: Arc<Database>,
    pub progress_tx: mpsc::Sender<SyncProgress>,
}

impl SyncWorker {
    /// Runs the sync loop with exponential backoff on errors.
    pub async fn run(&self) {
        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(300); // 5 minutes

        loop {
            match self.sync_once().await {
                Ok(()) => {
                    backoff = Duration::from_secs(1);
                    let _ = self
                        .progress_tx
                        .send(SyncProgress {
                            account_id: self.account_id.clone(),
                            status: SyncStatus::Completed,
                            synced_count: 0,
                            total_count: 0,
                            last_sync_time: Some(chrono::Utc::now().to_rfc3339()),
                        })
                        .await;
                }
                Err(e) => {
                    error!("Sync failed for account {}: {}", self.account_id, e);
                    let _ = self
                        .progress_tx
                        .send(SyncProgress {
                            account_id: self.account_id.clone(),
                            status: SyncStatus::Error(e.to_string()),
                            synced_count: 0,
                            total_count: 0,
                            last_sync_time: None,
                        })
                        .await;
                }
            }

            sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, max_backoff);
        }
    }

    /// Performs a single sync cycle.
    async fn sync_once(&self) -> Result<(), AeroError> {
        let account_id = self.account_id.clone();
        let account_config = self.account_config.clone();
        let db = Arc::clone(&self.db);
        let progress_tx = self.progress_tx.clone();

        // Run synchronous IMAP operations in a blocking thread
        tokio::task::spawn_blocking(move || {
            Self::sync_blocking(&account_id, &account_config, &db, &progress_tx)
        })
        .await
        .map_err(|e| AeroError::SyncError(e.to_string()))?
    }

    /// Performs synchronous IMAP sync (runs in a blocking thread).
    fn sync_blocking(
        account_id: &str,
        config: &AccountConfig,
        db: &Arc<Database>,
        progress_tx: &mpsc::Sender<SyncProgress>,
    ) -> Result<(), AeroError> {
        // Connect to IMAP server
        let mut session = Self::connect_blocking(config)?;

        // Discover and sync folders
        let folder_names = Self::list_folders_blocking(&mut session)?;
        info!(
            "Account {}: Found {} folders",
            account_id,
            folder_names.len()
        );

        for folder_name in &folder_names {
            // Skip excluded folders
            if config
                .excluded_folders
                .iter()
                .any(|excluded| excluded.eq_ignore_ascii_case(folder_name))
            {
                continue;
            }

            match Self::sync_folder_blocking(account_id, &mut session, folder_name, db, progress_tx)
            {
                Ok(count) => {
                    info!(
                        "Account {}: Synced {} new mails in {}",
                        account_id, count, folder_name
                    );
                }
                Err(e) => {
                    warn!(
                        "Account {}: Failed to sync folder {}: {}",
                        account_id, folder_name, e
                    );
                }
            }
        }

        // Logout
        let _ = session.logout();

        Ok(())
    }

    /// Connects to the IMAP server (synchronous).
    fn connect_blocking(
        config: &AccountConfig,
    ) -> Result<Session<TlsStream<TcpStream>>, AeroError> {
        // Build TLS connector
        let mut tls_builder = native_tls::TlsConnector::builder();
        if !config.advanced.verify_certificate {
            tls_builder.danger_accept_invalid_certs(true);
        }
        if let Some(ref cert_path) = config.advanced.ca_cert_path {
            tls_builder.add_root_certificate(
                native_tls::Certificate::from_pem(
                    &std::fs::read(cert_path)
                        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?,
                )
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?,
            );
        }
        let tls = tls_builder
            .build()
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        // Create IMAP client with TLS
        let client = imap::connect(
            format!("{}:{}", config.imap.host, config.imap.port),
            &config.imap.host,
            &tls,
        )
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        // Authenticate
        let session = match &config.auth {
            AuthConfig::Password { password_encrypted } => {
                // TODO: Decrypt the password. For now, treat as plain text.
                let password = String::from_utf8_lossy(password_encrypted);
                client
                    .login(&config.name, &password)
                    .map_err(|e| AeroError::ImapAuthFailed(e.0.to_string()))?
            }
            AuthConfig::OAuth2 { .. } => {
                return Err(AeroError::InvalidConfig(
                    "OAuth2 authentication not yet implemented".into(),
                ));
            }
        };

        Ok(session)
    }

    /// Lists available folders on the IMAP server (synchronous).
    fn list_folders_blocking(
        session: &mut Session<TlsStream<TcpStream>>,
    ) -> Result<Vec<String>, AeroError> {
        let mailboxes = session
            .list(None, Some("*"))
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let folders: Vec<String> = mailboxes.iter().map(|m| m.name().to_string()).collect();

        Ok(folders)
    }

    /// Syncs a single folder (synchronous).
    fn sync_folder_blocking(
        account_id: &str,
        session: &mut Session<TlsStream<TcpStream>>,
        folder_name: &str,
        db: &Arc<Database>,
        progress_tx: &mpsc::Sender<SyncProgress>,
    ) -> Result<u32, AeroError> {
        // SELECT the folder
        let mailbox = session
            .select(folder_name)
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let remote_uid_validity = mailbox.uid_validity.unwrap_or(0);
        let remote_exists = mailbox.exists;

        if remote_exists == 0 {
            return Ok(0);
        }

        // Ensure folder exists in local DB
        let folder_id = db.upsert_folder(
            account_id,
            folder_name,
            folder_name,
            Some(i64::from(remote_uid_validity)),
        )?;

        // Check local UIDVALIDITY
        let local_folder = db.get_folder_by_path(account_id, folder_name)?;
        let needs_full_sync = local_folder.as_ref().is_none_or(|f| {
            f.uid_validity != Some(i64::from(remote_uid_validity))
        });

        let (uids_to_fetch, total_count) = if needs_full_sync {
            // Full sync: fetch all UIDs
            info!(
                "Account {}: Full sync for {} (UIDVALIDITY changed)",
                account_id, folder_name
            );
            (format!("1:{remote_exists}"), remote_exists)
        } else {
            // Incremental sync: fetch only new UIDs
            let max_uid = db.get_max_uid(&folder_id)?.unwrap_or(0);
            if max_uid == 0 {
                (format!("1:{remote_exists}"), remote_exists)
            } else if max_uid >= remote_exists {
                // No new mails
                return Ok(0);
            } else {
                (format!("{}:*", max_uid + 1), remote_exists - max_uid)
            }
        };

        // Emit syncing status
        let _ = progress_tx.blocking_send(SyncProgress {
            account_id: account_id.to_string(),
            status: SyncStatus::Syncing,
            synced_count: 0,
            total_count,
            last_sync_time: None,
        });

        // Fetch mail UIDs and data
        let fetch_items = "UID BODY.PEEK[HEADER] FLAGS";
        let messages = session
            .uid_fetch(&uids_to_fetch, fetch_items)
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let mut synced_count: u32 = 0;

        for fetch_result in &messages {
            let uid = fetch_result.uid.unwrap_or(0);
            if uid == 0 {
                continue;
            }

            // Get the raw header
            let header_bytes = fetch_result.body().unwrap_or(&[]);

            // Parse the header to extract subject, from, date
            let parsed = mail_parser::parse_mail(header_bytes)?;

            // Create mail detail
            let mail_id = uuid::Uuid::new_v4().to_string();
            let mail = crate::models::mail::MailDetail {
                id: mail_id,
                account_id: account_id.to_string(),
                folder_id: folder_id.clone(),
                uid,
                subject: parsed.subject,
                from_name: parsed.from_name,
                from_address: parsed.from_address,
                to_addresses: parsed.to_addresses,
                cc_addresses: parsed.cc_addresses,
                date: parsed.date,
                body_html: None, // Will be fetched on demand
                body_text: None, // Will be fetched on demand
                is_read: false,
                is_starred: false,
                flags: Some(serde_json::to_string(&parsed.flags).unwrap_or_default()),
                message_id: parsed.message_id,
            };

            db.upsert_mail(&mail)?;
            synced_count += 1;

            // Emit progress periodically
            if synced_count % 10 == 0 {
                let _ = progress_tx.blocking_send(SyncProgress {
                    account_id: account_id.to_string(),
                    status: SyncStatus::Syncing,
                    synced_count,
                    total_count,
                    last_sync_time: None,
                });
            }
        }

        // Update folder sync metadata
        let unread = i64::from(db.count_unread(account_id)?);
        db.update_folder_sync(
            &folder_id,
            i64::from(remote_uid_validity),
            unread,
            i64::from(remote_exists),
        )?;

        Ok(synced_count)
    }
}
