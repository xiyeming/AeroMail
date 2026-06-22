#![allow(clippy::missing_errors_doc)]

mod draft;
mod imap_draft_sync;
mod mime_builder;
mod smtp_sender;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::compose::{ComposeDraft, ComposeDraftSummary, ReplyKind, SendMailRequest};
use crate::models::mail::MailDetail;
use crate::services::account_manager::AccountManager;

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

    pub fn save_draft(&self, mut draft: ComposeDraft) -> Result<ComposeDraft, AeroError> {
        self.draft_service.save_draft(&mut draft)?;
        Ok(draft)
    }

    pub fn get_draft(&self, draft_id: &str) -> Result<Option<ComposeDraft>, AeroError> {
        self.draft_service.get_draft(draft_id)
    }

    pub fn list_drafts(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<ComposeDraftSummary>, AeroError> {
        self.draft_service.list_drafts(account_id)
    }

    pub fn delete_draft(&self, draft_id: &str) -> Result<(), AeroError> {
        self.draft_service.delete_draft(draft_id)
    }

    pub fn prepare_reply(
        &self,
        account_id: &str,
        original: &MailDetail,
        kind: ReplyKind,
    ) -> Result<ComposeDraft, AeroError> {
        self.draft_service.prepare_reply(account_id, original, kind)
    }

    pub async fn send_mail(&self, req: SendMailRequest) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(&req.draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(req.draft_id.clone()))?;

        let account_id = draft.account_id.clone();
        let account_config = self.load_account_config(&account_id).await?;

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

        smtp_sender::send_message(&account_config, message_bytes.clone()).await?;

        // Clean up local draft
        self.draft_service.delete_draft(&draft.id)?;

        Ok(())
    }

    pub async fn sync_draft_to_imap(&self, draft_id: &str) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(draft_id.to_string()))?;

        let account_config = self.load_account_config(&draft.account_id).await?;

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

        let new_uid =
            imap_draft_sync::sync_draft_to_imap(&account_config, &draft, &message_bytes, &self.db)?;

        // Update synced_at and remote_uid
        let mut updated = draft;
        updated.synced_at = Some(chrono::Utc::now().timestamp());
        updated.remote_uid = Some(new_uid);
        self.draft_service.save_draft(&mut updated)?;

        Ok(())
    }

    async fn load_account_config(&self, account_id: &str) -> Result<AccountConfig, AeroError> {
        let am = self.account_manager.read().await;
        am.get_account_config(account_id)
    }
}
