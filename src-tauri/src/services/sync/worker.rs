use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::{RwLock, mpsc};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::mail::{ParsedAttachment, SyncProgress, SyncStatus};
use crate::services::imap_client;

use super::mail_parser;

/// Background sync worker for a single email account.
pub struct SyncWorker {
    pub account_id: String,
    pub account_config: Arc<RwLock<AccountConfig>>,
    pub db: Arc<Database>,
    pub progress_tx: mpsc::Sender<SyncProgress>,
    pub attachments_dir: PathBuf,
}

impl SyncWorker {
    /// Runs the sync loop with exponential backoff on errors.
    #[instrument(skip_all, fields(account_id = %self.account_id))]
    pub async fn run(&self) {
        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(300); // 5 minutes

        loop {
            debug!("starting sync cycle");
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
    #[instrument(skip_all, fields(account_id = %self.account_id))]
    async fn sync_once(&self) -> Result<(), AeroError> {
        let mut config = self.account_config.read().await.clone();
        crate::services::oauth2::ensure_access_token(
            Some(&self.account_id),
            &mut config,
            Some(self.db.as_ref()),
        )
        .await?;
        {
            let mut guard = self.account_config.write().await;
            *guard = config.clone();
        }

        debug!(host = %config.imap.host, port = config.imap.port, "connecting to IMAP server");
        let mut session = imap_client::connect_imap(&config).await?;

        let folder_names = Self::list_folders(&mut session).await?;
        info!(
            "Account {}: Found {} folders",
            self.account_id,
            folder_names.len()
        );

        let excluded_folders = self.account_config.read().await.excluded_folders.clone();

        for folder_name in &folder_names {
            if excluded_folders
                .iter()
                .any(|excluded| excluded.eq_ignore_ascii_case(folder_name))
            {
                debug!(folder = %folder_name, "skipping excluded folder");
                continue;
            }

            match self
                .sync_folder(&mut session, folder_name, &self.attachments_dir)
                .await
            {
                Ok(count) => {
                    info!(
                        "Account {}: Synced {} new mails in {}",
                        self.account_id, count, folder_name
                    );
                }
                Err(e) => {
                    warn!(
                        "Account {}: Failed to sync folder {}: {}",
                        self.account_id, folder_name, e
                    );
                }
            }
        }

        session
            .logout()
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        Ok(())
    }

    /// Lists available folders on the IMAP server.
    #[instrument(skip_all)]
    async fn list_folders(
        session: &mut imap_client::ImapSession,
    ) -> Result<Vec<String>, AeroError> {
        let mut stream = session
            .list(None, Some("*"))
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let mut folders = Vec::new();
        while let Some(name_res) = stream.next().await {
            let name = name_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
            folders.push(name.name().to_string());
        }

        debug!(folder_count = folders.len(), "listed IMAP folders");
        Ok(folders)
    }

    /// Syncs a single folder.
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    #[instrument(skip_all, fields(account_id = %self.account_id, folder_name = %folder_name))]
    async fn sync_folder(
        &self,
        session: &mut imap_client::ImapSession,
        folder_name: &str,
        attachments_dir: &Path,
    ) -> Result<u32, AeroError> {
        let mailbox = session
            .select(folder_name)
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let remote_uid_validity = mailbox.uid_validity.unwrap_or(0);
        let remote_exists = mailbox.exists;
        debug!(
            uid_validity = remote_uid_validity,
            exists = remote_exists,
            "selected folder"
        );

        if remote_exists == 0 {
            return Ok(0);
        }

        let folder_id = self.db.upsert_folder(
            &self.account_id,
            folder_name,
            folder_name,
            Some(i64::from(remote_uid_validity)),
        )?;

        let local_folder = self.db.get_folder_by_path(&self.account_id, folder_name)?;
        let needs_full_sync = local_folder
            .as_ref()
            .is_none_or(|f| f.uid_validity != Some(i64::from(remote_uid_validity)));

        let (uids_to_fetch, total_count) = if needs_full_sync {
            info!(
                "Account {}: Full sync for {} (UIDVALIDITY changed)",
                self.account_id, folder_name
            );
            (format!("1:{remote_exists}"), remote_exists)
        } else {
            let max_uid = self.db.get_max_uid(&folder_id)?.unwrap_or(0);
            if max_uid == 0 {
                (format!("1:{remote_exists}"), remote_exists)
            } else if max_uid >= remote_exists {
                debug!(max_uid, "no new mails to sync");
                return Ok(0);
            } else {
                (format!("{}:*", max_uid + 1), remote_exists - max_uid)
            }
        };

        let _ = self
            .progress_tx
            .send(SyncProgress {
                account_id: self.account_id.clone(),
                status: SyncStatus::Syncing,
                synced_count: 0,
                total_count,
                last_sync_time: None,
            })
            .await;

        let fetch_items = "UID BODY.PEEK[] FLAGS";
        debug!(uids = %uids_to_fetch, "fetching mails from server");
        let mut fetch_stream = session
            .uid_fetch(&uids_to_fetch, fetch_items)
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let mut synced_count: u32 = 0;

        while let Some(fetch_res) = fetch_stream.next().await {
            let fetch = fetch_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

            let uid = fetch.uid.unwrap_or(0);
            if uid == 0 {
                continue;
            }

            let raw_message = fetch.body().unwrap_or(&[]);
            let parsed = mail_parser::parse_mail(raw_message)?;

            let mail_id = self
                .db
                .get_mail_id_by_uid(&folder_id, uid)?
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let is_spam = is_spam_folder(folder_name);
            let is_seen = imap_client::is_seen_flag(fetch.flags());
            let is_flagged = imap_client::is_flagged_flag(fetch.flags());
            let flag_strings = imap_client::collect_flags(fetch.flags());

            let mail = crate::models::mail::MailDetail {
                id: mail_id.clone(),
                account_id: self.account_id.clone(),
                folder_id: folder_id.clone(),
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
            Self::save_attachments(&mail_id, attachments_dir, &self.db, &parsed.attachments)?;
            synced_count += 1;

            if synced_count % 10 == 0 {
                let _ = self
                    .progress_tx
                    .send(SyncProgress {
                        account_id: self.account_id.clone(),
                        status: SyncStatus::Syncing,
                        synced_count,
                        total_count,
                        last_sync_time: None,
                    })
                    .await;
            }
        }

        let unread = i64::from(self.db.count_unread(&self.account_id)?);
        self.db.update_folder_sync(
            &folder_id,
            i64::from(remote_uid_validity),
            unread,
            i64::from(remote_exists),
        )?;

        Ok(synced_count)
    }

    /// Saves parsed attachments to disk and records them in the database.
    #[instrument(skip_all, fields(mail_id = %mail_id, count = attachments.len()))]
    fn save_attachments(
        mail_id: &str,
        attachments_dir: &Path,
        db: &Arc<Database>,
        attachments: &[ParsedAttachment],
    ) -> Result<(), AeroError> {
        if attachments.is_empty() {
            return Ok(());
        }

        let mail_dir = attachments_dir.join(mail_id);
        std::fs::create_dir_all(&mail_dir)
            .map_err(|e| AeroError::Internal(format!("failed to create attachment dir: {e}")))?;

        db.delete_attachments(mail_id)?;

        let mut used_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (idx, attachment) in attachments.iter().enumerate() {
            let base_name = attachment
                .filename
                .clone()
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| format!("attachment-{idx}"));
            let safe_name = sanitize_filename(&base_name);
            let unique_name = unique_filename(&safe_name, &used_names);
            used_names.insert(unique_name.clone());

            let local_path = mail_dir.join(&unique_name);
            std::fs::write(&local_path, &attachment.data).map_err(|e| {
                AeroError::Internal(format!("failed to write attachment {unique_name}: {e}"))
            })?;

            db.insert_attachment(mail_id, attachment, &local_path)?;
        }

        Ok(())
    }
}

fn is_spam_folder(folder_name: &str) -> bool {
    let lower = folder_name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "spam" | "junk" | "[gmail]/spam" | "[gmail]/junk"
    ) || lower.contains("spam")
        || lower.contains("junk")
}

/// Sanitizes a filename so it is safe to store on the local filesystem.
fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim();
    let without_separators: String = trimmed
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect();
    let without_separators = without_separators.trim_start_matches('.');
    if without_separators.is_empty() {
        "unnamed".to_string()
    } else {
        without_separators.to_string()
    }
}

/// Generates a unique filename within a mail by appending a counter if needed.
fn unique_filename(name: &str, used: &std::collections::HashSet<String>) -> String {
    if !used.contains(name) {
        return name.to_string();
    }
    let dot = name.rfind('.').unwrap_or(name.len());
    let stem = &name[..dot];
    let ext = &name[dot..];
    for i in 1..=9999 {
        let candidate = format!("{stem}_{i}{ext}");
        if !used.contains(&candidate) {
            return candidate;
        }
    }
    format!("{}_{}", name, uuid::Uuid::new_v4())
}
