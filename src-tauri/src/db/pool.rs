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
    /// Opens (or creates) the database file and runs migrations.
    ///
    /// # Errors
    ///
    /// Returns an error if the database file cannot be opened or if any migration fails.
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

    /// Acquires a lock on the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the mutex is poisoned.
    pub fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AeroError> {
        self.connection
            .lock()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Returns the path to the database file.
    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Inserts or updates a setting in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
               value = excluded.value,
               updated_at = excluded.updated_at",
            (key, value, chrono::Utc::now().timestamp()),
        )?;
        drop(conn);
        Ok(())
    }

    /// Retrieves a setting from the database by key.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    // ---- AI Provider CRUD ----

    /// Lists all AI providers from the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_ai_providers(&self) -> Result<Vec<crate::models::ai::AiProvider>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, kind, api_key_encrypted, base_url, model, max_tokens FROM ai_providers",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::ai::AiProvider {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: serde_json::from_str(&row.get::<_, String>(2)?)
                    .unwrap_or(crate::models::ai::AiProviderKind::CustomOpenAICompatible),
                api_key_encrypted: row.get::<_, Vec<u8>>(3)?,
                base_url: row.get(4)?,
                model: row.get(5)?,
                max_tokens: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Retrieves a single AI provider by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not found or the query fails.
    pub fn get_ai_provider(&self, id: &str) -> Result<crate::models::ai::AiProvider, AeroError> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT id, name, kind, api_key_encrypted, base_url, model, max_tokens FROM ai_providers WHERE id = ?1",
            [id],
            |row| {
                Ok(crate::models::ai::AiProvider {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: serde_json::from_str(&row.get::<_, String>(2)?)
                        .unwrap_or(crate::models::ai::AiProviderKind::CustomOpenAICompatible),
                    api_key_encrypted: row.get::<_, Vec<u8>>(3)?,
                    base_url: row.get(4)?,
                    model: row.get(5)?,
                    max_tokens: row.get(6)?,
                })
            },
        )
        .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Inserts or updates an AI provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_ai_provider(&self, p: &crate::models::ai::AiProvider) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let kind_str = serde_json::to_string(&p.kind)?;
        conn.execute(
            "INSERT INTO ai_providers (id, name, kind, api_key_encrypted, base_url, model, max_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, kind=excluded.kind,
             api_key_encrypted=excluded.api_key_encrypted, base_url=excluded.base_url,
             model=excluded.model, max_tokens=excluded.max_tokens",
            (
                &p.id,
                &p.name,
                kind_str,
                &p.api_key_encrypted,
                &p.base_url,
                &p.model,
                &p.max_tokens,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Deletes an AI provider by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_ai_provider(&self, id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM ai_providers WHERE id = ?1", [id])?;
        drop(conn);
        Ok(())
    }

    // ---- AI Chat Session CRUD ----

    /// Retrieves a chat session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or the query fails.
    pub fn get_chat_session(&self, id: &str) -> Result<crate::models::ai::AiChatSession, AeroError> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT id, title, provider_id, model, context_mail_id, created_at, updated_at
             FROM ai_chat_sessions WHERE id = ?1",
            [id],
            |row| {
                Ok(crate::models::ai::AiChatSession {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    provider_id: row.get(2)?,
                    model: row.get(3)?,
                    context_mail_id: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Lists all chat sessions ordered by most recently updated.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_chat_sessions(&self) -> Result<Vec<crate::models::ai::AiChatSession>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, provider_id, model, context_mail_id, created_at, updated_at
             FROM ai_chat_sessions ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::ai::AiChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                provider_id: row.get(2)?,
                model: row.get(3)?,
                context_mail_id: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Inserts or updates a chat session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_chat_session(&self, s: &crate::models::ai::AiChatSession) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO ai_chat_sessions (id, title, provider_id, model, context_mail_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET title=excluded.title, provider_id=excluded.provider_id,
             model=excluded.model, context_mail_id=excluded.context_mail_id,
             updated_at=excluded.updated_at",
            (
                &s.id,
                &s.title,
                &s.provider_id,
                &s.model,
                &s.context_mail_id,
                s.created_at,
                s.updated_at,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Deletes a chat session and its messages by session ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_chat_session(&self, id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM ai_chat_messages WHERE session_id = ?1", [id])?;
        conn.execute("DELETE FROM ai_chat_sessions WHERE id = ?1", [id])?;
        drop(conn);
        Ok(())
    }

    /// Updates the `updated_at` timestamp of a chat session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn update_chat_session_timestamp(&self, session_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE ai_chat_sessions SET updated_at = ?1 WHERE id = ?2",
            (now, session_id),
        )?;
        drop(conn);
        Ok(())
    }

    // ---- AI Chat Message CRUD ----

    /// Retrieves all messages for a chat session, ordered by creation time.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_chat_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<crate::models::ai::AiChatMessage>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, created_at
             FROM ai_chat_messages WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([session_id], |row| {
            Ok(crate::models::ai::AiChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    // ---- Mail Context Helpers ----

    /// Returns the subject of a mail by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_mail_subject(&self, mail_id: &str) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT subject FROM mails WHERE id = ?1")?;
        let mut rows = stmt.query([mail_id])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Ok(None)
        }
    }

    /// Returns the `from_address` of a mail by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_mail_from_address(&self, mail_id: &str) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT from_address FROM mails WHERE id = ?1")?;
        let mut rows = stmt.query([mail_id])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Ok(None)
        }
    }

    /// Returns the `body_text` of a mail by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_mail_body_text(&self, mail_id: &str) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT body_text FROM mails WHERE id = ?1")?;
        let mut rows = stmt.query([mail_id])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Ok(None)
        }
    }

    /// Inserts a new chat message and returns it.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn insert_chat_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<crate::models::ai::AiChatMessage, AeroError> {
        let conn = self.connection()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO ai_chat_messages (id, session_id, role, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id, session_id, role, content, now),
        )?;
        drop(conn);
        Ok(crate::models::ai::AiChatMessage {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: now,
        })
    }
}
