use serde::Serialize;
use thiserror::Error;

use crate::models::error::ErrorPayload;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AeroError {
    #[error("database error: {0}")]
    Database(String),
    #[error("account not found: {0}")]
    AccountNotFound(String),
    #[error("invalid account configuration: {0}")]
    InvalidConfig(String),
    #[error("connection test failed: {0}")]
    ConnectionTestFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("AI provider not found")]
    AiProviderNotFound,
    #[error("AI API error: {0}")]
    AiApiError(String),
    #[error("AI rate limited")]
    AiRateLimited,
    #[error("AI context mail not found")]
    AiContextMailNotFound,
    #[error("translation provider not found")]
    TranslationProviderNotFound,
    #[error("translation API error: {0}")]
    TranslationApiError(String),
    #[error("no text to translate")]
    TranslationNoText,
    #[error("IMAP connection failed: {0}")]
    ImapConnectionFailed(String),
    #[error("IMAP authentication failed: {0}")]
    ImapAuthFailed(String),
    #[error("mail not found: {0}")]
    MailNotFound(String),
    #[error("folder not found: {0}")]
    FolderNotFound(String),
    #[error("SMTP connection failed: {0}")]
    SmtpConnectionFailed(String),
    #[error("SMTP authentication failed: {0}")]
    SmtpAuthFailed(String),
    #[error("invalid recipient: {0}")]
    InvalidRecipient(String),
    #[error("draft not found: {0}")]
    DraftNotFound(String),
    #[error("attachment not found: {0}")]
    AttachmentNotFound(String),
    #[error("invalid attachment: {0}")]
    InvalidAttachment(String),
    #[error("mail builder failed: {0}")]
    MailBuilderFailed(String),
    #[error("IMAP append failed: {0}")]
    ImapAppendFailed(String),
    #[error("sync error: {0}")]
    SyncError(String),
    #[error("search index error: {0}")]
    SearchIndexError(String),
    #[error("search query error: {0}")]
    SearchQueryError(String),
}

impl AeroError {
    /// Converts this error into an [`ErrorPayload`] suitable for the frontend.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            Self::Database(_) => ErrorPayload {
                code: "DATABASE_ERROR".to_string(),
                args: vec![],
            },
            Self::AccountNotFound(id) => ErrorPayload {
                code: "ACCOUNT_NOT_FOUND".to_string(),
                args: vec![id.clone()],
            },
            Self::InvalidConfig(reason) => ErrorPayload {
                code: "INVALID_ACCOUNT_CONFIG".to_string(),
                args: vec![reason.clone()],
            },
            Self::ConnectionTestFailed(reason) => ErrorPayload {
                code: "CONNECTION_TEST_FAILED".to_string(),
                args: vec![reason.clone()],
            },
            Self::Internal(reason) => ErrorPayload {
                code: "INTERNAL_ERROR".to_string(),
                args: vec![reason.clone()],
            },
            Self::AiProviderNotFound => ErrorPayload {
                code: "AI_PROVIDER_NOT_FOUND".to_string(),
                args: vec![],
            },
            Self::AiApiError(msg) => ErrorPayload {
                code: "AI_API_ERROR".to_string(),
                args: vec![msg.clone()],
            },
            Self::AiRateLimited => ErrorPayload {
                code: "AI_RATE_LIMITED".to_string(),
                args: vec![],
            },
            Self::AiContextMailNotFound => ErrorPayload {
                code: "AI_CONTEXT_MAIL_NOT_FOUND".to_string(),
                args: vec![],
            },
            Self::TranslationProviderNotFound => ErrorPayload {
                code: "TRANSLATION_PROVIDER_NOT_FOUND".to_string(),
                args: vec![],
            },
            Self::TranslationApiError(msg) => ErrorPayload {
                code: "TRANSLATION_API_ERROR".to_string(),
                args: vec![msg.clone()],
            },
            Self::TranslationNoText => ErrorPayload {
                code: "TRANSLATION_NO_TEXT".to_string(),
                args: vec![],
            },
            Self::ImapConnectionFailed(msg) => ErrorPayload {
                code: "IMAP_CONNECTION_FAILED".to_string(),
                args: vec![msg.clone()],
            },
            Self::ImapAuthFailed(msg) => ErrorPayload {
                code: "IMAP_AUTH_FAILED".to_string(),
                args: vec![msg.clone()],
            },
            Self::MailNotFound(id) => ErrorPayload {
                code: "MAIL_NOT_FOUND".to_string(),
                args: vec![id.clone()],
            },
            Self::FolderNotFound(id) => ErrorPayload {
                code: "FOLDER_NOT_FOUND".to_string(),
                args: vec![id.clone()],
            },
            Self::SmtpConnectionFailed(msg) => ErrorPayload {
                code: "SMTP_CONNECTION_FAILED".to_string(),
                args: vec![msg.clone()],
            },
            Self::SmtpAuthFailed(msg) => ErrorPayload {
                code: "SMTP_AUTH_FAILED".to_string(),
                args: vec![msg.clone()],
            },
            Self::InvalidRecipient(msg) => ErrorPayload {
                code: "INVALID_RECIPIENT".to_string(),
                args: vec![msg.clone()],
            },
            Self::DraftNotFound(id) => ErrorPayload {
                code: "DRAFT_NOT_FOUND".to_string(),
                args: vec![id.clone()],
            },
            Self::AttachmentNotFound(id) => ErrorPayload {
                code: "ATTACHMENT_NOT_FOUND".to_string(),
                args: vec![id.clone()],
            },
            Self::InvalidAttachment(msg) => ErrorPayload {
                code: "INVALID_ATTACHMENT".to_string(),
                args: vec![msg.clone()],
            },
            Self::MailBuilderFailed(msg) => ErrorPayload {
                code: "MAIL_BUILDER_FAILED".to_string(),
                args: vec![msg.clone()],
            },
            Self::ImapAppendFailed(msg) => ErrorPayload {
                code: "IMAP_APPEND_FAILED".to_string(),
                args: vec![msg.clone()],
            },
            Self::SyncError(msg) => ErrorPayload {
                code: "SYNC_ERROR".to_string(),
                args: vec![msg.clone()],
            },
            Self::SearchIndexError(msg) => ErrorPayload {
                code: "SEARCH_INDEX_ERROR".to_string(),
                args: vec![msg.clone()],
            },
            Self::SearchQueryError(msg) => ErrorPayload {
                code: "SEARCH_QUERY_ERROR".to_string(),
                args: vec![msg.clone()],
            },
        }
    }
}

impl From<rusqlite::Error> for AeroError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AeroError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<AeroError> for ErrorPayload {
    fn from(err: AeroError) -> Self {
        err.to_payload()
    }
}
