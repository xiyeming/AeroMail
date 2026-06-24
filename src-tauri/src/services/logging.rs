use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::Registry;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::reload::{self, Handle};
use tracing_subscriber::util::SubscriberInitExt;

use crate::db::pool::Database;
use crate::error::AeroError;

const SETTING_ENABLED: &str = "app.log.enabled";
const SETTING_DIR: &str = "app.log.dir";

/// Runtime logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    pub enabled: bool,
    pub dir: PathBuf,
}

/// Manages the global tracing subscriber and allows runtime reconfiguration.
pub struct LogService {
    reload_handle: Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>,
    guard: Mutex<Option<WorkerGuard>>,
}

impl LogService {
    /// Initializes the global logger with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber cannot be installed.
    pub fn new(config: &LogConfig) -> Result<Self, AeroError> {
        let (layer, guard) = build_layer(config)?;
        let (reload_layer, handle) = reload::Layer::new(layer);

        tracing_subscriber::registry().with(reload_layer).init();

        tracing::info!(
            dir = %config.dir.display(),
            enabled = config.enabled,
            "AeroMail logging initialized"
        );

        Ok(Self {
            reload_handle: handle,
            guard: Mutex::new(Some(guard)),
        })
    }

    /// Creates a service by reading persisted settings, falling back to defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be queried or the logger cannot be initialized.
    pub fn from_settings(db: &Database, default_dir: &Path) -> Result<Self, AeroError> {
        let enabled = db
            .get_setting(SETTING_ENABLED)?
            .is_some_and(|v| v == "true");
        let dir = db
            .get_setting(SETTING_DIR)?
            .map_or_else(|| default_dir.to_path_buf(), PathBuf::from);
        let config = LogConfig { enabled, dir };
        Self::new(&config)
    }

    /// Applies a new configuration, persisting it and rebuilding the log layer.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write or layer reload fails.
    pub fn apply(&self, db: &Database, config: &LogConfig) -> Result<(), AeroError> {
        db.set_setting(SETTING_ENABLED, &config.enabled.to_string())?;
        db.set_setting(SETTING_DIR, config.dir.to_string_lossy().as_ref())?;

        let (layer, guard) = build_layer(config)?;
        self.reload_handle
            .reload(layer)
            .map_err(|e| AeroError::Internal(format!("failed to reload log layer: {e}")))?;

        let mut stored = self
            .guard
            .lock()
            .map_err(|e| AeroError::Internal(format!("log guard mutex poisoned: {e}")))?;
        *stored = Some(guard);
        drop(stored);

        tracing::info!(
            dir = %config.dir.display(),
            enabled = config.enabled,
            "Log configuration updated"
        );

        Ok(())
    }

    /// Reads the currently persisted configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be queried.
    pub fn get_config(db: &Database, default_dir: &Path) -> Result<LogConfig, AeroError> {
        let enabled = db
            .get_setting(SETTING_ENABLED)?
            .is_some_and(|v| v == "true");
        let dir = db
            .get_setting(SETTING_DIR)?
            .map_or_else(|| default_dir.to_path_buf(), PathBuf::from);
        Ok(LogConfig { enabled, dir })
    }
}

fn build_layer(
    config: &LogConfig,
) -> Result<(Box<dyn Layer<Registry> + Send + Sync>, WorkerGuard), AeroError> {
    let (writer, guard) = if config.enabled {
        let appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("aeromail")
            .filename_suffix("log")
            .max_log_files(7)
            .build(&config.dir)
            .map_err(|e| AeroError::Internal(format!("failed to create log appender: {e}")))?;
        tracing_appender::non_blocking(appender)
    } else {
        tracing_appender::non_blocking(std::io::sink())
    };

    let layer = fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339());

    Ok((Box::new(layer), guard))
}
