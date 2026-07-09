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

/// Syncs mail flags from IMAP server to local database.
/// Pulls server state (read/starred) and updates local, ensuring
/// the remote is the source of truth.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn sync_read_flags_to_server(state: State<'_, AppState>) -> Result<u32, ErrorPayload> {
    info!("sync_mail_flags_from_server: START");

    let db = &state.db;
    let account_manager = state.account_manager.read().await;
    let accounts = account_manager.list_accounts().map_err(|e| e.to_payload())?;

    if accounts.is_empty() {
        info!("no accounts configured");
        return Ok(0);
    }

    let mut total_updated = 0u32;

    for account in &accounts {
        info!(account_id = %account.id, name = %account.name, "processing account");

        let config = match account_manager
            .get_account_config_with_refresh(&account.id)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "failed to get config, skipping account");
                continue;
            }
        };

        let folders = match db.list_folders(&account.id) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(error = %e, "failed to list folders");
                continue;
            }
        };

        info!(count = folders.len(), "found folders");

        for folder in &folders {
            let local_uids = match db.get_uids_in_folder(&folder.id) {
                Ok(uids) => uids,
                Err(_) => continue,
            };

            if local_uids.is_empty() {
                continue;
            }

            info!(folder = %folder.path, uid_count = local_uids.len(), "syncing flags");

            let mut session = match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                imap_client::connect_imap(&config),
            )
            .await
            {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "IMAP connect failed");
                    continue;
                }
                Err(_) => {
                    tracing::warn!("IMAP connect timeout");
                    continue;
                }
            };

            if let Err(e) = session.select(&folder.path).await {
                tracing::warn!(error = %e, "failed to select folder");
                let _ = session.logout().await;
                continue;
            }

            // Batch UIDs into chunks
            for chunk in local_uids.chunks(500) {
                let uid_set = chunk
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");

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

                let mut batch_updated = 0u32;
                while let Some(fetch_res) = fetch_stream.next().await {
                    let fetch = match fetch_res {
                        Ok(f) => f,
                        Err(_) => continue,
                    };
                    let uid = fetch.uid.unwrap_or(0);
                    if uid == 0 {
                        continue;
                    }

                    let is_seen = imap_client::is_seen_flag(fetch.flags());
                    let is_flagged = imap_client::is_flagged_flag(fetch.flags());

                    if db.update_mail_flags_by_uid(&folder.id, uid, is_seen, is_flagged).is_ok() {
                        batch_updated += 1;
                    }
                }
                drop(fetch_stream);

                if batch_updated > 0 {
                    info!(folder = %folder.path, updated = batch_updated, "flags updated");
                }
                total_updated += batch_updated;
            }

            // Update folder unread count after flag sync
            if let Ok(unread) = db.count_unread_in_folder(&folder.id) {
                let _ = db.update_folder_sync(
                    &folder.id,
                    folder.uid_validity.unwrap_or(0),
                    i64::from(unread),
                    folder.total_count,
                );
            }

            let _ = session.logout().await;
        }
    }

    info!(total = total_updated, "sync_mail_flags_from_server: DONE");
    Ok(total_updated)
}
