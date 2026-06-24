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
}

impl ComposeService {
    pub fn new(
        db: Arc<Database>,
        drafts_dir: PathBuf,
        account_manager: Arc<RwLock<AccountManager>>,
    ) -> Self {
        Self {
            draft_service: DraftService::new(Arc::clone(&db), drafts_dir),
            account_manager,
            db,
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
    pub fn delete_draft(&self, draft_id: &str) -> Result<(), AeroError> {
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
        if let Err(e) = Self::append_to_sent_folder(&account_config, &message_bytes).await {
            warn!(
                "Failed to append sent message to Sent folder for account {}: {}",
                account_id, e
            );
        }

        // Clean up local draft
        self.draft_service.delete_draft(&draft.id)?;

        Ok(())
    }

    /// Appends a sent message to the account's IMAP Sent folder.
    #[instrument(skip_all, fields(account_id = %config.id), err(Debug))]
    async fn append_to_sent_folder(
        config: &AccountConfig,
        message_bytes: &[u8],
    ) -> Result<(), AeroError> {
        debug!("connecting to IMAP to append sent message");
        let mut session = imap_client::connect_imap(config).await?;
        let sent_folder = imap_client::find_sent_folder(&mut session).await?;
        imap_client::append_message(&mut session, &sent_folder, Some("\\Seen"), message_bytes)
            .await?;
        let _ = session.logout().await;
        debug!("sent message appended to folder");
        Ok(())
    }

    #[instrument(skip_all, fields(draft_id = %draft_id), err(Debug))]
    pub async fn sync_draft_to_imap(&self, draft_id: &str) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(draft_id.to_string()))?;

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
