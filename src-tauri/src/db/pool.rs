use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

use super::migrations::run_migrations;
use crate::error::AeroError;

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    // Mutex<Connection> is acceptable for a desktop single-user app in phase 1.
    // Replace with r2d2/deadpool pool in later phases if concurrent access grows.
    connection: Mutex<Connection>,
}

impl Database {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, AeroError> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AeroError::Internal(e.to_string()))?;

        std::fs::create_dir_all(&app_dir)
            .map_err(|e| AeroError::Internal(format!("failed to create app dir: {e}")))?;

        let db_path = app_dir.join("aeromail.db");
        let mut conn = Connection::open(&db_path)
            .map_err(|e| AeroError::Database(format!("failed to open database: {e}")))?;

        run_migrations(&mut conn)?;

        Ok(Self {
            path: db_path,
            connection: Mutex::new(conn),
        })
    }

    pub fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AeroError> {
        self.connection
            .lock()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
