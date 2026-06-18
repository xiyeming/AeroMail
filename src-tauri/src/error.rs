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
}

impl AeroError {
    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            AeroError::Database(_) => ErrorPayload {
                code: "DATABASE_ERROR".to_string(),
                args: vec![],
            },
            AeroError::AccountNotFound(id) => ErrorPayload {
                code: "ACCOUNT_NOT_FOUND".to_string(),
                args: vec![id.clone()],
            },
            AeroError::InvalidConfig(reason) => ErrorPayload {
                code: "INVALID_ACCOUNT_CONFIG".to_string(),
                args: vec![reason.clone()],
            },
            AeroError::ConnectionTestFailed(reason) => ErrorPayload {
                code: "CONNECTION_TEST_FAILED".to_string(),
                args: vec![reason.clone()],
            },
            AeroError::Internal(reason) => ErrorPayload {
                code: "INTERNAL_ERROR".to_string(),
                args: vec![reason.clone()],
            },
        }
    }
}

impl From<rusqlite::Error> for AeroError {
    fn from(err: rusqlite::Error) -> Self {
        AeroError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AeroError {
    fn from(err: serde_json::Error) -> Self {
        AeroError::Internal(err.to_string())
    }
}

impl From<AeroError> for ErrorPayload {
    fn from(err: AeroError) -> Self {
        err.to_payload()
    }
}
