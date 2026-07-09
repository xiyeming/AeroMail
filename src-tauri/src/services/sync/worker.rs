use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::{Notify, RwLock, mpsc};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::mail::{NewMailsEvent, ParsedAttachment, SyncProgress, SyncStatus};
use crate::services::imap_client;

use super::mail_parser;

const LAST_SYNC_TIME_KEY: &str = "app.last_sync_time";
const SYNC_MAIL_DAYS_KEY: &str = "app.sync.mailDays";

#[allow(dead_code)]
fn read_timeout(config: &AccountConfig) -> Duration {
    Duration::from_secs(config.advanced.read_timeout_secs.max(1))
}

async fn with_read_timeout<F, T>(config: &AccountConfig, fut: F) -> Result<T, AeroError>
where
    F: std::future::Future<Output = Result<T, AeroError>>,
{
    timeout(read_timeout(config), fut).await.map_err(|_| {
        AeroError::ImapConnectionFailed(format!(
            "IMAP read timeout after {:?}",
            read_timeout(config)
        ))
    })?
}

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
    pub app_handle: tauri::AppHandle,
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

        let folder_names = Self::list_folders(&mut session, &config).await?;
        info!(
            "Account {}: Found {} folders",
            self.account_id,
            folder_names.len()
        );

        let excluded_folders = self.account_config.read().await.excluded_folders.clone();

        let mut folder_errors: Vec<(String, AeroError)> = Vec::new();
        let mut synced_count = 0u32;

        for folder_name in &folder_names {
            if excluded_folders
                .iter()
                .any(|excluded| excluded.eq_ignore_ascii_case(folder_name))
            {
                debug!(folder = %folder_name, "skipping excluded folder");
                continue;
            }

            // 使用 STATUS 命令快速检查文件夹是否有变化，避免不必要的 SELECT
            if let Ok(should_skip) = self
                .should_skip_folder(&mut session, folder_name, &config)
                .await
            {
                if should_skip {
                    debug!(folder = %folder_name, "folder unchanged, skipping sync");
                    continue;
                }
            }

            match self
                .sync_folder(&mut session, folder_name, &self.attachments_dir, &config)
                .await
            {
                Ok(count) => {
                    info!(
                        "Account {}: Synced {} new mails in {}",
                        self.account_id, count, folder_name
                    );
                    synced_count += count;
                }
                Err(e) => {
                    warn!(
                        "Account {}: Failed to sync folder {}: {}",
                        self.account_id, folder_name, e
                    );
                    folder_errors.push((folder_name.clone(), e));
                }
            }
        }

        let _ = session.logout().await;

        if !folder_errors.is_empty() {
            let message = folder_errors
                .iter()
                .map(|(name, err)| format!("{name}: {err}"))
                .collect::<Vec<_>>()
                .join("; ");
            let _ = self
                .progress_tx
                .send(SyncProgress {
                    account_id: self.account_id.clone(),
                    status: SyncStatus::Error,
                    synced_count,
                    total_count: 0,
                    last_sync_time: None,
                    message: Some(message),
                })
                .await;
            return Err(AeroError::SyncError(format!(
                "failed to sync {}/{} folders: {}",
                folder_errors.len(),
                folder_names.len(),
                folder_errors[0].1
            )));
        }

        Ok(())
    }

    /// Lists available folders on the IMAP server.
    #[instrument(skip_all)]
    async fn list_folders(
        session: &mut imap_client::ImapSession,
        config: &AccountConfig,
    ) -> Result<Vec<String>, AeroError> {
        let mut stream = with_read_timeout(config, async {
            session
                .list(None, Some("*"))
                .await
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))
        })
        .await?;

        let mut folders = Vec::new();
        while let Some(name_res) = stream.next().await {
            let name = name_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
            if name.attributes().iter().any(|attr| matches!(attr, async_imap::types::NameAttribute::NoSelect)) {
                continue;
            }
            folders.push(name.name().to_string());
        }

        debug!(folder_count = folders.len(), "listed IMAP folders");
        Ok(folders)
    }

    /// 使用 IMAP STATUS 命令快速检查文件夹是否需要同步。
    ///
    /// 同时校验 `MESSAGES`（邮件总数）、`UIDVALIDITY` 和 `UIDNEXT`。只有当三者都与本地记录一致，
    /// 且 `UIDVALIDITY` 有效时，才认为文件夹未发生变化并跳过完整同步。
    ///
    /// `UIDNEXT` 用于检测"删除与新增数量相同"的场景：即使 `MESSAGES` 不变，只要 `UIDNEXT`
    /// 增长，就说明有新邮件到来，必须同步。
    #[instrument(skip_all, fields(folder_name = %folder_name))]
    async fn should_skip_folder(
        &self,
        session: &mut imap_client::ImapSession,
        folder_name: &str,
        config: &AccountConfig,
    ) -> Result<bool, AeroError> {
        let status = with_read_timeout(config, async {
            session
                .status(folder_name, "(MESSAGES UIDVALIDITY UIDNEXT)")
                .await
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))
        })
        .await?;

        let remote_exists = status.exists;
        let remote_uid_validity = status.uid_validity.unwrap_or(0);
        let remote_uid_next = status.uid_next.unwrap_or(0);

        let local_folder = self.db.get_folder_by_path(&self.account_id, folder_name)?;
        let Some(local) = local_folder else {
            return Ok(false);
        };

        let local_total = u32::try_from(local.total_count).unwrap_or(0);
        let local_uid_validity = u32::try_from(local.uid_validity.unwrap_or(0)).unwrap_or(0);
        let local_uid_next = u32::try_from(local.uid_next.unwrap_or(0)).unwrap_or(0);

        let unchanged = remote_exists == local_total
            && remote_uid_validity != 0
            && remote_uid_validity == local_uid_validity
            && remote_uid_next == local_uid_next;

        if unchanged {
            debug!(
                folder = %folder_name,
                remote_exists,
                local_total,
                remote_uid_validity,
                local_uid_validity,
                remote_uid_next,
                local_uid_next,
                "folder unchanged, skipping sync"
            );
        }

        Ok(unchanged)
    }

    /// Fetches a UID range from the current folder, parses each message and
    /// upserts it into the local database. Returns the total number of messages
    /// synced so far (`starting_count` + messages fetched in this range) and
    /// the IDs of newly synced mails.
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
    ) -> Result<(u32, Vec<String>), AeroError> {
        let fetch_items = "(UID BODY.PEEK[] FLAGS)";
        debug!(uids = %uid_set, "fetching mails from server");
        let mut fetch_stream = session
            .uid_fetch(uid_set, fetch_items)
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        let mut synced_count = starting_count;
        let mut new_mail_ids: Vec<String> = Vec::new();
        const BATCH_SIZE: usize = 50;
        let mut mail_batch: Vec<crate::models::mail::MailDetail> = Vec::with_capacity(BATCH_SIZE);
        let mut attachment_batch: Vec<(String, Vec<ParsedAttachment>)> =
            Vec::with_capacity(BATCH_SIZE);

        while let Some(fetch_res) = fetch_stream.next().await {
            let fetch = fetch_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

            let uid = fetch.uid.unwrap_or(0);
            if uid == 0 {
                trace!("skipping fetched item without UID");
                continue;
            }

            let raw_message = fetch.body().unwrap_or(&[]);
            if raw_message.is_empty() {
                trace!(uid, "skipping fetched item with empty body");
                continue;
            }

            trace!(uid, bytes = raw_message.len(), "fetched mail from server");
            let parsed = mail_parser::parse_mail(raw_message)?;

            let mail_id =
                if let Some(message_id) = parsed.message_id.as_deref().filter(|s| !s.is_empty()) {
                    self.db
                        .get_mail_id_by_message_id(folder_id, message_id)?
                        .or_else(|| self.db.get_mail_id_by_uid(folder_id, uid).ok().flatten())
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
                } else {
                    self.db
                        .get_mail_id_by_uid(folder_id, uid)?
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
                };

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

            mail_batch.push(mail);
            attachment_batch.push((mail_id.clone(), parsed.attachments));
            new_mail_ids.push(mail_id);
            synced_count += 1;

            if mail_batch.len() >= BATCH_SIZE {
                self.db.upsert_mails_batch(&mail_batch)?;
                for (mid, atts) in &attachment_batch {
                    Self::save_attachments(mid, attachments_dir, &self.db, atts)?;
                }
                mail_batch.clear();
                attachment_batch.clear();
            }

            if synced_count % 10 == 0 {
                let _ = self.progress_tx.try_send(SyncProgress {
                    account_id: self.account_id.clone(),
                    status: SyncStatus::Syncing,
                    synced_count,
                    total_count,
                    last_sync_time: None,
                    message: None,
                });
            }
        }

        if !mail_batch.is_empty() {
            self.db.upsert_mails_batch(&mail_batch)?;
            for (mid, atts) in &attachment_batch {
                Self::save_attachments(mid, attachments_dir, &self.db, atts)?;
            }
        }

        Ok((synced_count, new_mail_ids))
    }

    /// Syncs a single folder.
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    #[instrument(skip_all, fields(account_id = %self.account_id, folder_name = %folder_name))]
    async fn sync_folder(
        &self,
        session: &mut imap_client::ImapSession,
        folder_name: &str,
        attachments_dir: &Path,
        config: &AccountConfig,
    ) -> Result<u32, AeroError> {
        let mailbox = with_read_timeout(config, async {
            session
                .select(folder_name)
                .await
                .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))
        })
        .await?;

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

        let local_folder = self.db.get_folder_by_path(&self.account_id, folder_name)?;
        let display_name = local_folder
            .as_ref()
            .map_or(folder_name, |f| f.name.as_str());
        if display_name == folder_name && local_folder.is_none() {
            // 本地无记录，直接使用路径名
        }

        let folder_id = self.db.upsert_folder(
            &self.account_id,
            display_name,
            folder_name,
            Some(i64::from(remote_uid_validity)),
            Some(i64::from(mailbox.uid_next.unwrap_or(0))),
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

        // Track the highest UID before sync to detect truly new messages
        let prev_max_uid = max_uid;

        let synced_count = if needs_full_sync {
            info!(
                "Account {}: Full sync for {} (UIDVALIDITY changed)",
                self.account_id, folder_name
            );
            let uid_set = build_sync_uid_set(session, sync_days, None, "1:*").await?;
            if uid_set.is_empty() {
                0
            } else {
                let (count, new_ids) = self
                    .fetch_and_upsert_range(
                        session,
                        &folder_id,
                        folder_name,
                        attachments_dir,
                        &uid_set,
                        0,
                        remote_exists,
                    )
                    .await?;
                if !new_ids.is_empty() {
                    self.emit_new_mails_event(&folder_id, &new_ids);
                }
                count
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
                            let (c, _) = self
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
                            count = c;
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

                    // Count mails in INBOX before fetch to detect genuinely new ones
                    let inbox_count_before = if folder_name.eq_ignore_ascii_case("INBOX") {
                        i64::from(self.db.count_mails_in_folder(&folder_id)?)
                    } else {
                        0
                    };

                    let (fetched_count, new_ids) = self
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
                    count = fetched_count;

                    // Emit incremental push for genuinely new mails
                    if !new_ids.is_empty() {
                        self.emit_new_mails_event(&folder_id, &new_ids);
                    }

                    // Notify based on genuinely new DB rows, not fetched count
                    if prev_max_uid > 0 && folder_name.eq_ignore_ascii_case("INBOX") {
                        let inbox_count_after = self.db.count_mails_in_folder(&folder_id)?;
                        let diff = i64::from(inbox_count_after) - inbox_count_before;
                        let new_mails = u32::try_from(diff.max(0)).unwrap_or(u32::MAX);
                        if new_mails > 0 {
                            self.send_new_mail_notification(new_mails, &folder_id);
                        }
                    }
                }
            }

            count
        };

        // Sync flags for existing mails to ensure remote state is source of truth
        self.sync_existing_mail_flags(session, &folder_id).await?;

        // Update unread count after flag sync
        let unread = i64::from(self.db.count_unread_in_folder(&folder_id)?);
        self.db.update_folder_sync(
            &folder_id,
            i64::from(remote_uid_validity),
            unread,
            i64::from(remote_exists),
        )?;

        Ok(synced_count)
    }

    /// Syncs flags (`is_read`, `is_starred`) for existing mails from the IMAP server.
    /// This ensures the remote state is the source of truth.
    async fn sync_existing_mail_flags(
        &self,
        session: &mut imap_client::ImapSession,
        folder_id: &str,
    ) -> Result<(), AeroError> {
        let local_uids = self.db.get_uids_in_folder(folder_id)?;
        if local_uids.is_empty() {
            return Ok(());
        }

        // Build UID set string
        let uid_set = local_uids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");

        // Fetch only flags for existing UIDs
        let mut fetch_stream = session
            .uid_fetch(&uid_set, "(UID FLAGS)")
            .await
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

        while let Some(fetch_res) = fetch_stream.next().await {
            let fetch = fetch_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
            let uid = fetch.uid.unwrap_or(0);
            if uid == 0 {
                continue;
            }

            let is_seen = imap_client::is_seen_flag(fetch.flags());
            let is_flagged = imap_client::is_flagged_flag(fetch.flags());

            self.db.update_mail_flags_by_uid(folder_id, uid, is_seen, is_flagged)?;
        }

        Ok(())
    }

    /// 发送新邮件系统通知，包含发件人和主题信息。
    fn send_new_mail_notification(&self, count: u32, folder_id: &str) {
        let locale = self
            .db
            .get_setting("app.locale")
            .ok()
            .flatten()
            .unwrap_or_else(|| sys_locale().unwrap_or_else(|| "en".to_string()));

        let is_zh = locale.starts_with("zh");

        let title = "AeroMail".to_string();

        // 尝试获取最新未读邮件的发件人和主题，丰富通知内容
        let summaries = match self.db.get_latest_unread_summaries(folder_id, 3) {
            Ok(s) => s,
            Err(e) => {
                warn!(folder_id = %folder_id, error = %e, "failed to load unread summaries for notification");
                Vec::new()
            }
        };

        let body = if summaries.is_empty() {
            // 降级：无详细信息时显示简单计数
            if is_zh {
                format!("{count} 封新邮件")
            } else {
                format!("{count} new messages")
            }
        } else if count == 1 {
            // 单封邮件：显示发件人和主题
            let summary = &summaries[0];
            let from_str = summary.from_name.as_deref().unwrap_or(if is_zh {
                "未知发件人"
            } else {
                "Unknown"
            });
            let subj_str = summary.subject.as_deref().unwrap_or(if is_zh {
                "无主题"
            } else {
                "No subject"
            });
            if is_zh {
                format!("来自 {from_str}: {subj_str}")
            } else {
                format!("From {from_str}: {subj_str}")
            }
        } else {
            // 多封邮件：显示第一封详情 + 其余计数
            let summary = &summaries[0];
            let from_str = summary.from_name.as_deref().unwrap_or(if is_zh {
                "未知发件人"
            } else {
                "Unknown"
            });
            let subj_str = summary.subject.as_deref().unwrap_or(if is_zh {
                "无主题"
            } else {
                "No subject"
            });
            let remaining = count - 1;
            if is_zh {
                format!("来自 {from_str}: {subj_str} 等 {remaining} 封新邮件")
            } else {
                format!("From {from_str}: {subj_str} and {remaining} more")
            }
        };

        if let Err(e) = self
            .app_handle
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show()
        {
            warn!(error = %e, "failed to send new mail notification");
        }
    }

    /// Emits a `sync:new_mails` event to the frontend with the mail headers
    /// of newly synced messages.
    fn emit_new_mails_event(&self, folder_id: &str, mail_ids: &[String]) {
        let mut headers = Vec::with_capacity(mail_ids.len());
        for id in mail_ids {
            if let Ok(Some(header)) = self.db.get_mail_header(id) {
                headers.push(header);
            }
        }
        if headers.is_empty() {
            return;
        }
        let event = NewMailsEvent {
            account_id: self.account_id.clone(),
            folder_id: folder_id.to_string(),
            mails: headers,
        };
        if let Err(e) = self.app_handle.emit("sync:new_mails", &event) {
            warn!(error = %e, "failed to emit sync:new_mails event");
        }
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
        let mut attachment_batch: Vec<(String, ParsedAttachment, std::path::PathBuf)> = Vec::new();

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

            attachment_batch.push((mail_id.to_string(), attachment.clone(), local_path));
        }

        if !attachment_batch.is_empty() {
            db.insert_attachments_batch(&attachment_batch)?;
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

/// Returns the system locale string (e.g. "zh-CN", "en-US").
fn sys_locale() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("LANG")
            .ok()
            .filter(|v| !v.is_empty() && *v != "C" && *v != "POSIX")
            .map(|v| v.split('.').next().unwrap_or(&v).to_string())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("defaults")
            .args(["read", "AppleGlobalDomain", "AppleLanguages"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                s.lines()
                    .next()
                    .map(|l| l.trim().trim_matches('"').to_string())
            })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LANG").ok().filter(|v| !v.is_empty())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}
