use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AeroError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<rusqlite::Error> for AeroError {
    fn from(err: rusqlite::Error) -> Self {
        AeroError::Database(err.to_string())
    }
}

impl From<std::io::Error> for AeroError {
    fn from(err: std::io::Error) -> Self {
        AeroError::Unknown(err.to_string())
    }
}
