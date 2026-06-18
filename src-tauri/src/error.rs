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
}

impl AeroError {
    /// Converts this error into an [`ErrorPayload`] suitable for the frontend.
    #[must_use]
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
