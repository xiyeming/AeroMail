use tauri::State;
use tracing::{debug, instrument};

use crate::AppState;
use crate::models::error::ErrorPayload;
use crate::models::translation::{TranslationProvider, TranslationProviderSummary};

/// Lists all configured translation providers as summaries.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn list_translation_providers(
    state: State<'_, AppState>,
) -> Result<Vec<TranslationProviderSummary>, ErrorPayload> {
    let providers = state
        .db
        .list_translation_providers()
        .map_err(|e| e.to_payload())?;
    Ok(providers
        .into_iter()
        .map(|p| {
            let (id, name, provider_type) = match &p {
                TranslationProvider::Traditional { id, name, .. } => {
                    (id.clone(), name.clone(), "traditional".to_string())
                }
                TranslationProvider::Ai { id, name, .. } => {
                    (id.clone(), name.clone(), "ai".to_string())
                }
            };
            TranslationProviderSummary {
                id,
                name,
                provider_type,
            }
        })
        .collect())
}

/// Creates or updates a translation provider in the database.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state, provider), fields(provider_id = %match &provider { TranslationProvider::Traditional { id, .. } | TranslationProvider::Ai { id, .. } => id.as_str() }), err(Debug))]
pub async fn upsert_translation_provider(
    provider: TranslationProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let id = match &provider {
        TranslationProvider::Traditional { id, .. } | TranslationProvider::Ai { id, .. } => {
            id.clone()
        }
    };
    state
        .db
        .upsert_translation_provider(&provider)
        .map_err(|e| e.to_payload())?;
    Ok(id)
}

/// Deletes a translation provider by ID.
///
/// # Errors
///
/// Returns an error if the provider is not found or the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(provider_id = %provider_id), err(Debug))]
pub async fn delete_translation_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .delete_translation_provider(&provider_id)
        .map_err(|e| e.to_payload())
}

/// Translates a mail's body text into the target language using the specified provider.
///
/// # Errors
///
/// Returns an error if the mail is not found, the provider is not found,
/// or the translation API call fails.
#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id, target_lang = %target_lang, provider_id = %provider_id), err(Debug))]
pub async fn translate_mail_text(
    mail_id: String,
    target_lang: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    debug!("translating mail body");
    let text = state
        .db
        .get_mail_body_text(&mail_id)
        .map_err(|e| e.to_payload())?
        .unwrap_or_default();
    let translation = state
        .translation_service
        .translate_mail(&text, &target_lang, &provider_id)
        .map_err(|e| e.to_payload())?;
    Ok(translation)
}

/// Retrieves a cached translation for a mail's body text, if one exists.
///
/// # Errors
///
/// Returns an error if the mail is not found or the database query fails.
#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id, target_lang = %target_lang), err(Debug))]
pub async fn get_cached_translation(
    mail_id: String,
    target_lang: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, ErrorPayload> {
    debug!("checking cached translation");
    let text = state
        .db
        .get_mail_body_text(&mail_id)
        .map_err(|e| e.to_payload())?
        .unwrap_or_default();
    let source_hash = crate::services::translation::sha256_hex(&text);
    let cached = state
        .db
        .get_any_translation(&source_hash, &target_lang)
        .map_err(|e| e.to_payload())?;
    Ok(cached.map(|c| c.translated_text))
}
