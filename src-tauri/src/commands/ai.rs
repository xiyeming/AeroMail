use tauri::State;
use tracing::instrument;

use crate::AppState;
use crate::error::AeroError;
use crate::models::ai::{
    AiChatMessage, AiChatSession, AiMcpServer, AiProvider, AiProviderPricing, AiProviderSummary,
    AiSkill, AiUsageSummary,
};
use crate::models::error::ErrorPayload;
use crate::services::ai::ChatMessage;

/// Lists all configured AI providers as summaries.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
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
#[instrument(skip(state, provider), fields(provider_id = %provider.id), err(Debug))]
pub async fn upsert_ai_provider(
    mut provider: AiProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    crate::services::ai::providers::apply_preset_defaults(&mut provider);
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
#[instrument(skip(state), fields(provider_id = %provider_id), err(Debug))]
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
#[instrument(skip(state), fields(provider_id = %provider_id), err(Debug))]
pub async fn test_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let ai = state.ai_service.read().await;
    let messages = vec![ChatMessage::new("user", "Say hello in one sentence.")];
    ai.complete(&provider_id, &messages, None)
        .await
        .map(|result| result.content)
        .map_err(|e| e.to_payload())
}

/// Creates a new AI chat session.
///
/// # Errors
///
/// Returns an error if the provider is not found or the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(provider_id = %provider_id, context_mail_id = ?context_mail_id), err(Debug))]
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

/// Quotes an email into an existing chat session.
///
/// # Errors
///
/// Returns an error if the session or mail is not found, or the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(session_id = %session_id, mail_id = %mail_id), err(Debug))]
pub async fn quote_mail_to_chat(
    session_id: String,
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<AiChatMessage, ErrorPayload> {
    crate::services::ai::chat::quote_mail_to_chat(&state.db, &session_id, &mail_id)
        .map_err(|e| e.to_payload())
}

/// Sends a user message in a chat session and returns the user message record.
///
/// # Errors
///
/// Returns an error if the session is not found, the database operations fail,
/// or the AI API request fails.
#[tauri::command]
#[instrument(skip(state, content), fields(session_id = %session_id, content_len = content.len()), err(Debug))]
pub async fn send_chat_message(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<AiChatMessage, ErrorPayload> {
    let ai = state.ai_service.read().await;
    let tool_registry = state.tool_registry.read().await;
    crate::services::ai::chat::send_message(&ai, &session_id, &content, &tool_registry)
        .await
        .map_err(|e| e.to_payload())
}

/// Lists all chat sessions ordered by most recently updated.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
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
#[instrument(skip(state), fields(session_id = %session_id), err(Debug))]
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
#[instrument(skip(state), fields(session_id = %session_id), err(Debug))]
pub async fn delete_chat_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .delete_chat_session(&session_id)
        .map_err(|e| e.to_payload())
}

/// Clears all messages in a chat session while keeping the session.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(session_id = %session_id), err(Debug))]
pub async fn clear_chat_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .clear_chat_messages(&session_id)
        .map_err(|e| e.to_payload())
}

/// Renames a chat session.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(session_id = %session_id), err(Debug))]
pub async fn rename_chat_session(
    session_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .rename_chat_session(&session_id, &title)
        .map_err(|e| e.to_payload())
}

/// Switches the AI provider/model used by a chat session.
///
/// # Errors
///
/// Returns an error if the provider is not found or the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(session_id = %session_id, provider_id = %provider_id), err(Debug))]
pub async fn set_chat_session_provider(
    session_id: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let provider = state
        .db
        .get_ai_provider(&provider_id)
        .map_err(|e| e.to_payload())?;
    state
        .db
        .set_chat_session_provider(&session_id, &provider_id, &provider.model)
        .map_err(|e| e.to_payload())?;
    Ok(())
}

/// Returns aggregate AI usage, optionally filtered by provider.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn get_ai_usage_summary(
    provider_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<AiUsageSummary>, ErrorPayload> {
    state
        .db
        .get_ai_usage_summary(provider_id.as_deref())
        .map_err(|e| e.to_payload())
}

/// Returns aggregate AI usage for a single chat session.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), fields(session_id = %session_id), err(Debug))]
pub async fn get_ai_session_usage(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<AiUsageSummary>, ErrorPayload> {
    state
        .db
        .get_ai_session_usage(&session_id)
        .map_err(|e| e.to_payload())
}

/// Creates or updates pricing for a provider/model combination.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state, pricing), err(Debug))]
pub async fn upsert_ai_provider_pricing(
    pricing: AiProviderPricing,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .upsert_ai_provider_pricing(&pricing)
        .map_err(|e| e.to_payload())
}

/// Lists all configured AI provider pricing records.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn list_ai_provider_pricing(
    state: State<'_, AppState>,
) -> Result<Vec<AiProviderPricing>, ErrorPayload> {
    state
        .db
        .list_ai_provider_pricing()
        .map_err(|e| e.to_payload())
}

/// Lists all configured MCP servers.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn list_ai_mcp_servers(
    state: State<'_, AppState>,
) -> Result<Vec<AiMcpServer>, ErrorPayload> {
    state.db.list_ai_mcp_servers().map_err(|e| e.to_payload())
}

/// Creates or updates an MCP server configuration.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state, server), err(Debug))]
pub async fn upsert_ai_mcp_server(
    server: AiMcpServer,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .upsert_ai_mcp_server(&server)
        .map_err(|e| e.to_payload())?;
    refresh_tool_registry(&state).await;
    Ok(())
}

/// Deletes an MCP server configuration.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(server_id = %server_id), err(Debug))]
pub async fn delete_ai_mcp_server(
    server_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .delete_ai_mcp_server(&server_id)
        .map_err(|e| e.to_payload())?;
    refresh_tool_registry(&state).await;
    Ok(())
}

/// Lists all configured skills.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn list_ai_skills(state: State<'_, AppState>) -> Result<Vec<AiSkill>, ErrorPayload> {
    state.db.list_ai_skills().map_err(|e| e.to_payload())
}

/// Creates or updates a skill configuration.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state, skill), err(Debug))]
pub async fn upsert_ai_skill(
    skill: AiSkill,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .upsert_ai_skill(&skill)
        .map_err(|e| e.to_payload())?;
    refresh_tool_registry(&state).await;
    Ok(())
}

/// Deletes a skill configuration.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(skill_id = %skill_id), err(Debug))]
pub async fn delete_ai_skill(
    skill_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .delete_ai_skill(&skill_id)
        .map_err(|e| e.to_payload())?;
    refresh_tool_registry(&state).await;
    Ok(())
}

/// Summarizes a mail using the specified AI provider.
///
/// # Errors
///
/// Returns an error if the mail or provider is not found, or the AI request fails.
#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id, provider_id = %provider_id), err(Debug))]
pub async fn summarize_mail(
    mail_id: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let body = state
        .db
        .get_mail_body_text(&mail_id)
        .map_err(|e| e.to_payload())?
        .unwrap_or_default();
    if body.trim().is_empty() {
        return Ok(String::new());
    }

    let ai = state.ai_service.read().await;
    let messages = vec![
        ChatMessage::new(
            "system",
            "You are a helpful assistant. Summarize the following email concisely in 3-5 sentences. Use the same language as the email.",
        ),
        ChatMessage::new("user", body),
    ];
    let result = ai
        .complete(&provider_id, &messages, None)
        .await
        .map_err(|e| e.to_payload())?;
    drop(ai);
    Ok(result.content.trim().to_string())
}

/// Extracts action items / to-dos from a mail using the specified AI provider.
///
/// # Errors
///
/// Returns an error if the mail or provider is not found, or the AI response cannot be parsed.
#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id, provider_id = %provider_id), err(Debug))]
pub async fn extract_todos(
    mail_id: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, ErrorPayload> {
    let body = state
        .db
        .get_mail_body_text(&mail_id)
        .map_err(|e| e.to_payload())?
        .unwrap_or_default();
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }

    let ai = state.ai_service.read().await;
    let messages = vec![
        ChatMessage::new(
            "system",
            "You are a helpful assistant. Extract action items / to-dos from the following email. Return ONLY a JSON array of strings, with no markdown formatting and no extra commentary.",
        ),
        ChatMessage::new("user", body),
    ];
    let result = ai
        .complete(&provider_id, &messages, None)
        .await
        .map_err(|e| e.to_payload())?;
    drop(ai);

    parse_todo_json(&result.content).map_err(|e| e.to_payload())
}

/// Assists with email composition using the specified AI provider.
///
/// # Errors
///
/// Returns an error if the provider is not found or the AI request fails.
#[tauri::command]
#[instrument(skip(state), fields(action = %action, provider_id = %provider_id), err(Debug))]
pub async fn ai_compose_assist(
    action: String,
    content: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let prompt = match action.as_str() {
        "write" => {
            "You are a helpful email writing assistant. Write a professional email based on the following draft or notes. Use the same language as the input. Output only the email body, no extra commentary."
        }
        "polish" => {
            "You are a helpful writing assistant. Polish and improve the following text while preserving its meaning. Use the same language as the input. Output only the improved text, no extra commentary."
        }
        "optimize-en" => {
            "You are a helpful writing assistant. Optimize the following text in English to make it more natural, professional, and fluent. Output only the optimized English text, no extra commentary."
        }
        _ => {
            "You are a helpful writing assistant. Improve the following text. Output only the improved text, no extra commentary."
        }
    };

    let ai = state.ai_service.read().await;
    let messages = vec![
        ChatMessage::new("system", prompt),
        ChatMessage::new("user", content),
    ];
    let result = ai
        .complete(&provider_id, &messages, None)
        .await
        .map_err(|e| e.to_payload())?;
    drop(ai);
    Ok(result.content.trim().to_string())
}

fn parse_todo_json(content: &str) -> Result<Vec<String>, AeroError> {
    let trimmed = content.trim();
    let inner = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map_or(trimmed, str::trim);
    let value = serde_json::from_str::<serde_json::Value>(inner)
        .map_err(|e| AeroError::AiApiError(format!("invalid todo JSON: {e}")))?;
    let array = value
        .as_array()
        .ok_or_else(|| AeroError::AiApiError("AI todo response is not a JSON array".to_string()))?;
    Ok(array
        .iter()
        .filter_map(|v| v.as_str().map(std::string::String::from))
        .collect())
}

async fn refresh_tool_registry(state: &AppState) {
    if let (Ok(servers), Ok(skills)) = (state.db.list_ai_mcp_servers(), state.db.list_ai_skills()) {
        state
            .tool_registry
            .write()
            .await
            .refresh(&servers, &skills)
            .await;
    }
}
