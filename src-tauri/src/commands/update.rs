use tauri_plugin_updater::UpdaterExt;

/// Check for updates from GitHub releases
#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update {
        Ok(Some(UpdateInfo {
            version: update.version,
            notes: update.body.unwrap_or_default(),
            pub_date: update.date.map(|d| d.to_string()).unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

/// Download and install the update
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        // Download and install with progress callback
        update
            .download_and_install(
                |chunk_length, content_length| {
                    tracing::debug!(
                        "downloaded {} of {:?}",
                        chunk_length,
                        content_length
                    );
                },
                || {
                    tracing::debug!("download finished");
                },
            )
            .await
            .map_err(|e| e.to_string())?;

        // Restart the app
        app.restart();
    }
    Ok(())
}

/// Get current app version
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub pub_date: String,
}
