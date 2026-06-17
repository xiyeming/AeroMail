use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, crate::error::AeroError> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| crate::error::AeroError::Unknown(e.to_string()))?;
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| crate::error::AeroError::Unknown(e.to_string()))?;
        let db_path = app_dir.join("aeromail.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| crate::error::AeroError::Database(e.to_string()))?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn get_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, crate::error::AeroError> {
        self.conn
            .lock()
            .map_err(|e| crate::error::AeroError::Database(e.to_string()))
    }
}
