mod mail_parser;
mod worker;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tauri::Emitter;
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::mail::SyncProgress;

use self::worker::SyncWorker;

/// Manages background sync tasks for all email accounts.
pub struct SyncService {
    db: Arc<Database>,
    /// Directory where attachment files are stored.
    attachments_dir: PathBuf,
    /// Map of `account_id` -> (`JoinHandle`, progress receiver)
    workers: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl SyncService {
    /// Creates a new `SyncService`.
    #[must_use]
    pub fn new(db: Arc<Database>, attachments_dir: PathBuf) -> Self {
        Self {
            db,
            attachments_dir,
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Starts background sync for a specific account.
    ///
    /// # Errors
    ///
    /// Returns an error if the account is not found or sync cannot be started.
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub async fn start_sync(
        &self,
        account_id: &str,
        app_handle: tauri::AppHandle,
    ) -> Result<(), AeroError> {
        // Check if already running
        {
            let workers = self.workers.read().await;
            if workers.contains_key(account_id) {
                info!("Sync already running for account {}", account_id);
                return Ok(());
            }
        }

        // Load account config from DB
        let account_config = self.load_account_config(account_id)?;

        // Create progress channel
        let (progress_tx, mut progress_rx) = mpsc::channel::<SyncProgress>(100);

        // Spawn progress forwarder task
        let handle_clone = app_handle.clone();
        let _account_id_clone = account_id.to_string();
        tokio::spawn(async move {
            while let Some(progress) = progress_rx.recv().await {
                let _ = handle_clone.emit("sync:progress", &progress);
            }
        });

        // Create and start worker
        let worker = SyncWorker {
            account_id: account_id.to_string(),
            account_config: Arc::new(RwLock::new(account_config)),
            db: Arc::clone(&self.db),
            progress_tx,
            attachments_dir: self.attachments_dir.clone(),
        };

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        self.workers
            .write()
            .await
            .insert(account_id.to_string(), handle);

        info!("Started sync for account {}", account_id);
        Ok(())
    }

    /// Stops background sync for a specific account.
    ///
    /// # Errors
    ///
    /// Returns an error if the stop operation fails.
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    pub async fn stop_sync(&self, account_id: &str) -> Result<(), AeroError> {
        let mut workers = self.workers.write().await;
        if let Some(handle) = workers.remove(account_id) {
            handle.abort();
            info!("Stopped sync for account {}", account_id);
        }
        drop(workers);
        Ok(())
    }

    /// Starts sync for all configured accounts.
    ///
    /// # Errors
    ///
    /// Returns an error if any account's sync cannot be started.
    #[allow(clippy::significant_drop_tightening)]
    #[instrument(skip_all, err(Debug))]
    pub async fn start_all(&self, app_handle: tauri::AppHandle) -> Result<(), AeroError> {
        let accounts = {
            let conn = self.db.connection()?;
            let mut stmt = conn.prepare("SELECT id FROM accounts")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AeroError::Database(e.to_string()))?
        };

        info!(
            account_count = accounts.len(),
            "starting sync for all accounts"
        );

        for account_id in accounts {
            if let Err(e) = self.start_sync(&account_id, app_handle.clone()).await {
                tracing::warn!("Failed to start sync for account {}: {}", account_id, e);
            }
        }

        Ok(())
    }

    /// Stops all running sync tasks.
    #[instrument(skip_all)]
    pub async fn stop_all(&self) {
        let mut workers = self.workers.write().await;
        for (_, handle) in workers.drain() {
            handle.abort();
        }
        drop(workers);
        info!("Stopped all sync tasks");
    }

    /// Loads an account configuration from the database.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    #[instrument(skip_all, fields(account_id = %account_id), err(Debug))]
    fn load_account_config(&self, account_id: &str) -> Result<AccountConfig, AeroError> {
        debug!("loading account config for sync worker");
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
                let provider = serde_json::from_str(&provider_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                let tls_mode_json: String = row.get(8)?;
                let tls_mode = serde_json::from_str(&tls_mode_json)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                let auth_type: String = row.get(9)?;
                let auth_credentials: Option<Vec<u8>> = row.get(10)?;

                let auth = match auth_type.as_str() {
                    "Password" => {
                        let encrypted = auth_credentials.unwrap_or_default();
                        let plaintext = crate::services::crypto::decrypt_password(&encrypted)
                            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                        crate::models::account::AuthConfig::Password {
                            password_encrypted: plaintext,
                        }
                    }
                    "OAuth2" => {
                        // Parse OAuth2 credentials from JSON
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
                        tls_mode: crate::models::account::TlsMode::Required, // Default
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
        .map_err(|e| AeroError::Database(e.to_string()))
    }
}
