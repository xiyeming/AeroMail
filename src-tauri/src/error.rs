use serde::Serialize;
use thiserror::Error;

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

impl From<AeroError> for String {
    fn from(err: AeroError) -> Self {
        serde_json::to_string(&err).unwrap_or_else(|_| err.to_string())
    }
}
