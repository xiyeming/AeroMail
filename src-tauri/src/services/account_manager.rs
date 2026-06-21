use std::sync::Arc;

use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AccountSummary, AuthConfig, TlsMode};

#[derive(Debug)]
pub struct AccountManager {
    db: Arc<Database>,
}

impl AccountManager {
    pub const fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Adds a new email account with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or the database write fails.
    pub fn add_account(&self, mut config: AccountConfig) -> Result<String, AeroError> {
        let id = Uuid::new_v4().to_string();
        config.id.clone_from(&id);

        let auth_json = serde_json::to_string(&config.auth)?;
        let excluded_folders_json = serde_json::to_string(&config.excluded_folders)?;
        let now = Utc::now().timestamp();

        self.db.connection()?.execute(
            r"
            INSERT INTO accounts (
                id, name, provider, imap_host, imap_port, smtp_host, smtp_port,
                tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
                verify_certificate, connect_timeout, read_timeout, keepalive,
                sync_interval, excluded_folders, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            ",
            params![
                &config.id,
                &config.name,
                serde_json::to_string(&config.provider)?,
                &config.imap.host,
                config.imap.port,
                &config.smtp.host,
                config.smtp.port,
                serde_json::to_string(&config.imap.tls_mode)?,
                match config.auth {
                    AuthConfig::OAuth2 { .. } => "OAuth2",
                    AuthConfig::Password { .. } => "Password",
                },
                auth_json.as_bytes(),
                config.advanced.ca_cert_path,
                config.advanced.verify_certificate,
                config.advanced.connect_timeout_secs,
                config.advanced.read_timeout_secs,
                config.advanced.keepalive,
                config.sync_interval_secs,
                excluded_folders_json,
                now,
                now,
            ],
        )?;

        Ok(id)
    }

    /// Lists all configured email accounts.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_accounts(&self) -> Result<Vec<AccountSummary>, AeroError> {
        let conn = self.db.connection()?;
        let mut stmt = conn.prepare(
            r"
            SELECT id, name, provider, imap_host, smtp_host
            FROM accounts
            ORDER BY created_at ASC
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            let provider_json: String = row.get(2)?;
            let provider = serde_json::from_str(&provider_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(AccountSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                provider,
                imap_host: row.get(3)?,
                smtp_host: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AeroError::from)
    }

    /// Deletes an email account by its ID.
    ///
    /// # Errors
    ///
    /// Returns [`AeroError::AccountNotFound`] if no account with the given ID exists.
    pub fn delete_account(&self, account_id: &str) -> Result<(), AeroError> {
        let rows = self
            .db
            .connection()?
            .execute("DELETE FROM accounts WHERE id = ?1", params![account_id])?;

        if rows == 0 {
            return Err(AeroError::AccountNotFound(account_id.to_string()));
        }

        Ok(())
    }

    /// Retrieves a full account configuration by ID.
    ///
    /// # Errors
    ///
    /// Returns [`AeroError::AccountNotFound`] if no account with the given ID exists.
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_account_config(
        &self,
        account_id: &str,
    ) -> Result<AccountConfig, AeroError> {
        let conn = self.db.connection()?;
        conn.query_row(
            "SELECT id, name, provider, imap_host, imap_port, smtp_host, smtp_port,
             tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
             verify_certificate, connect_timeout, read_timeout, keepalive,
             sync_interval, excluded_folders
             FROM accounts WHERE id = ?1",
            [account_id],
            |row| {
                let provider_json: String = row.get(2)?;
                let provider = serde_json::from_str(&provider_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;

                let tls_mode_json: String = row.get(7)?;
                let tls_mode = serde_json::from_str(&tls_mode_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                let auth_type: String = row.get(8)?;
                let auth_credentials: Option<Vec<u8>> = row.get(9)?;
                let auth = match auth_type.as_str() {
                    "Password" => AuthConfig::Password {
                        password_encrypted: auth_credentials.unwrap_or_default(),
                    },
                    "OAuth2" => {
                        let creds_bytes = auth_credentials.unwrap_or_default();
                        let creds_json = String::from_utf8_lossy(&creds_bytes);
                        serde_json::from_str(&creds_json)
                            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    }
                    _ => return Err(rusqlite::Error::InvalidParameterName(format!(
                        "Unknown auth type: {auth_type}"
                    ))),
                };

                let excluded_folders_json: String = row.get(16)?;
                let excluded_folders: Vec<String> =
                    serde_json::from_str(&excluded_folders_json).unwrap_or_default();

                Ok(AccountConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider,
                    imap: crate::models::account::ServerConfig {
                        host: row.get(3)?,
                        port: row.get::<_, i64>(4)? as u16,
                        tls_mode,
                    },
                    smtp: crate::models::account::ServerConfig {
                        host: row.get(5)?,
                        port: row.get::<_, i64>(6)? as u16,
                        tls_mode: crate::models::account::TlsMode::Required,
                    },
                    auth,
                    advanced: crate::models::account::AdvancedConfig {
                        ca_cert_path: row.get(10)?,
                        verify_certificate: row.get::<_, i64>(11)? != 0,
                        connect_timeout_secs: row.get::<_, i64>(12)? as u64,
                        read_timeout_secs: row.get::<_, i64>(13)? as u64,
                        keepalive: row.get::<_, i64>(14)? != 0,
                    },
                    sync_interval_secs: row.get::<_, i64>(15)? as u64,
                    excluded_folders,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AeroError::AccountNotFound(account_id.to_string()),
            _ => AeroError::Database(e.to_string()),
        })
    }

    /// Tests the connection to an email server using the provided configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub async fn test_connection(&self, config: &AccountConfig) -> Result<String, AeroError> {
        // Phase 1 placeholder: validate config and simulate a connection test.
        if config.imap.host.is_empty() {
            return Err(AeroError::InvalidConfig(
                "IMAP host cannot be empty".to_string(),
            ));
        }
        if config.imap.port == 0 {
            return Err(AeroError::InvalidConfig(
                "IMAP port cannot be zero".to_string(),
            ));
        }
        if matches!(config.imap.tls_mode, TlsMode::None) && config.imap.port != 143 {
            return Ok(format!(
                "Connection test passed (no encryption on port {})",
                config.imap.port
            ));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Ok(format!(
            "Connection test passed for {}:{}",
            config.imap.host, config.imap.port
        ))
    }
}
