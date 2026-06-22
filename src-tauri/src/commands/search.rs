#![allow(clippy::missing_errors_doc)]

use tauri::State;

use crate::AppState;
use crate::models::error::ErrorPayload;
use crate::models::search::{SearchQuery, SearchResult, SearchStats};

#[tauri::command]
pub async fn search_mails(
    query: SearchQuery,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, ErrorPayload> {
    let search = state.search_service.read().await;
    search.search(&query).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn index_pending_mails(state: State<'_, AppState>) -> Result<u64, ErrorPayload> {
    let search = state.search_service.read().await;
    search.index_pending_mails().map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_search_stats(state: State<'_, AppState>) -> Result<SearchStats, ErrorPayload> {
    let search = state.search_service.read().await;
    search.stats().map_err(|e| e.to_payload())
}
