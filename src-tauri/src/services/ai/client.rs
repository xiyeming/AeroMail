use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AeroError;
use crate::models::ai::AiProvider;
use crate::services::ai::ChatMessage;

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessagePayload>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessagePayload {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessagePayload,
}

/// Sends a chat completion request to an OpenAI-compatible API.
///
/// # Errors
///
/// Returns an error if the API key is invalid, the HTTP request fails,
/// or the response cannot be parsed.
pub async fn chat_completion(
    client: &Client,
    provider: &AiProvider,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
) -> Result<String, AeroError> {
    let api_key = String::from_utf8(provider.api_key_encrypted.clone())
        .map_err(|e| AeroError::AiApiError(format!("invalid api key: {e}")))?;
    let base_url = provider
        .base_url
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            AeroError::AiApiError("AI provider base_url is not configured".to_string())
        })?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let payload_messages: Vec<ChatMessagePayload> = messages
        .into_iter()
        .map(|m| ChatMessagePayload {
            role: m.role,
            content: m.content,
        })
        .collect();

    let body = ChatCompletionRequest {
        model: provider.model.clone(),
        messages: payload_messages,
        max_tokens,
    };

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if status.as_u16() == 429 {
            return Err(AeroError::AiRateLimited);
        }
        return Err(AeroError::AiApiError(format!("HTTP {status}: {text}")));
    }

    let data: ChatCompletionResponse = res
        .json()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    data.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AeroError::AiApiError("empty response".to_string()))
}
