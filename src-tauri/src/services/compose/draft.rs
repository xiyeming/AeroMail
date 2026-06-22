use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::compose::{
    AttachmentDraft, ComposeDraft, ComposeDraftSummary, ReplyContext, ReplyKind,
};
use crate::models::mail::MailDetail;

pub struct DraftService {
    db: Arc<Database>,
    drafts_dir: PathBuf,
}

impl DraftService {
    pub const fn new(db: Arc<Database>, drafts_dir: PathBuf) -> Self {
        Self { db, drafts_dir }
    }

    /// Validates that the given ID is a UUID and returns it.
    fn validate_id<'a>(id: &'a str, name: &str) -> Result<&'a str, AeroError> {
        if Uuid::parse_str(id).is_ok() {
            Ok(id)
        } else {
            Err(AeroError::InvalidAttachment(format!(
                "invalid {name} id: {id}"
            )))
        }
    }

    /// Returns the directory for a draft's attachments.
    pub fn draft_dir(&self, draft_id: &str) -> Result<PathBuf, AeroError> {
        let id = Self::validate_id(draft_id, "draft")?;
        Ok(self.drafts_dir.join(id))
    }

    /// Returns the directory for a specific attachment.
    pub fn attachment_dir(
        &self,
        draft_id: &str,
        attachment_id: &str,
    ) -> Result<PathBuf, AeroError> {
        let draft_dir = self.draft_dir(draft_id)?;
        let id = Self::validate_id(attachment_id, "attachment")?;
        Ok(draft_dir.join(id))
    }

    /// Ensures the draft directory exists.
    fn ensure_draft_dir(&self, draft_id: &str) -> Result<PathBuf, AeroError> {
        let dir = self.draft_dir(draft_id)?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| AeroError::Internal(format!("failed to create draft dir: {e}")))?;
        Ok(dir)
    }

    /// Saves a draft and its attachment metadata locally.
    pub fn save_draft(&self, draft: &mut ComposeDraft) -> Result<(), AeroError> {
        if draft.id.is_empty() {
            draft.id = Uuid::new_v4().to_string();
        }
        draft.saved_at = Utc::now().timestamp();
        self.ensure_draft_dir(&draft.id)?;
        self.db.upsert_draft(draft)?;
        Ok(())
    }

    /// Retrieves a draft by ID.
    pub fn get_draft(&self, draft_id: &str) -> Result<Option<ComposeDraft>, AeroError> {
        self.db.get_draft(draft_id)
    }

    /// Lists draft summaries.
    pub fn list_drafts(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<ComposeDraftSummary>, AeroError> {
        self.db.list_drafts(account_id)
    }

    /// Deletes a draft and its attachment files.
    pub fn delete_draft(&self, draft_id: &str) -> Result<(), AeroError> {
        self.db.delete_draft(draft_id)?;
        let dir = self.draft_dir(draft_id)?;
        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .map_err(|e| AeroError::Internal(format!("failed to remove draft dir: {e}")))?;
        }
        Ok(())
    }

    /// Writes attachment bytes to disk inside the draft directory.
    pub fn write_attachment(
        &self,
        draft_id: &str,
        attachment: &AttachmentDraft,
        data: &[u8],
    ) -> Result<PathBuf, AeroError> {
        self.ensure_draft_dir(draft_id)?;
        let dir = self.attachment_dir(draft_id, &attachment.id)?;
        std::fs::create_dir_all(&dir).map_err(|e| {
            AeroError::InvalidAttachment(format!("failed to create attachment dir: {e}"))
        })?;
        let path = dir.join(&attachment.filename);
        std::fs::write(&path, data).map_err(|e| {
            AeroError::InvalidAttachment(format!("failed to write attachment: {e}"))
        })?;
        Ok(path)
    }

    /// Reads attachment bytes from disk.
    pub fn read_attachment(
        &self,
        draft_id: &str,
        attachment_id: &str,
        filename: &str,
    ) -> Result<Vec<u8>, AeroError> {
        let path = self.attachment_dir(draft_id, attachment_id)?.join(filename);
        std::fs::read(&path)
            .map_err(|e| AeroError::AttachmentNotFound(format!("{}: {e}", path.display())))
    }

    /// Prepares a reply/forward draft from an existing mail.
    pub fn prepare_reply(
        &self,
        account_id: &str,
        original: &MailDetail,
        kind: ReplyKind,
    ) -> Result<ComposeDraft, AeroError> {
        let own_address = self
            .db
            .get_account_email(account_id)?
            .map(|a| a.to_lowercase());

        let mut to = Vec::new();
        let mut cc = Vec::new();

        if let Some(ref from) = original.from_address
            && matches!(kind, ReplyKind::Reply | ReplyKind::ReplyAll)
            && own_address
                .as_ref()
                .is_none_or(|own| extract_email(from).to_lowercase() != *own)
        {
            to.push(from.clone());
        }

        if matches!(kind, ReplyKind::ReplyAll) {
            if let Some(ref to_addrs) = original.to_addresses {
                for addr in parse_address_list(to_addrs) {
                    if own_address
                        .as_ref()
                        .is_none_or(|own| extract_email(&addr).to_lowercase() != *own)
                    {
                        to.push(addr);
                    }
                }
            }
            if let Some(ref cc_addrs) = original.cc_addresses {
                for addr in parse_address_list(cc_addrs) {
                    if own_address
                        .as_ref()
                        .is_none_or(|own| extract_email(&addr).to_lowercase() != *own)
                    {
                        cc.push(addr);
                    }
                }
            }
        }

        let subject_prefix = match kind {
            ReplyKind::Reply | ReplyKind::ReplyAll => "Re: ",
            ReplyKind::Forward => "Fwd: ",
        };
        let subject = match original.subject {
            Some(ref s) if s.starts_with(subject_prefix) => s.clone(),
            Some(ref s) => format!("{subject_prefix}{s}"),
            None => subject_prefix.trim_end().to_string(),
        };

        let quote = format!(
            "\n\nOn {}, {} wrote:\n> {}",
            original
                .date
                .and_then(|d| chrono::DateTime::from_timestamp(d, 0))
                .map(|d| d.to_rfc2822())
                .unwrap_or_default(),
            original.from_address.clone().unwrap_or_default(),
            original
                .body_text
                .clone()
                .unwrap_or_default()
                .replace('\n', "\n> ")
        );
        let body_html = format!(
            "\n\n<blockquote>{}\n</blockquote>",
            html_escape::encode_safe(&original.body_text.clone().unwrap_or_default())
        );

        Ok(ComposeDraft {
            id: String::new(),
            account_id: account_id.to_string(),
            reply_context: Some(ReplyContext {
                original_mail_id: original.id.clone(),
                original_message_id: original.message_id.clone(),
                kind,
            }),
            subject,
            to,
            cc,
            bcc: Vec::new(),
            body_html,
            body_text: quote,
            attachments: Vec::new(),
            saved_at: 0,
            synced_at: None,
            remote_uid: None,
        })
    }
}

/// Parses a JSON address list string, falling back to comma-separated values.
fn parse_address_list(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(json).unwrap_or_else(|_| {
        json.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

/// Extracts the bare email address from a header value like `"Name" <addr@example.com>`.
fn extract_email(addr: &str) -> &str {
    let trimmed = addr.trim();
    if let Some(start) = trimmed.find('<') {
        if let Some(end) = trimmed.find('>') {
            return trimmed[start + 1..end].trim();
        }
    }
    trimmed
}
