mod mail_parser;
mod worker;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use tauri::Emitter;
use tokio::sync::{Notify, RwLock, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::mail::{SyncProgress, SyncStatus};
use crate::services::imap_client;
use crate::services::oauth2;

use self::worker::{SyncWorker, build_sync_uid_set, is_spam_folder, load_sync_mail_days};

type WorkerHandle = (JoinHandle<()>, Arc<Notify>);

/// Manages background sync tasks for all email accounts.
pub struct SyncService {
    db: Arc<Database>,
    /// Directory where attachment files are stored.
    attachments_dir: PathBuf,
    /// Map of `account_id` -> worker handle
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
}

impl SyncService {
    /// Creates a new `SyncService`.
    #[must_use]
    pub fn new(db: Arc<Database>, attachments_dir: PathBuf) -> Self {
        Self {
            db,
            attachments_dir,
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Starts background sync for a specific account.
    ///
    /// # Errors
    ///
    /// Returns an error if the account is not found or sync cannot be started.
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub async fn start_sync(
        &self,
        account_id: &str,
        app_handle: tauri::AppHandle,
    ) -> Result<(), AeroError> {
        // If the worker is already running, wake it up for an immediate sync
        // instead of silently doing nothing.
        {
            let workers = self.workers.read().await;
            if let Some((_, notify)) = workers.get(account_id) {
                info!(
                    "Sync already running for account {}; waking worker",
                    account_id
                );
                notify.notify_one();
                return Ok(());
            }
        }

        // Load account config from DB
        let account_config = match self.load_account_config(account_id) {
            Ok(config) => config,
            Err(e) => {
                let progress = SyncProgress {
                    account_id: account_id.to_string(),
                    status: SyncStatus::Error,
                    synced_count: 0,
                    total_count: 0,
                    last_sync_time: None,
                    message: Some(e.to_string()),
                };
                let _ = app_handle.emit("sync:progress", &progress);
                return Err(e);
            }
        };

        // Create progress channel
        let (progress_tx, mut progress_rx) = mpsc::channel::<SyncProgress>(100);

        // Spawn progress forwarder task
        let handle_clone = app_handle.clone();
        let _account_id_clone = account_id.to_string();
        tokio::spawn(async move {
            while let Some(progress) = progress_rx.recv().await {
                let _ = handle_clone.emit("sync:progress", &progress);
            }
        });

        // Create and start worker
        let wake_notify = Arc::new(Notify::new());
        let worker = SyncWorker {
            account_id: account_id.to_string(),
            account_config: Arc::new(RwLock::new(account_config)),
            db: Arc::clone(&self.db),
            progress_tx,
            attachments_dir: self.attachments_dir.clone(),
            wake_notify: Arc::clone(&wake_notify),
        };

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        self.workers
            .write()
            .await
            .insert(account_id.to_string(), (handle, wake_notify));

        info!("Started sync for account {}", account_id);
        Ok(())
    }

    /// Stops background sync for a specific account.
    ///
    /// # Errors
    ///
    /// Returns an error if the stop operation fails.
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub async fn stop_sync(&self, account_id: &str) -> Result<(), AeroError> {
        let mut workers = self.workers.write().await;
        if let Some((handle, _)) = workers.remove(account_id) {
            handle.abort();
            info!("Stopped sync for account {}", account_id);
        }
        drop(workers);
        Ok(())
    }

    /// Fetches older messages from the IMAP server for a single folder when the
    /// local mail list runs out of pages. Returns the number of newly synced
    /// messages.
    ///
    /// # Errors
    ///
    /// Returns an error if the folder or account cannot be found, the IMAP
    /// connection fails, or messages cannot be persisted.
    #[allow(clippy::too_many_lines)]
    #[instrument(skip_all, fields(folder_id = %folder_id, limit), err(Debug))]
    pub async fn fetch_older_mails(&self, folder_id: &str, limit: u32) -> Result<u32, AeroError> {
        let folder = self
            .db
            .get_folder_by_id(folder_id)?
            .ok_or_else(|| AeroError::MailNotFound(folder_id.to_string()))?;

        let mut config = self.load_account_config(&folder.account_id)?;
        oauth2::ensure_access_token(
            Some(&folder.account_id),
            &mut config,
            Some(self.db.as_ref()),
        )
        .await?;

        debug!(host = %config.imap.host, port = config.imap.port, "connecting to IMAP server for older mail fetch");
        let mut session = imap_client::connect_imap(&config).await?;

        let mailbox = session
            .select(&folder.path)
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
        let remote_uid_validity = mailbox.uid_validity.unwrap_or(0);
        let remote_exists = mailbox.exists;

        debug!(
            uid_validity = remote_uid_validity,
            exists = remote_exists,
            "selected folder for older mail fetch"
        );

        if remote_exists == 0 {
            session
                .logout()
                .await
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
            return Ok(0);
        }

        // Make sure UIDVALIDITY is up to date locally.
        self.db.upsert_folder(
            &folder.account_id,
            &folder.path,
            &folder.path,
            Some(i64::from(remote_uid_validity)),
        )?;

        let local_count = self.db.count_mails_in_folder(folder_id)?;
        let min_uid = self.db.get_min_uid(folder_id)?.unwrap_or(0);

        if local_count >= remote_exists || min_uid <= 1 {
            session
                .logout()
                .await
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
            return Ok(0);
        }

        let missing = remote_exists - local_count;
        let fetch_count = std::cmp::min(limit, missing);
        let start_uid = std::cmp::max(1, min_uid.saturating_sub(fetch_count));
        let range = format!("{}:{}", start_uid, min_uid - 1);

        debug!(range = %range, local_count, remote_exists, "fetching older messages from server");

        let sync_days = load_sync_mail_days(&self.db)?;
        let uid_set = build_sync_uid_set(&mut session, sync_days, Some(&range), &range).await?;
        if uid_set.is_empty() {
            session
                .logout()
                .await
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
            return Ok(0);
        }

        let mut fetch_stream = session
            .uid_fetch(&uid_set, "(UID BODY.PEEK[] FLAGS)")
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let attachments_dir = self.attachments_dir.clone();
        let mut synced = 0u32;

        while let Some(fetch_res) = fetch_stream.next().await {
            let fetch = fetch_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

            let uid = fetch.uid.unwrap_or(0);
            if uid == 0 {
                debug!("skipping fetched item without UID");
                continue;
            }

            let raw_message = fetch.body().unwrap_or(&[]);
            if raw_message.is_empty() {
                debug!(uid, "skipping fetched item with empty body");
                continue;
            }

            debug!(
                uid,
                bytes = raw_message.len(),
                "fetched older mail from server"
            );
            let parsed = mail_parser::parse_mail(raw_message)?;

            let mail_id = self
                .db
                .get_mail_id_by_uid(folder_id, uid)?
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let is_spam = is_spam_folder(&folder.path);
            let is_seen = imap_client::is_seen_flag(fetch.flags());
            let is_flagged = imap_client::is_flagged_flag(fetch.flags());
            let flag_strings = imap_client::collect_flags(fetch.flags());

            let mail = crate::models::mail::MailDetail {
                id: mail_id.clone(),
                account_id: folder.account_id.clone(),
                folder_id: folder_id.to_string(),
                uid,
                subject: parsed.subject,
                from_name: parsed.from_name,
                from_address: parsed.from_address,
                to_addresses: parsed.to_addresses,
                cc_addresses: parsed.cc_addresses,
                date: parsed.date,
                body_html: parsed.body_html,
                body_text: parsed.body_text,
                is_read: is_seen,
                is_starred: is_flagged,
                is_archived: false,
                is_spam,
                flags: Some(serde_json::to_string(&flag_strings).unwrap_or_default()),
                message_id: parsed.message_id,
            };

            self.db.upsert_mail(&mail)?;
            SyncWorker::save_attachments(
                &mail_id,
                &attachments_dir,
                &self.db,
                &parsed.attachments,
            )?;
            synced += 1;
        }

        drop(fetch_stream);

        let unread = i64::from(self.db.count_unread_in_folder(folder_id)?);
        self.db.update_folder_sync(
            folder_id,
            i64::from(remote_uid_validity),
            unread,
            i64::from(remote_exists),
        )?;

        session
            .logout()
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        info!("Fetched {} older messages for folder {}", synced, folder_id);
        Ok(synced)
    }

    /// Starts sync for all configured accounts.
    ///
    /// # Errors
    ///
    /// Returns an error if any account's sync cannot be started.
    #[allow(clippy::significant_drop_tightening)]
    #[instrument(skip_all, err(Debug))]
    pub async fn start_all(&self, app_handle: tauri::AppHandle) -> Result<(), AeroError> {
        let accounts = {
            let conn = self.db.connection()?;
            let mut stmt = conn.prepare("SELECT id FROM accounts")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AeroError::Database(e.to_string()))?
        };

        info!(
            account_count = accounts.len(),
            "starting sync for all accounts"
        );

        for account_id in accounts {
            if let Err(e) = self.start_sync(&account_id, app_handle.clone()).await {
                tracing::warn!("Failed to start sync for account {}: {}", account_id, e);
            }
        }

        Ok(())
    }

    /// Stops all running sync tasks.
    #[instrument(skip_all)]
    pub async fn stop_all(&self) {
        let mut workers = self.workers.write().await;
        for (_, (handle, _)) in workers.drain() {
            handle.abort();
        }
        drop(workers);
        info!("Stopped all sync tasks");
    }

    /// Loads an account configuration from the database.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    fn load_account_config(&self, account_id: &str) -> Result<AccountConfig, AeroError> {
        debug!("loading account config for sync worker");
        let conn = self.db.connection()?;
        conn.query_row(
            "SELECT id, name, email, provider, imap_host, imap_port, smtp_host, smtp_port,
             tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
             verify_certificate, connect_timeout, read_timeout, keepalive,
             sync_interval, excluded_folders
             FROM accounts WHERE id = ?1",
            [account_id],
            |row| {
                let name: String = row.get(1)?;
                let email: Option<String> = row.get(2)?;
                let provider_json: String = row.get(3)?;
                let provider = serde_json::from_str(&provider_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                let tls_mode_json: String = row.get(8)?;
                let tls_mode = serde_json::from_str(&tls_mode_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                let auth_type: String = row.get(9)?;
                let auth_credentials: Option<Vec<u8>> = row.get(10)?;

                let auth = match auth_type.as_str() {
                    "Password" => {
                        let encrypted = auth_credentials.unwrap_or_default();
                        let plaintext = crate::services::crypto::decrypt_password(&encrypted)
                            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                        crate::models::account::AuthConfig::Password {
                            password_encrypted: plaintext,
                        }
                    }
                    "OAuth2" => {
                        // Parse OAuth2 credentials from JSON
                        let creds_bytes = auth_credentials.unwrap_or_default();
                        let creds_json = String::from_utf8_lossy(&creds_bytes);
                        serde_json::from_str(&creds_json)
                            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    }
                    _ => {
                        return Err(rusqlite::Error::InvalidParameterName(format!(
                            "Unknown auth type: {auth_type}"
                        )));
                    }
                };

                let excluded_folders_json: String = row.get(17)?;
                let excluded_folders: Vec<String> =
                    serde_json::from_str(&excluded_folders_json).unwrap_or_default();

                Ok(AccountConfig {
                    id: row.get(0)?,
                    name,
                    email,
                    provider,
                    imap: crate::models::account::ServerConfig {
                        host: row.get(4)?,
                        port: row.get::<_, i64>(5)? as u16,
                        tls_mode,
                    },
                    smtp: crate::models::account::ServerConfig {
                        host: row.get(6)?,
                        port: row.get::<_, i64>(7)? as u16,
                        tls_mode: crate::models::account::TlsMode::Required, // Default
                    },
                    auth,
                    advanced: crate::models::account::AdvancedConfig {
                        ca_cert_path: row.get(11)?,
                        verify_certificate: row.get::<_, i64>(12)? != 0,
                        connect_timeout_secs: row.get::<_, i64>(13)? as u64,
                        read_timeout_secs: row.get::<_, i64>(14)? as u64,
                        keepalive: row.get::<_, i64>(15)? != 0,
                    },
                    sync_interval_secs: row.get::<_, i64>(16)? as u64,
                    excluded_folders,
                })
            },
        )
        .map_err(|e| AeroError::Database(e.to_string()))
    }
}
