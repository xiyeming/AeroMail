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

    // ---- Translation Provider CRUD ----

    /// Lists all translation providers.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_translation_providers(
        &self,
    ) -> Result<Vec<crate::models::translation::TranslationProvider>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT config_json FROM translation_providers",
        )?;
        let rows = stmt.query_map([], |row| {
            let config_json: String = row.get(0)?;
            let provider: crate::models::translation::TranslationProvider =
                serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            Ok(provider)
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Retrieves a single translation provider by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not found or the query fails.
    pub fn get_translation_provider(
        &self,
        id: &str,
    ) -> Result<crate::models::translation::TranslationProvider, AeroError> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT config_json FROM translation_providers WHERE id = ?1",
            [id],
            |row| {
                let config_json: String = row.get(0)?;
                serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
            },
        )
        .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Inserts or updates a translation provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_translation_provider(
        &self,
        p: &crate::models::translation::TranslationProvider,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let (id, name, provider_type) = match p {
            crate::models::translation::TranslationProvider::Traditional {
                id, name, ..
            } => (id, name, "traditional"),
            crate::models::translation::TranslationProvider::Ai { id, name, .. } => {
                (id, name, "ai")
            }
        };
        let config_json = serde_json::to_string(p)?;
        conn.execute(
            "INSERT INTO translation_providers (id, name, provider_type, config_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, provider_type=excluded.provider_type, config_json=excluded.config_json",
            (id, name, provider_type, config_json),
        )?;
        drop(conn);
        Ok(())
    }

    /// Deletes a translation provider by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_translation_provider(&self, id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM translation_providers WHERE id = ?1",
            [id],
        )?;
        drop(conn);
        Ok(())
    }

    /// Retrieves a cached translation by source hash, target language, and provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_translation(
        &self,
        source_hash: &str,
        target_lang: &str,
        provider_id: &str,
    ) -> Result<Option<crate::models::translation::CachedTranslation>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, source_hash, target_lang, provider_id, translated_text, created_at
             FROM translations WHERE source_hash = ?1 AND target_lang = ?2 AND provider_id = ?3",
        )?;
        let mut rows = stmt.query(rusqlite::params![
            source_hash,
            target_lang,
            provider_id
        ])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::translation::CachedTranslation {
                id: row.get(0)?,
                source_hash: row.get(1)?,
                target_lang: row.get(2)?,
                provider_id: row.get(3)?,
                translated_text: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves any cached translation by source hash and target language,
    /// regardless of which provider produced it.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_any_translation(
        &self,
        source_hash: &str,
        target_lang: &str,
    ) -> Result<Option<crate::models::translation::CachedTranslation>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, source_hash, target_lang, provider_id, translated_text, created_at
             FROM translations WHERE source_hash = ?1 AND target_lang = ?2
             ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(rusqlite::params![source_hash, target_lang])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::translation::CachedTranslation {
                id: row.get(0)?,
                source_hash: row.get(1)?,
                target_lang: row.get(2)?,
                provider_id: row.get(3)?,
                translated_text: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Saves a translation to the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn save_translation(
        &self,
        source_hash: &str,
        target_lang: &str,
        provider_id: &str,
        translated_text: &str,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO translations (id, source_hash, target_lang, provider_id, translated_text, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&id, source_hash, target_lang, provider_id, translated_text, now),
        )?;
        drop(conn);
        Ok(())
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

    // ---- Folder CRUD ----

    /// Inserts or updates a folder and returns its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_folder(
        &self,
        account_id: &str,
        name: &str,
        path: &str,
        uid_validity: Option<i64>,
    ) -> Result<String, AeroError> {
        let conn = self.connection()?;
        // Check if folder already exists for this account
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM folders WHERE account_id = ?1 AND path = ?2",
                (account_id, path),
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            conn.execute(
                "UPDATE folders SET name = ?1, uid_validity = ?2 WHERE id = ?3",
                (name, uid_validity, &id),
            )?;
            Ok(id)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO folders (id, account_id, name, path, uid_validity)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (&id, account_id, name, path, uid_validity),
            )?;
            Ok(id)
        }
    }

    /// Lists all folders for an account.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_folders(
        &self,
        account_id: &str,
    ) -> Result<Vec<crate::models::mail::FolderInfo>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, path, unread_count, total_count, uid_validity, last_sync_at
             FROM folders WHERE account_id = ?1 ORDER BY path ASC",
        )?;
        let rows = stmt.query_map([account_id], |row| {
            Ok(crate::models::mail::FolderInfo {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                unread_count: row.get(4)?,
                total_count: row.get(5)?,
                uid_validity: row.get(6)?,
                last_sync_at: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Gets a folder by account_id and path.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_folder_by_path(
        &self,
        account_id: &str,
        path: &str,
    ) -> Result<Option<crate::models::mail::FolderInfo>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, path, unread_count, total_count, uid_validity, last_sync_at
             FROM folders WHERE account_id = ?1 AND path = ?2",
        )?;
        let mut rows = stmt.query((account_id, path))?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::mail::FolderInfo {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                unread_count: row.get(4)?,
                total_count: row.get(5)?,
                uid_validity: row.get(6)?,
                last_sync_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Updates folder sync metadata after a successful sync.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn update_folder_sync(
        &self,
        folder_id: &str,
        uid_validity: i64,
        unread: i64,
        total: i64,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE folders SET uid_validity = ?1, unread_count = ?2, total_count = ?3, last_sync_at = ?4
             WHERE id = ?5",
            (uid_validity, unread, total, now, folder_id),
        )?;
        drop(conn);
        Ok(())
    }

    // ---- Mail CRUD ----

    /// Inserts or updates a mail item.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_mail(&self, mail: &crate::models::mail::MailDetail) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO mails (id, account_id, folder_id, uid, subject, from_name, from_address,
             to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, flags, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(id) DO UPDATE SET
               subject=excluded.subject, from_name=excluded.from_name, from_address=excluded.from_address,
               to_addresses=excluded.to_addresses, cc_addresses=excluded.cc_addresses,
               date=excluded.date, body_html=excluded.body_html, body_text=excluded.body_text,
               is_read=excluded.is_read, is_starred=excluded.is_starred, flags=excluded.flags",
            (
                &mail.id,
                &mail.account_id,
                &mail.folder_id,
                mail.uid,
                &mail.subject,
                &mail.from_name,
                &mail.from_address,
                &mail.to_addresses,
                &mail.cc_addresses,
                mail.date,
                &mail.body_html,
                &mail.body_text,
                mail.is_read,
                mail.is_starred,
                &mail.flags,
                now,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Lists mails in a folder with pagination.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_mails(
        &self,
        folder_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<crate::models::mail::MailHeader>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             date, is_read, is_starred, 0 as has_attachments
             FROM mails WHERE folder_id = ?1
             ORDER BY date DESC, uid DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map((folder_id, limit, offset), |row| {
            Ok(crate::models::mail::MailHeader {
                id: row.get(0)?,
                account_id: row.get(1)?,
                folder_id: row.get(2)?,
                uid: row.get::<_, i64>(3)? as u32,
                subject: row.get(4)?,
                from_name: row.get(5)?,
                from_address: row.get(6)?,
                date: row.get(7)?,
                is_read: row.get::<_, i64>(8)? != 0,
                is_starred: row.get::<_, i64>(9)? != 0,
                has_attachments: row.get::<_, i64>(10)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Gets full mail detail by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_mail_detail(
        &self,
        mail_id: &str,
    ) -> Result<Option<crate::models::mail::MailDetail>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, flags
             FROM mails WHERE id = ?1",
        )?;
        let mut rows = stmt.query([mail_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::mail::MailDetail {
                id: row.get(0)?,
                account_id: row.get(1)?,
                folder_id: row.get(2)?,
                uid: row.get::<_, i64>(3)? as u32,
                subject: row.get(4)?,
                from_name: row.get(5)?,
                from_address: row.get(6)?,
                to_addresses: row.get(7)?,
                cc_addresses: row.get(8)?,
                date: row.get(9)?,
                body_html: row.get(10)?,
                body_text: row.get(11)?,
                is_read: row.get::<_, i64>(12)? != 0,
                is_starred: row.get::<_, i64>(13)? != 0,
                flags: row.get(14)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Marks a mail as read or unread.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn mark_mail_read(&self, mail_id: &str, is_read: bool) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE mails SET is_read = ?1 WHERE id = ?2",
            (is_read as i64, mail_id),
        )?;
        drop(conn);
        Ok(())
    }

    /// Toggles the starred status of a mail and returns the new state.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn toggle_mail_star(&self, mail_id: &str) -> Result<bool, AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE mails SET is_starred = CASE WHEN is_starred = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            [mail_id],
        )?;
        let new_state: i64 = conn.query_row(
            "SELECT is_starred FROM mails WHERE id = ?1",
            [mail_id],
            |row| row.get(0),
        )?;
        drop(conn);
        Ok(new_state != 0)
    }

    /// Gets the maximum UID in a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_max_uid(&self, folder_id: &str) -> Result<Option<u32>, AeroError> {
        let conn = self.connection()?;
        let mut stmt =
            conn.prepare("SELECT MAX(uid) FROM mails WHERE folder_id = ?1")?;
        let mut rows = stmt.query([folder_id])?;
        if let Some(row) = rows.next()? {
            let max_uid: Option<i64> = row.get(0)?;
            Ok(max_uid.map(|v| v as u32))
        } else {
            Ok(None)
        }
    }

    /// Counts unread mails for an account across all folders.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn count_unread(&self, account_id: &str) -> Result<u32, AeroError> {
        let conn = self.connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mails WHERE account_id = ?1 AND is_read = 0",
            [account_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Deletes a mail by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_mail(&self, mail_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM mails WHERE id = ?1", [mail_id])?;
        drop(conn);
        Ok(())
    }

    /// Moves a mail to a different folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn move_mail(&self, mail_id: &str, target_folder_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE mails SET folder_id = ?1 WHERE id = ?2",
            (target_folder_id, mail_id),
        )?;
        drop(conn);
        Ok(())
    }

    /// Gets all attachments for a mail.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_attachments(
        &self,
        mail_id: &str,
    ) -> Result<Vec<crate::models::mail::AttachmentInfo>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, mail_id, filename, mime_type, size, content_id, is_inline
             FROM attachments WHERE mail_id = ?1",
        )?;
        let rows = stmt.query_map([mail_id], |row| {
            Ok(crate::models::mail::AttachmentInfo {
                id: row.get(0)?,
                mail_id: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                content_id: row.get(5)?,
                is_inline: row.get::<_, i64>(6)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    // ---- Draft CRUD ----

    /// Inserts or updates a compose draft.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_draft(
        &self,
        draft: &crate::models::compose::ComposeDraft,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let to_json = serde_json::to_string(&draft.to)?;
        let cc_json = serde_json::to_string(&draft.cc)?;
        let bcc_json = serde_json::to_string(&draft.bcc)?;
        let reply_context_json = serde_json::to_string(&draft.reply_context)?;
        let attachments_json = serde_json::to_string(&draft.attachments)?;
        conn.execute(
            "INSERT INTO drafts (id, account_id, subject, to_addresses, cc_addresses, bcc_addresses,
             reply_context_json, body_html, body_text, attachments_json, saved_at, synced_at, remote_uid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
               account_id=excluded.account_id, subject=excluded.subject,
               to_addresses=excluded.to_addresses, cc_addresses=excluded.cc_addresses,
               bcc_addresses=excluded.bcc_addresses, reply_context_json=excluded.reply_context_json,
               body_html=excluded.body_html, body_text=excluded.body_text,
               attachments_json=excluded.attachments_json, saved_at=excluded.saved_at,
               synced_at=excluded.synced_at, remote_uid=excluded.remote_uid",
            (
                &draft.id,
                &draft.account_id,
                &draft.subject,
                to_json,
                cc_json,
                bcc_json,
                reply_context_json,
                &draft.body_html,
                &draft.body_text,
                attachments_json,
                draft.saved_at,
                draft.synced_at,
                draft.remote_uid.map(|v| v as i64),
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Retrieves a draft by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_draft(
        &self,
        draft_id: &str,
    ) -> Result<Option<crate::models::compose::ComposeDraft>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, subject, to_addresses, cc_addresses, bcc_addresses,
             reply_context_json, body_html, body_text, attachments_json, saved_at,
             synced_at, remote_uid FROM drafts WHERE id = ?1",
        )?;
        let mut rows = stmt.query([draft_id])?;
        if let Some(row) = rows.next()? {
            let to: Vec<String> = serde_json::from_str(&row.get::<_, String>(3)?)?;
            let cc: Vec<String> = serde_json::from_str(&row.get::<_, String>(4)?)?;
            let bcc: Vec<String> = serde_json::from_str(&row.get::<_, String>(5)?)?;
            let reply_context: Option<crate::models::compose::ReplyContext> =
                serde_json::from_str(&row.get::<_, String>(6)?)?;
            let attachments: Vec<crate::models::compose::AttachmentDraft> =
                serde_json::from_str(&row.get::<_, String>(9)?)?;
            Ok(Some(crate::models::compose::ComposeDraft {
                id: row.get(0)?,
                account_id: row.get(1)?,
                subject: row.get(2)?,
                to,
                cc,
                bcc,
                reply_context,
                body_html: row.get(7)?,
                body_text: row.get(8)?,
                attachments,
                saved_at: row.get(10)?,
                synced_at: row.get(11)?,
                remote_uid: row.get::<_, Option<i64>>(12)?.map(|v| v as u32),
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists drafts for an account or all drafts if account_id is None.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_drafts(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<crate::models::compose::ComposeDraftSummary>, AeroError> {
        let conn = self.connection()?;
        let sql = match account_id {
            Some(_) => "SELECT id, account_id, subject, to_addresses, saved_at, attachments_json
             FROM drafts WHERE account_id = ?1 ORDER BY saved_at DESC",
            None => "SELECT id, account_id, subject, to_addresses, saved_at, attachments_json
             FROM drafts ORDER BY saved_at DESC",
        };
        let mut stmt = conn.prepare(sql)?;
        let mut rows = match account_id {
            Some(id) => stmt.query([id])?,
            None => stmt.query([])?,
        };
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let to: Vec<String> = serde_json::from_str(&row.get::<_, String>(3)?)
                .unwrap_or_default();
            let attachments: Vec<crate::models::compose::AttachmentDraft> =
                serde_json::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or_default();
            result.push(crate::models::compose::ComposeDraftSummary {
                id: row.get(0)?,
                account_id: row.get(1)?,
                subject: row.get(2)?,
                to,
                saved_at: row.get(4)?,
                has_attachments: !attachments.is_empty(),
            });
        }
        Ok(result)
    }

    /// Deletes a draft by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_draft(&self, draft_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM drafts WHERE id = ?1", [draft_id])?;
        drop(conn);
        Ok(())
    }

    /// Gets the account_id for a draft.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_draft_account_id(
        &self,
        draft_id: &str,
    ) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT account_id FROM drafts WHERE id = ?1")?;
        let mut rows = stmt.query([draft_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    // ---- Search Index Helpers ----

    /// Updates the indexed_at timestamp for a mail.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn update_mail_indexed_at(&self, mail_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE mails SET indexed_at = ?1 WHERE id = ?2",
            (now, mail_id),
        )?;
        drop(conn);
        Ok(())
    }

    /// Gets mails that haven't been indexed yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_unindexed_mails(
        &self,
    ) -> Result<Vec<crate::models::mail::MailDetail>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, flags
             FROM mails WHERE indexed_at IS NULL LIMIT 100",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::mail::MailDetail {
                id: row.get(0)?,
                account_id: row.get(1)?,
                folder_id: row.get(2)?,
                uid: row.get::<_, i64>(3)? as u32,
                subject: row.get(4)?,
                from_name: row.get(5)?,
                from_address: row.get(6)?,
                to_addresses: row.get(7)?,
                cc_addresses: row.get(8)?,
                date: row.get(9)?,
                body_html: row.get(10)?,
                body_text: row.get(11)?,
                is_read: row.get::<_, i64>(12)? != 0,
                is_starred: row.get::<_, i64>(13)? != 0,
                flags: row.get(14)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }
}
