use std::sync::Arc;

use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AccountSummary, AuthConfig};
use crate::services::crypto;
use tracing::{debug, info, instrument};

#[derive(Debug)]
pub struct AccountManager {
    db: Arc<Database>,
}

impl AccountManager {
    pub const fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Returns a reference to the underlying database.
    #[must_use]
    pub fn db(&self) -> &Database {
        self.db.as_ref()
    }

    /// Adds a new email account with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or the database write fails.
    #[instrument(skip_all, fields(account_id = tracing::field::Empty, account_name = %config.name, email = ?config.email), err(Debug))]
    pub fn add_account(&self, mut config: AccountConfig) -> Result<String, AeroError> {
        let id = Uuid::new_v4().to_string();
        config.id.clone_from(&id);
        tracing::Span::current().record("account_id", &id);

        debug!("persisting new account to database");

        let (auth_type, auth_credentials) = match &config.auth {
            AuthConfig::Password { password_encrypted } => {
                let encrypted = crypto::encrypt_password(password_encrypted)?;
                ("Password", encrypted)
            }
            AuthConfig::OAuth2 { .. } => {
                let json = serde_json::to_string(&config.auth)?;
                ("OAuth2", json.into_bytes())
            }
        };

        let excluded_folders_json = serde_json::to_string(&config.excluded_folders)?;
        let now = Utc::now().timestamp();

        self.db.connection()?.execute(
            r"
            INSERT INTO accounts (
                id, name, email, provider, imap_host, imap_port, smtp_host, smtp_port,
                tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
                verify_certificate, connect_timeout, read_timeout, keepalive,
                sync_interval, excluded_folders, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
            ",
            params![
                &config.id,
                &config.name,
                &config.email,
                serde_json::to_string(&config.provider)?,
                &config.imap.host,
                config.imap.port,
                &config.smtp.host,
                config.smtp.port,
                serde_json::to_string(&config.imap.tls_mode)?,
                auth_type,
                auth_credentials,
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

        info!(account_id = %id, "account added");
        Ok(id)
    }

    /// Lists all configured email accounts.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    #[allow(clippy::significant_drop_tightening)]
    #[instrument(skip_all, err(Debug))]
    pub fn list_accounts(&self) -> Result<Vec<AccountSummary>, AeroError> {
        let conn = self.db.connection()?;
        let mut stmt = conn.prepare(
            r"
            SELECT id, name, email, provider, imap_host, smtp_host
            FROM accounts
            ORDER BY created_at ASC
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            let provider_json: String = row.get(3)?;
            let provider = serde_json::from_str(&provider_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(AccountSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                provider,
                imap_host: row.get(4)?,
                smtp_host: row.get(5)?,
            })
        })?;

        let accounts = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AeroError::from)?;
        info!(count = accounts.len(), "listed accounts");
        Ok(accounts)
    }

    /// Deletes an email account by its ID.
    ///
    /// # Errors
    ///
    /// Returns [`AeroError::AccountNotFound`] if no account with the given ID exists.
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub fn delete_account(&self, account_id: &str) -> Result<(), AeroError> {
        let rows = self
            .db
            .connection()?
            .execute("DELETE FROM accounts WHERE id = ?1", params![account_id])?;

        if rows == 0 {
            return Err(AeroError::AccountNotFound(account_id.to_string()));
        }

        info!(account_id = %account_id, "deleted account");
        Ok(())
    }

    /// Retrieves a full account configuration by ID.
    ///
    /// # Errors
    ///
    /// Returns [`AeroError::AccountNotFound`] if no account with the given ID exists.
    #[allow(
        clippy::cast_lossless,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::significant_drop_tightening
    )]
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub fn get_account_config(&self, account_id: &str) -> Result<AccountConfig, AeroError> {
        debug!("loading account configuration");
        let conn = self.db.connection()?;
        conn.query_row(
            "SELECT id, name, email, provider, imap_host, imap_port, smtp_host, smtp_port,
             tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
             verify_certificate, connect_timeout, read_timeout, keepalive,
             sync_interval, excluded_folders
             FROM accounts WHERE id = ?1",
            [account_id],
            |row| {
                let name: String = row.get(1)?;
                let email: Option<String> = row.get(2)?;

                let provider_json: String = row.get(3)?;
                let provider = serde_json::from_str(&provider_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let tls_mode_json: String = row.get(8)?;
                let tls_mode = serde_json::from_str(&tls_mode_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                let auth_type: String = row.get(9)?;
                let auth_credentials: Option<Vec<u8>> = row.get(10)?;
                let auth = match auth_type.as_str() {
                    "Password" => {
                        let encrypted = auth_credentials.unwrap_or_default();
                        let plaintext = crypto::decrypt_password(&encrypted)
                            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                        AuthConfig::Password {
                            password_encrypted: plaintext,
                        }
                    }
                    "OAuth2" => {
                        let creds_bytes = auth_credentials.unwrap_or_default();
                        let creds_json = String::from_utf8_lossy(&creds_bytes);
                        serde_json::from_str(&creds_json)
                            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    }
                    _ => {
                        return Err(rusqlite::Error::InvalidParameterName(format!(
                            "Unknown auth type: {auth_type}"
                        )));
                    }
                };

                let excluded_folders_json: String = row.get(17)?;
                let excluded_folders: Vec<String> =
                    serde_json::from_str(&excluded_folders_json).unwrap_or_default();

                Ok(AccountConfig {
                    id: row.get(0)?,
                    name,
                    email,
                    provider,
                    imap: crate::models::account::ServerConfig {
                        host: row.get(4)?,
                        port: row.get::<_, i64>(5)? as u16,
                        tls_mode,
                    },
                    smtp: crate::models::account::ServerConfig {
                        host: row.get(6)?,
                        port: row.get::<_, i64>(7)? as u16,
                        tls_mode: crate::models::account::TlsMode::Required,
                    },
                    auth,
                    advanced: crate::models::account::AdvancedConfig {
                        ca_cert_path: row.get(11)?,
                        verify_certificate: row.get::<_, i64>(12)? != 0,
                        connect_timeout_secs: row.get::<_, i64>(13)? as u64,
                        read_timeout_secs: row.get::<_, i64>(14)? as u64,
                        keepalive: row.get::<_, i64>(15)? != 0,
                    },
                    sync_interval_secs: row.get::<_, i64>(16)? as u64,
                    excluded_folders,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AeroError::AccountNotFound(account_id.to_string())
            }
            _ => AeroError::Database(e.to_string()),
        })
    }

    /// Retrieves a full account configuration by ID and refreshes an `OAuth2`
    /// access token if it is expired.
    ///
    /// # Errors
    ///
    /// Returns [`AeroError::AccountNotFound`] if no account with the given ID exists.
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub async fn get_account_config_with_refresh(
        &self,
        account_id: &str,
    ) -> Result<AccountConfig, AeroError> {
        debug!("refreshing OAuth2 token if needed");
        let mut config = self.get_account_config(account_id)?;
        crate::services::oauth2::ensure_access_token(
            Some(account_id),
            &mut config,
            Some(self.db.as_ref()),
        )
        .await?;
        Ok(config)
    }

    /// Tests the connection to an email server using the provided configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or the connection fails.
    #[instrument(skip_all, fields(host = %config.imap.host, port = config.imap.port), err(Debug))]
    pub async fn test_connection(&self, config: &AccountConfig) -> Result<String, AeroError> {
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

        debug!("starting account connection test");
        let mut config = config.clone();
        crate::services::oauth2::ensure_access_token(None, &mut config, None).await?;

        let mut session = crate::services::imap_client::connect_imap(&config).await?;
        session
            .logout()
            .await
            .map_err(|e| AeroError::ConnectionTestFailed(e.to_string()))?;

        info!(host = %config.imap.host, port = config.imap.port, "connection test passed");
        Ok(format!(
            "Connection test passed for {}:{}",
            config.imap.host, config.imap.port
        ))
    }
}
