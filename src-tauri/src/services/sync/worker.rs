use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::{Notify, RwLock, mpsc};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::mail::{ParsedAttachment, SyncProgress, SyncStatus};
use crate::services::imap_client;

use super::mail_parser;

const LAST_SYNC_TIME_KEY: &str = "app.last_sync_time";
const SYNC_MAIL_DAYS_KEY: &str = "app.sync.mailDays";

/// Maps the persisted `app.sync.mailDays` setting to a number of days.
/// `None` means "all" (no date cutoff). Defaults to 7 days.
pub fn load_sync_mail_days(db: &Database) -> Result<Option<u32>, AeroError> {
    let value = db
        .get_setting(SYNC_MAIL_DAYS_KEY)?
        .unwrap_or_else(|| "7".to_string());
    Ok(match value.as_str() {
        "30" => Some(30),
        "90" => Some(90),
        "180" => Some(180),
        "all" => None,
        _ => Some(7),
    })
}

/// Returns a compact comma-separated UID set string (e.g. `1:5,7,10:12`).
fn compact_uid_set(uids: &[u32]) -> String {
    if uids.is_empty() {
        return String::new();
    }
    let mut uids = uids.to_vec();
    uids.sort_unstable();
    let mut parts = Vec::new();
    let mut start = uids[0];
    let mut prev = start;
    for uid in uids.iter().copied().skip(1) {
        if uid != prev + 1 {
            parts.push(if start == prev {
                start.to_string()
            } else {
                format!("{start}:{prev}")
            });
            start = uid;
        }
        prev = uid;
    }
    parts.push(if start == prev {
        start.to_string()
    } else {
        format!("{start}:{prev}")
    });
    parts.join(",")
}

/// Searches the selected folder for UIDs matching the date cutoff and optional
/// UID range.
async fn search_uids_since(
    session: &mut imap_client::ImapSession,
    days: u32,
    uid_range: Option<&str>,
) -> Result<Vec<u32>, AeroError> {
    let since_date = (chrono::Utc::now() - chrono::Duration::days(i64::from(days)))
        .format("%d-%b-%Y")
        .to_string();
    let criteria = uid_range.map_or_else(
        || format!("SINCE {since_date}"),
        |range| format!("UID {range} SINCE {since_date}"),
    );
    let uids = session
        .uid_search(&criteria)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let mut uids: Vec<u32> = uids.into_iter().collect();
    uids.sort_unstable();
    Ok(uids)
}

/// Builds the UID set to fetch for a sync operation, applying the configured
/// date cutoff when one is set.
pub async fn build_sync_uid_set(
    session: &mut imap_client::ImapSession,
    sync_days: Option<u32>,
    range: Option<&str>,
    fallback: &str,
) -> Result<String, AeroError> {
    if let Some(days) = sync_days {
        let uids = search_uids_since(session, days, range).await?;
        Ok(compact_uid_set(&uids))
    } else {
        Ok(range.map_or_else(|| fallback.to_string(), std::string::ToString::to_string))
    }
}

/// Background sync worker for a single email account.
pub struct SyncWorker {
    pub account_id: String,
    pub account_config: Arc<RwLock<AccountConfig>>,
    pub db: Arc<Database>,
    pub progress_tx: mpsc::Sender<SyncProgress>,
    pub attachments_dir: PathBuf,
    pub wake_notify: Arc<Notify>,
}

impl SyncWorker {
    /// Runs the sync loop using the account's configured interval between
    /// successful syncs, with exponential backoff on errors.
    #[instrument(skip_all, fields(account_id = %self.account_id))]
    pub async fn run(&self) {
        let mut error_backoff = Duration::from_secs(1);
        let max_error_backoff = Duration::from_secs(300); // 5 minutes

        // Notify the UI immediately so the refresh button is disabled while
        // the initial connection is being established.
        let _ = self
            .progress_tx
            .send(SyncProgress {
                account_id: self.account_id.clone(),
                status: SyncStatus::Syncing,
                synced_count: 0,
                total_count: 0,
                last_sync_time: None,
                message: None,
            })
            .await;

        loop {
            debug!("starting sync cycle");
            let sync_interval = self.account_config.read().await.sync_interval_secs;
            // Avoid a zero/very short interval that would hammer the server.
            let sync_interval = std::cmp::max(sync_interval, 15);
            let mut last_sync_ok = true;

            match self.sync_once().await {
                Ok(()) => {
                    error_backoff = Duration::from_secs(1);
                    let last_sync_time = Some(chrono::Utc::now().to_rfc3339());
                    if let Some(ref time) = last_sync_time {
                        if let Err(e) = self.db.set_setting(LAST_SYNC_TIME_KEY, time) {
                            warn!("Failed to persist last sync time: {}", e);
                        }
                    }
                    let _ = self
                        .progress_tx
                        .send(SyncProgress {
                            account_id: self.account_id.clone(),
                            status: SyncStatus::Completed,
                            synced_count: 0,
                            total_count: 0,
                            last_sync_time,
                            message: None,
                        })
                        .await;
                }
                Err(e) => {
                    error!("Sync failed for account {}: {}", self.account_id, e);
                    last_sync_ok = false;
                    let _ = self
                        .progress_tx
                        .send(SyncProgress {
                            account_id: self.account_id.clone(),
                            status: SyncStatus::Error,
                            synced_count: 0,
                            total_count: 0,
                            last_sync_time: None,
                            message: Some(e.to_string()),
                        })
                        .await;
                }
            }

            let wait = if last_sync_ok {
                Duration::from_secs(sync_interval)
            } else {
                error_backoff
            };

            tokio::select! {
                biased;
                () = self.wake_notify.notified() => {
                    debug!("manual refresh woke worker; syncing immediately");
                    error_backoff = Duration::from_secs(1);
                }
                () = sleep(wait) => {}
            }

            if !last_sync_ok {
                error_backoff = std::cmp::min(error_backoff * 2, max_error_backoff);
            }
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

    /// Fetches a UID range from the current folder, parses each message and
    /// upserts it into the local database. Returns the total number of messages
    /// synced so far (`starting_count` + messages fetched in this range).
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all, fields(account_id = %self.account_id, folder_name = %folder_name, uid_set = %uid_set))]
    async fn fetch_and_upsert_range(
        &self,
        session: &mut imap_client::ImapSession,
        folder_id: &str,
        folder_name: &str,
        attachments_dir: &Path,
        uid_set: &str,
        starting_count: u32,
        total_count: u32,
    ) -> Result<u32, AeroError> {
        let fetch_items = "(UID BODY.PEEK[] FLAGS)";
        debug!(uids = %uid_set, "fetching mails from server");
        let mut fetch_stream = session
            .uid_fetch(uid_set, fetch_items)
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let mut synced_count = starting_count;

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

            debug!(uid, bytes = raw_message.len(), "fetched mail from server");
            let parsed = mail_parser::parse_mail(raw_message)?;

            let mail_id = self
                .db
                .get_mail_id_by_uid(folder_id, uid)?
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let is_spam = is_spam_folder(folder_name);
            let is_seen = imap_client::is_seen_flag(fetch.flags());
            let is_flagged = imap_client::is_flagged_flag(fetch.flags());
            let flag_strings = imap_client::collect_flags(fetch.flags());

            let mail = crate::models::mail::MailDetail {
                id: mail_id.clone(),
                account_id: self.account_id.clone(),
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
                        message: None,
                    })
                    .await;
            }
        }

        Ok(synced_count)
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

        let highest_uid = mailbox.uid_next.map(|n| n.saturating_sub(1));

        let max_uid = self.db.get_max_uid(&folder_id)?.unwrap_or(0);
        let min_uid = self.db.get_min_uid(&folder_id)?;
        let local_count = self.db.count_mails_in_folder(&folder_id)?;

        let sync_days = load_sync_mail_days(&self.db)?;

        let synced_count = if needs_full_sync {
            info!(
                "Account {}: Full sync for {} (UIDVALIDITY changed)",
                self.account_id, folder_name
            );
            let uid_set = build_sync_uid_set(session, sync_days, None, "1:*").await?;
            if uid_set.is_empty() {
                0
            } else {
                self.fetch_and_upsert_range(
                    session,
                    &folder_id,
                    folder_name,
                    attachments_dir,
                    &uid_set,
                    0,
                    remote_exists,
                )
                .await?
            }
        } else {
            let mut count = 0;

            // Backfill older messages before fetching new ones. If we fetch new
            // messages first, local_count can rise to (or above) remote_exists
            // and this backfill branch would be skipped even when older UIDs
            // are still missing.
            if local_count < remote_exists {
                if let Some(min_uid) = min_uid {
                    if min_uid > 1 {
                        let missing = remote_exists - local_count;
                        let range = format!("1:{}", min_uid - 1);
                        debug!(
                            local_count,
                            remote_exists, min_uid, "backfilling older messages"
                        );
                        let uid_set =
                            build_sync_uid_set(session, sync_days, Some(&range), &range).await?;
                        if !uid_set.is_empty() {
                            count = self
                                .fetch_and_upsert_range(
                                    session,
                                    &folder_id,
                                    folder_name,
                                    attachments_dir,
                                    &uid_set,
                                    count,
                                    count + missing,
                                )
                                .await?;
                        }
                    }
                }
            }

            // Fetch messages that arrived after our highest known UID.
            let new_range = match highest_uid {
                Some(highest) if max_uid >= highest => {
                    debug!(max_uid, highest, "no new mails to sync");
                    None
                }
                Some(highest) => {
                    let count_diff = highest - max_uid;
                    Some((format!("{}:{}", max_uid + 1, highest), count_diff))
                }
                None if max_uid > 0 => {
                    // UIDNEXT not provided; ask for everything after our max
                    // and let the server return only the new messages.
                    Some((format!("{}:*", max_uid + 1), 0))
                }
                None => Some(("1:*".to_string(), remote_exists)),
            };

            if let Some((range, total)) = new_range {
                let uid_set = build_sync_uid_set(session, sync_days, Some(&range), &range).await?;
                if !uid_set.is_empty() {
                    let _ = self
                        .progress_tx
                        .send(SyncProgress {
                            account_id: self.account_id.clone(),
                            status: SyncStatus::Syncing,
                            synced_count: count,
                            total_count: count + total,
                            last_sync_time: None,
                            message: None,
                        })
                        .await;

                    count = self
                        .fetch_and_upsert_range(
                            session,
                            &folder_id,
                            folder_name,
                            attachments_dir,
                            &uid_set,
                            count,
                            count + total,
                        )
                        .await?;
                }
            }

            count
        };

        let unread = i64::from(self.db.count_unread_in_folder(&folder_id)?);
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
    pub(crate) fn save_attachments(
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

pub fn is_spam_folder(folder_name: &str) -> bool {
    let lower = folder_name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "spam" | "junk" | "[gmail]/spam" | "[gmail]/junk"
    ) || lower.contains("spam")
        || lower.contains("junk")
}

/// Sanitizes a filename so it is safe to store on the local filesystem.
pub fn sanitize_filename(name: &str) -> String {
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
pub fn unique_filename(name: &str, used: &std::collections::HashSet<String>) -> String {
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
