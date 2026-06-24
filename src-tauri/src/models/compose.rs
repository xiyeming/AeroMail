use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComposeDraft {
    pub id: String,
    pub account_id: String,
    pub reply_context: Option<ReplyContext>,
    pub subject: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub body_html: String,
    pub body_text: String,
    pub attachments: Vec<AttachmentDraft>,
    pub saved_at: i64,
    pub synced_at: Option<i64>,
    pub remote_uid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyContext {
    pub original_mail_id: String,
    pub original_message_id: Option<String>,
    pub kind: ReplyKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyKind {
    Reply,
    ReplyAll,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentDraft {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub local_path: Option<String>,
    pub content_id: Option<String>,
    pub is_inline: bool,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComposeDraftSummary {
    pub id: String,
    pub account_id: String,
    pub subject: String,
    pub to: Vec<String>,
    pub saved_at: i64,
    pub has_attachments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMailRequest {
    pub draft_id: String,
}
