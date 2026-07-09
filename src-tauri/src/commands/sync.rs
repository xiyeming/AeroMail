#![allow(clippy::missing_errors_doc)]

use futures::StreamExt;
use tauri::{AppHandle, State};
use tracing::{info, instrument};

use crate::AppState;
use crate::models::error::ErrorPayload;
use crate::services::imap_client;

#[tauri::command]
#[instrument(skip(state, app_handle), fields(account_id = %account_id), err(Debug))]
pub async fn start_sync(
    account_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let sync = state.sync_service.read().await;
    sync.start_sync(&account_id, app_handle)
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(account_id = %account_id), err(Debug))]
pub async fn stop_sync(account_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let sync = state.sync_service.read().await;
    sync.stop_sync(&account_id)
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(folder_id = %folder_id, limit), err(Debug))]
pub async fn fetch_older_mails(
    folder_id: String,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<u32, ErrorPayload> {
    let sync = state.sync_service.read().await;
    let fetched = sync
        .fetch_older_mails(&folder_id, limit)
        .await
        .map_err(|e| e.to_payload())?;
    drop(sync);

    if fetched > 0 {
        let index_result = state.search_service.read().await.index_pending_mails();
        if let Err(e) = index_result {
            tracing::warn!(error = %e, "failed to index fetched older mails");
        }
    }

    Ok(fetched)
}

/// Pushes local read state to the IMAP server.
/// Fixes inconsistency where local DB has mails marked as read
/// but the IMAP server doesn't have the `\Seen` flag set.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn sync_read_flags_to_server(state: State<'_, AppState>) -> Result<u32, ErrorPayload> {
    let db = &state.db;
    let locally_read = db.get_locally_read_mails().map_err(|e| e.to_payload())?;

    if locally_read.is_empty() {
        info!("no locally read mails to sync");
        return Ok(0);
    }

    info!(count = locally_read.len(), "found locally read mails");

    // Group by folder_id
    let mut folder_uids: std::collections::HashMap<String, Vec<u32>> =
        std::collections::HashMap::new();
    for (folder_id, uid) in locally_read {
        folder_uids.entry(folder_id).or_default().push(uid);
    }

    let account_manager = state.account_manager.read().await;
    let mut synced_count = 0u32;

    for (folder_id, uids) in &folder_uids {
        info!(folder_id = %folder_id, uid_count = uids.len(), "processing folder");

        let folder = match db.get_folder_by_id(folder_id) {
            Ok(Some(f)) => f,
            _ => {
                tracing::warn!(folder_id = %folder_id, "folder not found in DB");
                continue;
            }
        };
        let config = match account_manager
            .get_account_config_with_refresh(&folder.account_id)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(folder_id = %folder_id, error = %e, "skipping folder");
                continue;
            }
        };

        let mut session = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            imap_client::connect_imap(&config),
        )
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                tracing::warn!(folder_id = %folder_id, error = %e, "failed to connect");
                continue;
            }
            Err(_) => {
                tracing::warn!(folder_id = %folder_id, "IMAP connect timeout");
                continue;
            }
        };

        if let Err(e) = session.select(&folder.path).await {
            tracing::warn!(error = %e, "failed to select folder");
            let _ = session.logout().await;
            continue;
        }

        // Batch UIDs into chunks to avoid overly long FETCH commands
        for chunk in uids.chunks(100) {
            let uid_set = chunk
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");

            // First fetch current flags to only update those not already \Seen
            let mut fetch_stream = match session
                .uid_fetch(&uid_set, "(UID FLAGS)")
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(error = %e, "failed to fetch flags");
                    continue;
                }
            };

            let mut needs_update = Vec::new();
            while let Some(fetch_res) = fetch_stream.next().await {
                let fetch = match fetch_res {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let uid = fetch.uid.unwrap_or(0);
                if uid == 0 {
                    continue;
                }
                if !imap_client::is_seen_flag(fetch.flags()) {
                    needs_update.push(uid);
                }
            }
            drop(fetch_stream);

            if needs_update.is_empty() {
                info!("all mails in chunk already have \\Seen flag");
                continue;
            }

            let update_set = needs_update
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");

            match session.uid_store(&update_set, "+FLAGS (\\Seen)").await {
                Ok(_) => {
                    info!(count = needs_update.len(), "set \\Seen flag on server");
                    synced_count += needs_update.len() as u32;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to set \\Seen flag");
                }
            }
        }

        let _ = session.logout().await;
    }

    info!(count = synced_count, "synced read flags to IMAP server");
    Ok(synced_count)
}
