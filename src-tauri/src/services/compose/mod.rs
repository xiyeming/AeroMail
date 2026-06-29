#![allow(clippy::missing_errors_doc)]

mod draft;
mod imap_draft_sync;
mod mime_builder;
mod smtp_sender;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::compose::{ComposeDraft, ComposeDraftSummary, ReplyKind, SendMailRequest};
use crate::models::mail::MailDetail;
use crate::services::account_manager::AccountManager;
use crate::services::imap_client;

use self::draft::DraftService;

pub struct ComposeService {
    pub(crate) draft_service: DraftService,
    account_manager: Arc<RwLock<AccountManager>>,
    db: Arc<Database>,
    attachments_dir: PathBuf,
}

impl ComposeService {
    pub fn new(
        db: Arc<Database>,
        drafts_dir: PathBuf,
        attachments_dir: PathBuf,
        account_manager: Arc<RwLock<AccountManager>>,
    ) -> Self {
        Self {
            draft_service: DraftService::new(Arc::clone(&db), drafts_dir),
            account_manager,
            db,
            attachments_dir,
        }
    }

    #[instrument(skip_all, fields(draft_id = %draft.id, account_id = ?draft.account_id), err(Debug))]
    pub fn save_draft(&self, mut draft: ComposeDraft) -> Result<ComposeDraft, AeroError> {
        self.draft_service.save_draft(&mut draft)?;
        Ok(draft)
    }

    #[instrument(skip_all, fields(draft_id = %draft_id), err(Debug))]
    pub fn get_draft(&self, draft_id: &str) -> Result<Option<ComposeDraft>, AeroError> {
        self.draft_service.get_draft(draft_id)
    }

    #[instrument(skip_all, fields(account_id = ?account_id), err(Debug))]
    pub fn list_drafts(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<ComposeDraftSummary>, AeroError> {
        self.draft_service.list_drafts(account_id)
    }

    #[instrument(skip_all, fields(draft_id = %draft_id), err(Debug))]
    pub async fn delete_draft(&self, draft_id: &str) -> Result<(), AeroError> {
        // Fetch the draft first so we can clean up the IMAP copy if it exists
        let draft = self.draft_service.get_draft(draft_id)?;

        if let Some(ref draft) = draft {
            if let Some(uid) = draft.remote_uid {
                if uid > 0 {
                    match self.load_account_config(&draft.account_id).await {
                        Ok(config) => match imap_client::connect_imap(&config).await {
                            Ok(mut session) => {
                                if let Ok(drafts_folder) =
                                    imap_client::find_drafts_folder(&mut session).await
                                {
                                    if let Err(e) = imap_draft_sync::delete_uid(
                                        &mut session,
                                        &drafts_folder,
                                        uid,
                                    )
                                    .await
                                    {
                                        warn!(
                                            "Failed to delete IMAP draft {uid} in \
                                                 {drafts_folder}: {e}"
                                        );
                                    }
                                }
                                let _ = session.logout().await;
                            }
                            Err(e) => {
                                warn!("Failed to connect IMAP for draft cleanup: {e}");
                            }
                        },
                        Err(e) => {
                            warn!("Failed to load account config for draft cleanup: {e}");
                        }
                    }
                }
            }
        }

        self.draft_service.delete_draft(draft_id)
    }

    #[instrument(skip_all, fields(account_id = %account_id, original_mail_id = %original.id), err(Debug))]
    pub fn prepare_reply(
        &self,
        account_id: &str,
        original: &MailDetail,
        kind: ReplyKind,
    ) -> Result<ComposeDraft, AeroError> {
        self.draft_service.prepare_reply(account_id, original, kind)
    }

    #[instrument(skip_all, fields(draft_id = %req.draft_id), err(Debug))]
    pub async fn send_mail(&self, req: SendMailRequest) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(&req.draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(req.draft_id.clone()))?;

        let account_id = draft.account_id.clone();
        let account_config = self.load_account_config(&account_id).await?;

        debug!("building MIME message");
        // Collect attachment bytes
        let mut attachment_bytes = Vec::new();
        for att in &draft.attachments {
            let bytes = self
                .draft_service
                .read_attachment(&draft.id, &att.id, &att.filename)?;
            attachment_bytes.push((att.id.clone(), bytes));
        }

        let from_address = account_config
            .email
            .as_deref()
            .unwrap_or(&account_config.name);
        let message_bytes = mime_builder::build_message(
            &draft,
            from_address,
            &account_config.name,
            &attachment_bytes,
        )?;

        debug!("sending message via SMTP");
        smtp_sender::send_message(&account_config, message_bytes.clone()).await?;

        // Append a copy to the IMAP Sent folder so the message appears there.
        let sent_folder = match Self::append_to_sent_folder(&account_config, &message_bytes).await {
            Ok(path) => path,
            Err(e) => {
                warn!(
                    "Failed to append sent message to IMAP for account {}: {}",
                    account_id, e
                );
                "Sent".to_string()
            }
        };

        // Always keep a local copy so the Sent virtual folder shows the mail
        // immediately, even if the IMAP append failed or before the next sync.
        if let Err(e) = self.save_local_sent_copy(&account_config, &sent_folder, &message_bytes) {
            warn!(
                "Failed to save local sent copy for account {}: {}",
                account_id, e
            );
        }

        // Clean up local draft
        self.draft_service.delete_draft(&draft.id)?;

        Ok(())
    }

    /// Appends a sent message to the account's IMAP Sent folder and returns the
    /// folder path used.
    #[instrument(skip_all, fields(account_id = %config.id), err(Debug))]
    async fn append_to_sent_folder(
        config: &AccountConfig,
        message_bytes: &[u8],
    ) -> Result<String, AeroError> {
        debug!("connecting to IMAP to append sent message");
        let mut session = imap_client::connect_imap(config).await?;
        let sent_folder = imap_client::find_sent_folder(&mut session).await?;
        imap_client::append_message(&mut session, &sent_folder, Some("\\Seen"), message_bytes)
            .await?;
        let _ = session.logout().await;
        debug!("sent message appended to folder");
        Ok(sent_folder)
    }

    /// Saves a copy of a sent message into the local database under the Sent folder.
    #[instrument(skip_all, fields(account_id = %config.id), err(Debug))]
    fn save_local_sent_copy(
        &self,
        config: &AccountConfig,
        folder_path: &str,
        message_bytes: &[u8],
    ) -> Result<(), AeroError> {
        let parsed = crate::services::sync::mail_parser::parse_mail(message_bytes)?;
        let folder_id = self
            .db
            .upsert_folder(&config.id, folder_path, folder_path, None)?;

        let now = chrono::Utc::now().timestamp();
        let mail_id = parsed
            .message_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let mail = crate::models::mail::MailDetail {
            id: mail_id.clone(),
            account_id: config.id.clone(),
            folder_id,
            uid: 0,
            subject: parsed.subject,
            from_name: parsed.from_name,
            from_address: parsed.from_address,
            to_addresses: parsed.to_addresses,
            cc_addresses: parsed.cc_addresses,
            date: parsed.date.or(Some(now)),
            body_html: parsed.body_html,
            body_text: parsed.body_text,
            is_read: true,
            is_starred: false,
            is_archived: false,
            is_spam: false,
            flags: Some(serde_json::to_string(&vec![r"\Seen"]).unwrap_or_default()),
            message_id: parsed.message_id,
        };

        self.db.upsert_mail(&mail)?;
        crate::services::sync::worker::SyncWorker::save_attachments(
            &mail_id,
            &self.attachments_dir,
            &self.db,
            &parsed.attachments,
        )?;
        Ok(())
    }

    #[instrument(skip_all, fields(draft_id = %draft_id), err(Debug))]
    pub async fn sync_draft_to_imap(&self, draft_id: &str) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(draft_id.to_string()))?;

        // 没有收件人时 lettre 无法构建 envelope；跳过同步，等待用户填写完整后再同步
        if draft.to.is_empty() && draft.cc.is_empty() && draft.bcc.is_empty() {
            debug!("draft has no recipients, skipping IMAP sync");
            return Ok(());
        }

        let account_config = self.load_account_config(&draft.account_id).await?;

        debug!("building MIME message for IMAP draft sync");
        let mut attachment_bytes = Vec::new();
        for att in &draft.attachments {
            let bytes = self
                .draft_service
                .read_attachment(&draft.id, &att.id, &att.filename)?;
            attachment_bytes.push((att.id.clone(), bytes));
        }

        let from_address = account_config
            .email
            .as_deref()
            .unwrap_or(&account_config.name);
        let message_bytes = mime_builder::build_message(
            &draft,
            from_address,
            &account_config.name,
            &attachment_bytes,
        )?;

        debug!("uploading draft to IMAP");
        let new_uid =
            imap_draft_sync::sync_draft_to_imap(&account_config, &draft, &message_bytes, &self.db)
                .await?;

        // Update synced_at and remote_uid
        let mut updated = draft;
        updated.synced_at = Some(chrono::Utc::now().timestamp());
        updated.remote_uid = Some(new_uid);
        self.draft_service.save_draft(&mut updated)?;

        Ok(())
    }

    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    async fn load_account_config(&self, account_id: &str) -> Result<AccountConfig, AeroError> {
        let am = self.account_manager.read().await;
        am.get_account_config_with_refresh(account_id).await
    }
}
