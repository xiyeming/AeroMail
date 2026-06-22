use tauri::State;

use crate::AppState;
use crate::models::ai::{AiChatMessage, AiChatSession, AiProvider, AiProviderSummary};
use crate::models::error::ErrorPayload;
use crate::services::ai::ChatMessage;

/// Lists all configured AI providers as summaries.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
pub async fn list_ai_providers(
    state: State<'_, AppState>,
) -> Result<Vec<AiProviderSummary>, ErrorPayload> {
    let ai = state.ai_service.read().await;
    ai.list_providers().map_err(|e| e.to_payload())
}

/// Creates or updates an AI provider in the database.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
pub async fn upsert_ai_provider(
    provider: AiProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let db = &state.db;
    db.upsert_ai_provider(&provider)
        .map_err(|e| e.to_payload())?;
    Ok(provider.id)
}

/// Deletes an AI provider by ID.
///
/// # Errors
///
/// Returns an error if the provider is not found or the database write fails.
#[tauri::command]
pub async fn delete_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .delete_ai_provider(&provider_id)
        .map_err(|e| e.to_payload())
}

/// Sends a test prompt to the specified AI provider to verify connectivity.
///
/// # Errors
///
/// Returns an error if the provider is not found or the API request fails.
#[tauri::command]
pub async fn test_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let ai = state.ai_service.read().await;
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say hello in one sentence.".to_string(),
    }];
    ai.complete(&provider_id, messages)
        .await
        .map_err(|e| e.to_payload())
}

/// Creates a new AI chat session.
///
/// # Errors
///
/// Returns an error if the provider is not found or the database write fails.
#[tauri::command]
pub async fn create_chat_session(
    provider_id: String,
    context_mail_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiChatSession, ErrorPayload> {
    let provider = state
        .db
        .get_ai_provider(&provider_id)
        .map_err(|e| e.to_payload())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let session = AiChatSession {
        id,
        title: None,
        provider_id,
        model: provider.model,
        context_mail_id,
        created_at: now,
        updated_at: now,
    };
    state
        .db
        .upsert_chat_session(&session)
        .map_err(|e| e.to_payload())?;
    Ok(session)
}

/// Sends a user message in a chat session and returns the user message record.
///
/// # Errors
///
/// Returns an error if the session is not found, the database operations fail,
/// or the AI API request fails.
#[tauri::command]
pub async fn send_chat_message(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<AiChatMessage, ErrorPayload> {
    let ai = state.ai_service.read().await;
    crate::services::ai::chat::send_message(&ai, &session_id, &content)
        .await
        .map_err(|e| e.to_payload())
}

/// Lists all chat sessions ordered by most recently updated.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
pub async fn list_chat_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<AiChatSession>, ErrorPayload> {
    state.db.list_chat_sessions().map_err(|e| e.to_payload())
}

/// Retrieves all messages for a chat session.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
pub async fn get_chat_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AiChatMessage>, ErrorPayload> {
    state
        .db
        .get_chat_messages(&session_id)
        .map_err(|e| e.to_payload())
}

/// Deletes a chat session and its messages.
///
/// # Errors
///
/// Returns an error if the session is not found or the database write fails.
#[tauri::command]
pub async fn delete_chat_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .delete_chat_session(&session_id)
        .map_err(|e| e.to_payload())
}
