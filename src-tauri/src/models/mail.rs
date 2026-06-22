use serde::{Deserialize, Serialize};

/// A slim mail header for list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailHeader {
    pub id: String,
    pub account_id: String,
    pub folder_id: String,
    pub uid: u32,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub date: Option<i64>,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachments: bool,
}

/// Full mail detail for the viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailDetail {
    pub id: String,
    pub account_id: String,
    pub folder_id: String,
    pub uid: u32,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub to_addresses: Option<String>,
    pub cc_addresses: Option<String>,
    pub date: Option<i64>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub flags: Option<String>,
    pub message_id: Option<String>,
}

/// IMAP folder metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub path: String,
    pub unread_count: i64,
    pub total_count: i64,
    pub uid_validity: Option<i64>,
    pub last_sync_at: Option<i64>,
}

/// Sync progress event emitted to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub account_id: String,
    pub status: SyncStatus,
    pub synced_count: u32,
    pub total_count: u32,
    pub last_sync_time: Option<String>,
}

/// Sync status variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error(String),
    Completed,
}

/// Parsed mail from mail-parser, ready for database insertion.
#[derive(Debug, Clone)]
pub struct ParsedMail {
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub to_addresses: Option<String>,
    pub cc_addresses: Option<String>,
    pub date: Option<i64>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub has_attachments: bool,
    pub flags: Vec<String>,
    pub message_id: Option<String>,
}

/// Attachment information for a mail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: String,
    pub mail_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub content_id: Option<String>,
    pub is_inline: bool,
}
