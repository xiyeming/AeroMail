#![allow(clippy::missing_errors_doc)]

use tauri::State;
use tracing::instrument;

use crate::AppState;
use crate::models::error::ErrorPayload;
use crate::models::mail::MailHeader;
use crate::models::search::{SearchQuery, SearchResult, SearchStats};

#[tauri::command]
#[instrument(skip(state, query), fields(query = %query.query, folder_id = ?query.folder_id, account_id = ?query.account_id), err(Debug))]
pub async fn search_mails(
    query: SearchQuery,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, ErrorPayload> {
    let search = state.search_service.read().await;
    search.search(&query).map_err(|e| e.to_payload())
}

/// Searches mails by subject/body and returns lightweight headers for
/// autocomplete use-cases such as @-mentions in the AI assistant.
///
/// # Errors
///
/// Returns an error if the search query or database lookup fails.
#[tauri::command]
#[instrument(skip(state), fields(query = %query), err(Debug))]
pub async fn search_mail_summaries(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<MailHeader>, ErrorPayload> {
    let search = state.search_service.read().await;
    let trimmed = query.trim().to_string();
    let results = if trimmed.is_empty() {
        Vec::new()
    } else {
        search
            .search(&SearchQuery {
                query,
                folder_id: None,
                account_id: None,
                date_from: None,
                date_to: None,
                has_attachment: None,
                is_read: None,
            })
            .map_err(|e| e.to_payload())?
    };
    drop(search);

    let mut headers = Vec::with_capacity(results.len().min(10));
    for result in results.iter().take(10) {
        if let Ok(Some(header)) = state.db.get_mail_header(&result.mail_id) {
            headers.push(header);
        }
    }

    if headers.is_empty() {
        if trimmed.is_empty() {
            if let Ok(recent) = state.db.list_recent_mails(10) {
                headers = recent;
            }
        } else if let Ok(found) = state.db.search_mail_headers(&trimmed, 10) {
            headers = found;
        }
    }

    Ok(headers)
}

#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn index_pending_mails(state: State<'_, AppState>) -> Result<u64, ErrorPayload> {
    let search = state.search_service.read().await;
    search.index_pending_mails().map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn get_search_stats(state: State<'_, AppState>) -> Result<SearchStats, ErrorPayload> {
    let search = state.search_service.read().await;
    search.stats().map_err(|e| e.to_payload())
}
