#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::significant_drop_tightening
)]

use std::path::PathBuf;

use chrono::Utc;
use std::sync::Mutex;

use rusqlite::{Connection, params};
use tauri::Manager;

use super::migrations::run_migrations;
use crate::error::AeroError;

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
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

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

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

    /// Runs the provided closure inside a database transaction.
    ///
    /// The closure receives a `&Connection` and returns a `Result<T, AeroError>`.
    /// If the closure returns `Ok`, the transaction is committed; otherwise it is rolled back.
    pub fn with_transaction<T, F>(&self, f: F) -> Result<T, AeroError>
    where
        F: FnOnce(&Connection) -> Result<T, AeroError>,
    {
        let mut conn = self.connection()?;
        let tx = conn
            .transaction()
            .map_err(|e| AeroError::Database(e.to_string()))?;
        match f(&tx) {
            Ok(result) => {
                tx.commit()
                    .map_err(|e| AeroError::Database(e.to_string()))?;
                Ok(result)
            }
            Err(e) => {
                let _ = tx.rollback();
                Err(e)
            }
        }
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
    pub fn get_chat_session(
        &self,
        id: &str,
    ) -> Result<crate::models::ai::AiChatSession, AeroError> {
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
    pub fn upsert_chat_session(
        &self,
        s: &crate::models::ai::AiChatSession,
    ) -> Result<(), AeroError> {
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

    /// Clears all messages in a chat session while keeping the session itself.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn clear_chat_messages(&self, session_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM ai_chat_messages WHERE session_id = ?1",
            [session_id],
        )?;
        drop(conn);
        Ok(())
    }

    /// Renames a chat session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn rename_chat_session(&self, session_id: &str, title: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE ai_chat_sessions SET title = ?1 WHERE id = ?2",
            (title, session_id),
        )?;
        drop(conn);
        Ok(())
    }

    /// Updates the provider and model of a chat session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn set_chat_session_provider(
        &self,
        session_id: &str,
        provider_id: &str,
        model: &str,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE ai_chat_sessions SET provider_id = ?1, model = ?2 WHERE id = ?3",
            (provider_id, model, session_id),
        )?;
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
            "SELECT id, session_id, role, content, thinking, created_at
             FROM ai_chat_messages WHERE session_id = ?1 ORDER BY created_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map([session_id], |row| {
            Ok(crate::models::ai::AiChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                thinking: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Inserts an AI usage log entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn insert_ai_usage_log(
        &self,
        log: &crate::models::ai::AiUsageLog,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO ai_usage_logs (
                id, session_id, provider_id, model,
                prompt_tokens, completion_tokens, total_tokens,
                estimated, cost, currency, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                &log.id,
                &log.session_id,
                &log.provider_id,
                &log.model,
                log.prompt_tokens,
                log.completion_tokens,
                log.total_tokens,
                log.estimated,
                log.cost,
                &log.currency,
                log.created_at,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Returns aggregate AI usage, optionally filtered by provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_ai_usage_summary(
        &self,
        provider_id: Option<&str>,
    ) -> Result<Vec<crate::models::ai::AiUsageSummary>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT provider_id, model,
                    COALESCE(SUM(prompt_tokens), 0),
                    COALESCE(SUM(completion_tokens), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(cost), 0.0),
                    COALESCE(MIN(currency), 'USD')
             FROM ai_usage_logs
             WHERE (?1 IS NULL OR provider_id = ?1)
             GROUP BY provider_id, model
             ORDER BY provider_id, model",
        )?;
        let rows = stmt.query_map([provider_id], |row| {
            Ok(crate::models::ai::AiUsageSummary {
                provider_id: row.get(0)?,
                model: row.get(1)?,
                total_prompt_tokens: row.get::<_, i64>(2)? as u64,
                total_completion_tokens: row.get::<_, i64>(3)? as u64,
                total_tokens: row.get::<_, i64>(4)? as u64,
                total_cost: row.get(5)?,
                currency: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Returns aggregate AI usage for a single chat session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_ai_session_usage(
        &self,
        session_id: &str,
    ) -> Result<Option<crate::models::ai::AiUsageSummary>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT provider_id, model,
                    COALESCE(SUM(prompt_tokens), 0),
                    COALESCE(SUM(completion_tokens), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(cost), 0.0),
                    COALESCE(MIN(currency), 'USD')
             FROM ai_usage_logs
             WHERE session_id = ?1
             GROUP BY provider_id, model
             LIMIT 1",
        )?;
        let mut rows = stmt.query([session_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::ai::AiUsageSummary {
                provider_id: row.get(0)?,
                model: row.get(1)?,
                total_prompt_tokens: row.get::<_, i64>(2)? as u64,
                total_completion_tokens: row.get::<_, i64>(3)? as u64,
                total_tokens: row.get::<_, i64>(4)? as u64,
                total_cost: row.get(5)?,
                currency: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves pricing for a provider/model combination.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_ai_provider_pricing(
        &self,
        provider_id: &str,
        model: &str,
    ) -> Result<Option<crate::models::ai::AiProviderPricing>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, provider_id, model, input_price_per_1k, output_price_per_1k,
                    currency, effective_from
             FROM ai_provider_pricing
             WHERE provider_id = ?1 AND model = ?2
             LIMIT 1",
        )?;
        let mut rows = stmt.query([provider_id, model])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::ai::AiProviderPricing {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                model: row.get(2)?,
                input_price_per_1k: row.get(3)?,
                output_price_per_1k: row.get(4)?,
                currency: row.get(5)?,
                effective_from: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Inserts or updates pricing for a provider/model combination.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_ai_provider_pricing(
        &self,
        pricing: &crate::models::ai::AiProviderPricing,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO ai_provider_pricing (
                id, provider_id, model, input_price_per_1k, output_price_per_1k,
                currency, effective_from
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(provider_id, model) DO UPDATE SET
                input_price_per_1k = excluded.input_price_per_1k,
                output_price_per_1k = excluded.output_price_per_1k,
                currency = excluded.currency,
                effective_from = excluded.effective_from",
            (
                &pricing.id,
                &pricing.provider_id,
                &pricing.model,
                pricing.input_price_per_1k,
                pricing.output_price_per_1k,
                &pricing.currency,
                pricing.effective_from,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Lists all configured AI provider pricing records.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_ai_provider_pricing(
        &self,
    ) -> Result<Vec<crate::models::ai::AiProviderPricing>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, provider_id, model, input_price_per_1k, output_price_per_1k,
                    currency, effective_from
             FROM ai_provider_pricing
             ORDER BY provider_id, model",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::ai::AiProviderPricing {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                model: row.get(2)?,
                input_price_per_1k: row.get(3)?,
                output_price_per_1k: row.get(4)?,
                currency: row.get(5)?,
                effective_from: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    // ---- AI MCP Servers ----

    /// Inserts or updates an MCP server configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the database write fails.
    pub fn upsert_ai_mcp_server(
        &self,
        server: &crate::models::ai::AiMcpServer,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let args_json = server
            .args
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| AeroError::Database(format!("failed to serialize args: {e}")))?;
        conn.execute(
            "INSERT INTO ai_mcp_servers (
                id, name, transport, command, args, url, env_json, is_enabled,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                transport = excluded.transport,
                command = excluded.command,
                args = excluded.args,
                url = excluded.url,
                env_json = excluded.env_json,
                is_enabled = excluded.is_enabled,
                updated_at = excluded.updated_at",
            (
                &server.id,
                &server.name,
                &server.transport,
                &server.command,
                &args_json,
                &server.url,
                &server.env_json,
                server.is_enabled,
                server.created_at,
                server.updated_at,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Deletes an MCP server configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_ai_mcp_server(&self, id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM ai_mcp_servers WHERE id = ?1", [id])?;
        drop(conn);
        Ok(())
    }

    /// Lists all MCP server configurations.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_ai_mcp_servers(&self) -> Result<Vec<crate::models::ai::AiMcpServer>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, transport, command, args, url, env_json, is_enabled,
                    created_at, updated_at
             FROM ai_mcp_servers
             ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            let args_json: Option<String> = row.get(4)?;
            let args = args_json
                .map(|j| serde_json::from_str::<Vec<String>>(&j))
                .transpose()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            Ok(crate::models::ai::AiMcpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                transport: row.get(2)?,
                command: row.get(3)?,
                args,
                url: row.get(5)?,
                env_json: row.get(6)?,
                is_enabled: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    // ---- AI Skills ----

    /// Inserts or updates a skill configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the database write fails.
    pub fn upsert_ai_skill(&self, skill: &crate::models::ai::AiSkill) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let args_json = skill
            .args
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| AeroError::Database(format!("failed to serialize args: {e}")))?;
        conn.execute(
            "INSERT INTO ai_skills (
                id, name, description, input_schema_json, command, args, working_dir,
                timeout_seconds, is_enabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                input_schema_json = excluded.input_schema_json,
                command = excluded.command,
                args = excluded.args,
                working_dir = excluded.working_dir,
                timeout_seconds = excluded.timeout_seconds,
                is_enabled = excluded.is_enabled,
                updated_at = excluded.updated_at",
            (
                &skill.id,
                &skill.name,
                &skill.description,
                &skill.input_schema_json,
                &skill.command,
                &args_json,
                &skill.working_dir,
                skill.timeout_seconds,
                skill.is_enabled,
                skill.created_at,
                skill.updated_at,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Deletes a skill configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_ai_skill(&self, id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM ai_skills WHERE id = ?1", [id])?;
        drop(conn);
        Ok(())
    }

    /// Lists all skill configurations.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_ai_skills(&self) -> Result<Vec<crate::models::ai::AiSkill>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, input_schema_json, command, args, working_dir,
                    timeout_seconds, is_enabled, created_at, updated_at
             FROM ai_skills
             ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            let args_json: Option<String> = row.get(5)?;
            let args = args_json
                .map(|j| serde_json::from_str::<Vec<String>>(&j))
                .transpose()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            Ok(crate::models::ai::AiSkill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                input_schema_json: row.get(3)?,
                command: row.get(4)?,
                args,
                working_dir: row.get(6)?,
                timeout_seconds: row.get(7)?,
                is_enabled: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
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
        let mut stmt = conn.prepare("SELECT config_json FROM translation_providers")?;
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
            crate::models::translation::TranslationProvider::Traditional { id, name, .. } => {
                (id, name, "traditional")
            }
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
        conn.execute("DELETE FROM translation_providers WHERE id = ?1", [id])?;
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
        let mut rows = stmt.query(rusqlite::params![source_hash, target_lang, provider_id])?;
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
        thinking: Option<&str>,
    ) -> Result<crate::models::ai::AiChatMessage, AeroError> {
        let conn = self.connection()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO ai_chat_messages (id, session_id, role, content, thinking, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&id, session_id, role, content, thinking, now),
        )?;
        drop(conn);
        Ok(crate::models::ai::AiChatMessage {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            thinking: thinking.map(std::string::ToString::to_string),
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
        uid_next: Option<i64>,
    ) -> Result<String, AeroError> {
        let conn = self.connection()?;
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM folders WHERE account_id = ?1 AND path = ?2",
                (account_id, path),
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            conn.execute(
                "UPDATE folders SET name = ?1, uid_validity = ?2, uid_next = ?3 WHERE id = ?4",
                (name, uid_validity, uid_next, &id),
            )?;
            Ok(id)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO folders (id, account_id, name, path, uid_validity, uid_next)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (&id, account_id, name, path, uid_validity, uid_next),
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
            "SELECT id, account_id, name, path, unread_count, total_count, uid_validity, uid_next, last_sync_at
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
                uid_next: row.get(7)?,
                last_sync_at: row.get(8)?,
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
            "SELECT id, account_id, name, path, unread_count, total_count, uid_validity, uid_next, last_sync_at
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
                uid_next: row.get(7)?,
                last_sync_at: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Gets a folder by its local ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_folder_by_id(
        &self,
        folder_id: &str,
    ) -> Result<Option<crate::models::mail::FolderInfo>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, path, unread_count, total_count, uid_validity, uid_next, last_sync_at
             FROM folders WHERE id = ?1",
        )?;
        let mut rows = stmt.query([folder_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::mail::FolderInfo {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                unread_count: row.get(4)?,
                total_count: row.get(5)?,
                uid_validity: row.get(6)?,
                uid_next: row.get(7)?,
                last_sync_at: row.get(8)?,
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
             to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, is_archived, is_spam, flags, message_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
             ON CONFLICT(id) DO UPDATE SET
               account_id=excluded.account_id, folder_id=excluded.folder_id, uid=excluded.uid,
               subject=excluded.subject, from_name=excluded.from_name, from_address=excluded.from_address,
               to_addresses=excluded.to_addresses, cc_addresses=excluded.cc_addresses,
               date=excluded.date, body_html=excluded.body_html, body_text=excluded.body_text,
               is_read=excluded.is_read, is_starred=excluded.is_starred, is_archived=excluded.is_archived, is_spam=excluded.is_spam,
               flags=excluded.flags, message_id=excluded.message_id",
            params![
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
                mail.is_archived,
                mail.is_spam,
                &mail.flags,
                &mail.message_id,
                now,
            ],
        )?;
        drop(conn);
        Ok(())
    }

    /// Upserts multiple mails in a single transaction for better performance.
    pub fn upsert_mails_batch(
        &self,
        mails: &[crate::models::mail::MailDetail],
    ) -> Result<(), AeroError> {
        if mails.is_empty() {
            return Ok(());
        }
        self.with_transaction(|conn| {
            let now = chrono::Utc::now().timestamp();
            let mut stmt = conn.prepare(
                "INSERT INTO mails (id, account_id, folder_id, uid, subject, from_name, from_address,
                 to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, is_archived, is_spam, flags, message_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
                 ON CONFLICT(id) DO UPDATE SET
                   account_id=excluded.account_id, folder_id=excluded.folder_id, uid=excluded.uid,
                   subject=excluded.subject, from_name=excluded.from_name, from_address=excluded.from_address,
                   to_addresses=excluded.to_addresses, cc_addresses=excluded.cc_addresses,
                   date=excluded.date, body_html=excluded.body_html, body_text=excluded.body_text,
                   is_read=excluded.is_read, is_starred=excluded.is_starred, is_archived=excluded.is_archived, is_spam=excluded.is_spam,
                   flags=excluded.flags, message_id=excluded.message_id",
            )?;
            for mail in mails {
                stmt.execute(params![
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
                    mail.is_archived,
                    mail.is_spam,
                    &mail.flags,
                    &mail.message_id,
                    now,
                ])?;
            }
            Ok(())
        })
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
             date, is_read, is_starred, is_archived, is_spam,
             EXISTS(SELECT 1 FROM attachments WHERE mail_id = mails.id) as has_attachments
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
                is_archived: row.get::<_, i64>(10)? != 0,
                is_spam: row.get::<_, i64>(11)? != 0,
                has_attachments: row.get::<_, i64>(12)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Returns the most recent mail headers across all accounts.
    ///
    /// Used as a fallback for @-mentions when the search index yields no results.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_recent_mails(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::mail::MailHeader>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             date, is_read, is_starred, is_archived, is_spam,
             EXISTS(SELECT 1 FROM attachments WHERE mail_id = mails.id) as has_attachments
             FROM mails
             ORDER BY date DESC, uid DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |row| {
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
                is_archived: row.get::<_, i64>(10)? != 0,
                is_spam: row.get::<_, i64>(11)? != 0,
                has_attachments: row.get::<_, i64>(12)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Searches mail headers by a fuzzy match against subject, sender name, or
    /// sender address.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query cannot be executed.
    pub fn search_mail_headers(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::mail::MailHeader>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             date, is_read, is_starred, is_archived, is_spam,
             EXISTS(SELECT 1 FROM attachments WHERE mail_id = mails.id) as has_attachments
             FROM mails
             WHERE LOWER(subject) LIKE LOWER(?1)
                OR LOWER(from_name) LIKE LOWER(?1)
                OR LOWER(from_address) LIKE LOWER(?1)
             ORDER BY date DESC, uid DESC
             LIMIT ?2",
        )?;
        let pattern = format!("%{query}%");
        let rows = stmt.query_map(params![pattern, limit], |row| {
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
                is_archived: row.get::<_, i64>(10)? != 0,
                is_spam: row.get::<_, i64>(11)? != 0,
                has_attachments: row.get::<_, i64>(12)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Returns mails from the INBOX folders of the given accounts, merged and sorted.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_inbox_mails(
        &self,
        account_ids: &[String],
        limit: u32,
        offset: u32,
    ) -> Result<Vec<crate::models::mail::MailHeader>, AeroError> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = account_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT m.id, m.account_id, m.folder_id, m.uid, m.subject, m.from_name, m.from_address,
             m.date, m.is_read, m.is_starred, m.is_archived, m.is_spam,
             EXISTS(SELECT 1 FROM attachments WHERE mail_id = m.id) as has_attachments
             FROM mails m
             JOIN folders f ON m.folder_id = f.id
             WHERE m.account_id IN ({placeholders}) AND f.path = 'INBOX'
             ORDER BY m.date DESC, m.uid DESC
             LIMIT ?{} OFFSET ?{}",
            account_ids.len() + 1,
            account_ids.len() + 2,
        );
        let conn = self.connection()?;
        let mut stmt = conn.prepare(&sql)?;
        let mut param_values: Vec<String> = account_ids.to_vec();
        param_values.push(limit.to_string());
        param_values.push(offset.to_string());
        let rows = stmt.query_map(rusqlite::params_from_iter(param_values.iter()), |row| {
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
                is_archived: row.get::<_, i64>(10)? != 0,
                is_spam: row.get::<_, i64>(11)? != 0,
                has_attachments: row.get::<_, i64>(12)? != 0,
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
             to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, is_archived, is_spam, flags, message_id
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
                is_archived: row.get::<_, i64>(14)? != 0,
                is_spam: row.get::<_, i64>(15)? != 0,
                flags: row.get(16)?,
                message_id: row.get(17)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Gets a mail header by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_mail_header(
        &self,
        mail_id: &str,
    ) -> Result<Option<crate::models::mail::MailHeader>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             date, is_read, is_starred, is_archived, is_spam,
             EXISTS(SELECT 1 FROM attachments WHERE mail_id = mails.id) as has_attachments
             FROM mails WHERE id = ?1",
        )?;
        let mut rows = stmt.query([mail_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::mail::MailHeader {
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
                is_archived: row.get::<_, i64>(10)? != 0,
                is_spam: row.get::<_, i64>(11)? != 0,
                has_attachments: row.get::<_, i64>(12)? != 0,
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
        self.with_transaction(|conn| {
            let folder_id: String = conn.query_row(
                "SELECT folder_id FROM mails WHERE id = ?1",
                [mail_id],
                |row| row.get(0),
            )?;
            conn.execute(
                "UPDATE mails SET is_read = ?1 WHERE id = ?2",
                (is_read as i64, mail_id),
            )?;
            let unread_delta = if is_read { -1 } else { 1 };
            conn.execute(
                "UPDATE folders SET unread_count = MAX(0, unread_count + ?1) WHERE id = ?2",
                (unread_delta, &folder_id),
            )?;
            Ok(())
        })
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

    /// Sets the archived status of a mail and returns the new state.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn set_mail_archived(&self, mail_id: &str, archived: bool) -> Result<bool, AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE mails SET is_archived = ?1 WHERE id = ?2",
            (archived as i64, mail_id),
        )?;
        drop(conn);
        Ok(archived)
    }

    /// Sets the spam status of a mail and returns the new state.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn set_mail_spam(&self, mail_id: &str, spam: bool) -> Result<bool, AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE mails SET is_spam = ?1 WHERE id = ?2",
            (spam as i64, mail_id),
        )?;
        drop(conn);
        Ok(spam)
    }

    /// Returns a comma-separated list of SQL-safe quoted strings for use in an
    /// `IN` clause. Input strings are assumed to be static lowercase names.
    fn quoted_in_list(values: &[&str]) -> String {
        values
            .iter()
            .map(|v| format!("'{}'", v.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn sent_folder_condition() -> String {
        let names =
            Self::quoted_in_list(&["sent", "sent items", "sent messages", "[gmail]/sent mail"]);
        format!("LOWER(f.path) IN ({names}) OR LOWER(f.name) IN ({names})")
    }

    fn spam_folder_condition() -> String {
        let names = Self::quoted_in_list(&["spam", "junk", "[gmail]/spam", "[gmail]/junk"]);
        format!("LOWER(f.path) IN ({names}) OR LOWER(f.name) IN ({names})")
    }

    fn trash_folder_condition() -> String {
        let names = Self::quoted_in_list(&[
            "trash",
            "deleted",
            "deleted items",
            "deleted messages",
            "bin",
            "[gmail]/trash",
            "[gmail]/bin",
        ]);
        format!("LOWER(f.path) IN ({names}) OR LOWER(f.name) IN ({names})")
    }

    /// Lists mails for a virtual folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails or the virtual folder is unknown.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_virtual_mails(
        &self,
        virtual_folder: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<crate::models::mail::MailHeader>, AeroError> {
        let conn = self.connection()?;
        let sql = match virtual_folder {
            "starred" => "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
                 date, is_read, is_starred, is_archived, is_spam,
                 EXISTS(SELECT 1 FROM attachments WHERE mail_id = mails.id) as has_attachments
                 FROM mails WHERE is_starred = 1
                 ORDER BY date DESC, uid DESC
                 LIMIT ?1 OFFSET ?2"
                .to_string(),
            "archived" => "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
                 date, is_read, is_starred, is_archived, is_spam,
                 EXISTS(SELECT 1 FROM attachments WHERE mail_id = mails.id) as has_attachments
                 FROM mails WHERE is_archived = 1
                 ORDER BY date DESC, uid DESC
                 LIMIT ?1 OFFSET ?2"
                .to_string(),
            "sent" => {
                let condition = Self::sent_folder_condition();
                format!(
                    "SELECT m.id, m.account_id, m.folder_id, m.uid, m.subject, m.from_name, m.from_address,
                     m.date, m.is_read, m.is_starred, m.is_archived, m.is_spam,
                     EXISTS(SELECT 1 FROM attachments WHERE mail_id = m.id) as has_attachments
                     FROM mails m JOIN folders f ON m.folder_id = f.id
                     WHERE {condition}
                     ORDER BY m.date DESC, m.uid DESC
                     LIMIT ?1 OFFSET ?2"
                )
            }
            "spam" => {
                let condition = Self::spam_folder_condition();
                format!(
                    "SELECT m.id, m.account_id, m.folder_id, m.uid, m.subject, m.from_name, m.from_address,
                     m.date, m.is_read, m.is_starred, m.is_archived, m.is_spam,
                     EXISTS(SELECT 1 FROM attachments WHERE mail_id = m.id) as has_attachments
                     FROM mails m JOIN folders f ON m.folder_id = f.id
                     WHERE (m.is_spam = 1 OR {condition})
                     ORDER BY m.date DESC, m.uid DESC
                     LIMIT ?1 OFFSET ?2"
                )
            }
            "trash" => {
                let condition = Self::trash_folder_condition();
                format!(
                    "SELECT m.id, m.account_id, m.folder_id, m.uid, m.subject, m.from_name, m.from_address,
                     m.date, m.is_read, m.is_starred, m.is_archived, m.is_spam,
                     EXISTS(SELECT 1 FROM attachments WHERE mail_id = m.id) as has_attachments
                     FROM mails m JOIN folders f ON m.folder_id = f.id
                     WHERE {condition}
                     ORDER BY m.date DESC, m.uid DESC
                     LIMIT ?1 OFFSET ?2"
                )
            }
            _ => {
                return Err(AeroError::InvalidConfig(format!(
                    "Unknown virtual folder: {virtual_folder}"
                )));
            }
        };

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
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
                is_archived: row.get::<_, i64>(10)? != 0,
                is_spam: row.get::<_, i64>(11)? != 0,
                has_attachments: row.get::<_, i64>(12)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Counts unread mails in a virtual folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn count_virtual_unread(&self, virtual_folder: &str) -> Result<u32, AeroError> {
        let conn = self.connection()?;
        let sql = match virtual_folder {
            "starred" => {
                "SELECT COUNT(*) FROM mails WHERE is_read = 0 AND is_starred = 1".to_string()
            }
            "spam" => {
                let condition = Self::spam_folder_condition();
                format!(
                    "SELECT COUNT(*) FROM mails m JOIN folders f ON m.folder_id = f.id
                     WHERE m.is_read = 0 AND (m.is_spam = 1 OR {condition})"
                )
            }
            "archived" => {
                "SELECT COUNT(*) FROM mails WHERE is_read = 0 AND is_archived = 1".to_string()
            }
            "trash" => {
                let condition = Self::trash_folder_condition();
                format!(
                    "SELECT COUNT(*) FROM mails m JOIN folders f ON m.folder_id = f.id
                     WHERE m.is_read = 0 AND {condition}"
                )
            }
            _ => return Ok(0),
        };
        let count: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
        Ok(count as u32)
    }

    /// Gets a mail ID by its folder and message ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_mail_id_by_message_id(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt =
            conn.prepare("SELECT id FROM mails WHERE folder_id = ?1 AND message_id = ?2 LIMIT 1")?;
        let mut rows = stmt.query(rusqlite::params![folder_id, message_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Gets the maximum UID in a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_max_uid(&self, folder_id: &str) -> Result<Option<u32>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT MAX(uid) FROM mails WHERE folder_id = ?1")?;
        let mut rows = stmt.query([folder_id])?;
        if let Some(row) = rows.next()? {
            let max_uid: Option<i64> = row.get(0)?;
            Ok(max_uid.map(|v| v as u32))
        } else {
            Ok(None)
        }
    }

    /// Gets the minimum UID in a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_min_uid(&self, folder_id: &str) -> Result<Option<u32>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT MIN(uid) FROM mails WHERE folder_id = ?1")?;
        let mut rows = stmt.query([folder_id])?;
        if let Some(row) = rows.next()? {
            let min_uid: Option<i64> = row.get(0)?;
            Ok(min_uid.map(|v| v as u32))
        } else {
            Ok(None)
        }
    }

    /// Counts all mails in a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn count_mails_in_folder(&self, folder_id: &str) -> Result<u32, AeroError> {
        let conn = self.connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mails WHERE folder_id = ?1",
            [folder_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Gets all UIDs for a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_uids_in_folder(&self, folder_id: &str) -> Result<Vec<u32>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT uid FROM mails WHERE folder_id = ?1")?;
        let rows = stmt.query_map([folder_id], |row| {
            let uid: i64 = row.get(0)?;
            Ok(uid as u32)
        })?;
        let mut uids = Vec::new();
        for row in rows {
            uids.push(row?);
        }
        Ok(uids)
    }

    /// Updates mail flags (is_read, is_starred) by UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn update_mail_flags_by_uid(
        &self,
        folder_id: &str,
        uid: u32,
        is_read: bool,
        is_starred: bool,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE mails SET is_read = ?1, is_starred = ?2 WHERE folder_id = ?3 AND uid = ?4",
            (is_read as i64, is_starred as i64, folder_id, uid),
        )?;
        Ok(())
    }

    /// Gets locally-read mails grouped by folder for syncing to IMAP.
    /// Returns `(folder_id, uid)` pairs where `is_read = 1` and `uid > 0`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_locally_read_mails(&self) -> Result<Vec<(String, u32)>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT folder_id, uid FROM mails WHERE is_read = 1 AND uid > 0",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Gets a mail ID by its folder and IMAP UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_mail_id_by_uid(
        &self,
        folder_id: &str,
        uid: u32,
    ) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt =
            conn.prepare("SELECT id FROM mails WHERE folder_id = ?1 AND uid = ?2 LIMIT 1")?;
        let mut rows = stmt.query(rusqlite::params![folder_id, uid as i64])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
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

    /// Counts unread mails for a specific folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn count_unread_in_folder(&self, folder_id: &str) -> Result<u32, AeroError> {
        let conn = self.connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mails WHERE folder_id = ?1 AND is_read = 0",
            [folder_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// 获取指定文件夹中最新的未读邮件信息，用于通知显示。
    /// 返回未读邮件摘要列表，按日期降序排列。
    ///
    /// # Errors
    ///
    /// 数据库查询失败或 limit 超出范围时返回错误。
    pub fn get_latest_unread_summaries(
        &self,
        folder_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::models::mail::UnreadMailSummary>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT from_name, subject FROM mails
             WHERE folder_id = ?1 AND is_read = 0
             ORDER BY date DESC LIMIT ?2",
        )?;
        let limit_i64 = i64::try_from(limit)
            .map_err(|_| AeroError::Internal(format!("limit {limit} is too large")))?;
        let rows = stmt.query_map(rusqlite::params![folder_id, limit_i64], |row| {
            Ok(crate::models::mail::UnreadMailSummary {
                from_name: row.get(0)?,
                subject: row.get(1)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Gets the account email address for the given account ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_account_email(&self, account_id: &str) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT email FROM accounts WHERE id = ?1")?;
        let mut rows = stmt.query([account_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Updates the encrypted OAuth2 / password credentials for an account.
    ///
    /// # Errors
    ///
    /// Returns an error if the account is not found or the write fails.
    pub fn update_account_auth_credentials(
        &self,
        account_id: &str,
        credentials: &[u8],
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let rows = conn.execute(
            "UPDATE accounts SET auth_credentials_encrypted = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![credentials, Utc::now().timestamp(), account_id],
        )?;
        if rows == 0 {
            return Err(AeroError::AccountNotFound(account_id.to_string()));
        }
        Ok(())
    }

    /// Deletes a mail by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_mail(&self, mail_id: &str) -> Result<(), AeroError> {
        self.with_transaction(|conn| {
            let (folder_id, is_read): (String, bool) = conn.query_row(
                "SELECT folder_id, is_read FROM mails WHERE id = ?1",
                [mail_id],
                |row| Ok((row.get(0)?, row.get::<_, i64>(1)? != 0)),
            )?;
            conn.execute("DELETE FROM mails WHERE id = ?1", [mail_id])?;
            if !is_read {
                conn.execute(
                    "UPDATE folders SET unread_count = MAX(0, unread_count - 1) WHERE id = ?1",
                    [&folder_id],
                )?;
            }
            Ok(())
        })
    }

    /// Moves a mail to a different folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn move_mail(&self, mail_id: &str, target_folder_id: &str) -> Result<(), AeroError> {
        self.with_transaction(|conn| {
            let (source_folder_id, is_read): (String, bool) = conn.query_row(
                "SELECT folder_id, is_read FROM mails WHERE id = ?1",
                [mail_id],
                |row| Ok((row.get(0)?, row.get::<_, i64>(1)? != 0)),
            )?;
            conn.execute(
                "UPDATE mails SET folder_id = ?1 WHERE id = ?2",
                (target_folder_id, mail_id),
            )?;
            if !is_read {
                conn.execute(
                    "UPDATE folders SET unread_count = MAX(0, unread_count - 1) WHERE id = ?1",
                    [&source_folder_id],
                )?;
                conn.execute(
                    "UPDATE folders SET unread_count = unread_count + 1 WHERE id = ?1",
                    [target_folder_id],
                )?;
            }
            Ok(())
        })
    }

    /// Deletes all attachment records for a mail.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_attachments(&self, mail_id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM attachments WHERE mail_id = ?1", [mail_id])?;
        drop(conn);
        Ok(())
    }

    /// Inserts a single attachment record.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn insert_attachment(
        &self,
        mail_id: &str,
        attachment: &crate::models::mail::ParsedAttachment,
        local_path: &std::path::Path,
    ) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let id = uuid::Uuid::new_v4().to_string();
        let filename = attachment
            .filename
            .clone()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| "unnamed".to_string());
        let size =
            i64::try_from(attachment.size).map_err(|e| AeroError::Internal(e.to_string()))?;
        conn.execute(
            "INSERT INTO attachments (id, mail_id, filename, mime_type, size, content_id, local_path, is_inline)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &id,
                mail_id,
                filename,
                &attachment.mime_type,
                size,
                &attachment.content_id,
                local_path.to_str(),
                attachment.is_inline as i64,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Inserts multiple attachment records in a single transaction.
    pub fn insert_attachments_batch(
        &self,
        attachments: &[(
            String,
            crate::models::mail::ParsedAttachment,
            std::path::PathBuf,
        )],
    ) -> Result<(), AeroError> {
        if attachments.is_empty() {
            return Ok(());
        }
        self.with_transaction(|conn| {
            let mut stmt = conn.prepare(
                "INSERT INTO attachments (id, mail_id, filename, mime_type, size, content_id, local_path, is_inline)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for (mail_id, attachment, local_path) in attachments {
                let id = uuid::Uuid::new_v4().to_string();
                let filename = attachment
                    .filename
                    .clone()
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| "unnamed".to_string());
                let size = i64::try_from(attachment.size).map_err(|e| AeroError::Internal(e.to_string()))?;
                stmt.execute(params![
                    &id,
                    mail_id,
                    filename,
                    &attachment.mime_type,
                    size,
                    &attachment.content_id,
                    local_path.to_str(),
                    attachment.is_inline as i64,
                ])?;
            }
            Ok(())
        })
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

    /// Gets the local filesystem path for an attachment by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_attachment_path(&self, attachment_id: &str) -> Result<Option<String>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT local_path FROM attachments WHERE id = ?1")?;
        let mut rows = stmt.query([attachment_id])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Ok(None)
        }
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
        let mut stmt = conn.prepare(
            "SELECT id, account_id, subject, to_addresses, saved_at, attachments_json
             FROM drafts WHERE (?1 IS NULL OR account_id = ?1)
             ORDER BY saved_at DESC",
        )?;
        let mut rows = stmt.query([account_id])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let to: Vec<String> =
                serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default();
            let attachments: Vec<crate::models::compose::AttachmentDraft> =
                serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default();
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
    pub fn get_draft_account_id(&self, draft_id: &str) -> Result<Option<String>, AeroError> {
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
    pub fn get_unindexed_mails(&self) -> Result<Vec<crate::models::mail::MailDetail>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, folder_id, uid, subject, from_name, from_address,
             to_addresses, cc_addresses, date, body_html, body_text, is_read, is_starred, is_archived, is_spam, flags, message_id
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
                is_archived: row.get::<_, i64>(14)? != 0,
                is_spam: row.get::<_, i64>(15)? != 0,
                flags: row.get(16)?,
                message_id: row.get(17)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    // ---- Todo CRUD ----

    /// Lists all todos ordered by creation time descending.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_todos(&self) -> Result<Vec<crate::models::todo::TodoItem>, AeroError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, text, done, mail_id, created_at, completed_at, reminder_at, notified_at, completion_log_json
             FROM todos ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let completion_log_json: Option<String> = row.get(8)?;
            let completion_log = completion_log_json
                .and_then(|s| serde_json::from_str::<Vec<i64>>(&s).ok())
                .unwrap_or_default();
            Ok(crate::models::todo::TodoItem {
                id: row.get(0)?,
                text: row.get(1)?,
                done: row.get::<_, i64>(2)? != 0,
                mail_id: row.get(3)?,
                created_at: row.get(4)?,
                completed_at: row.get(5)?,
                reminder_at: row.get(6)?,
                notified_at: row.get(7)?,
                completion_log,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AeroError::Database(e.to_string()))
    }

    /// Inserts or updates a todo item.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn upsert_todo(&self, todo: &crate::models::todo::TodoItem) -> Result<(), AeroError> {
        let conn = self.connection()?;
        let completion_log_json = serde_json::to_string(&todo.completion_log).unwrap_or_default();
        conn.execute(
            "INSERT INTO todos (id, text, done, mail_id, created_at, completed_at, reminder_at, notified_at, completion_log_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
               text = excluded.text,
               done = excluded.done,
               mail_id = excluded.mail_id,
               created_at = excluded.created_at,
               completed_at = excluded.completed_at,
               reminder_at = excluded.reminder_at,
               notified_at = excluded.notified_at,
               completion_log_json = excluded.completion_log_json",
            (
                &todo.id,
                &todo.text,
                todo.done as i64,
                &todo.mail_id,
                todo.created_at,
                todo.completed_at,
                todo.reminder_at,
                todo.notified_at,
                completion_log_json,
            ),
        )?;
        drop(conn);
        Ok(())
    }

    /// Deletes a todo by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn delete_todo(&self, id: &str) -> Result<(), AeroError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM todos WHERE id = ?1", [id])?;
        drop(conn);
        Ok(())
    }

    /// Deletes all completed todos and returns the number removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub fn clear_completed_todos(&self) -> Result<usize, AeroError> {
        let conn = self.connection()?;
        let removed = conn.execute("DELETE FROM todos WHERE done = 1", [])?;
        drop(conn);
        Ok(removed)
    }
}
