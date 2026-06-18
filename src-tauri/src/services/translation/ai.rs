use crate::error::AeroError;
use crate::models::ai::AiProvider;
use crate::services::ai::ChatMessage;

/// Translates `source_text` into `target_lang` via an AI provider.
///
/// Uses the async `chat_completion` client internally, bridged via a
/// dedicated Tokio runtime since this function is synchronous.
///
/// # Errors
///
/// Returns an error if the runtime cannot be created or the API call fails.
pub fn translate(
    ai_provider: &AiProvider,
    source_text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    let prompt = format!(
        "Translate the following text to {target_lang}. \
         Only output the translation, no explanation:\n\n{source_text}"
    );

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let client = reqwest::Client::new();
    let max_tokens = ai_provider.max_tokens.unwrap_or(2048);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    rt.block_on(crate::services::ai::client::chat_completion(
        &client,
        ai_provider,
        messages,
        max_tokens,
    ))
}
