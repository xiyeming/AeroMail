use crate::error::AeroError;
use crate::models::translation::{TraditionalProviderKind, TranslationProvider};

/// Dispatches a translation request to the appropriate traditional provider.
///
/// # Errors
///
/// Returns an error if the provider type is unsupported or the API call fails.
pub fn translate(
    provider: &TranslationProvider,
    source_text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    match provider {
        TranslationProvider::Traditional {
            kind,
            api_key_encrypted,
            endpoint,
            ..
        } => {
            let api_key = String::from_utf8(api_key_encrypted.clone())
                .map_err(|e| AeroError::TranslationApiError(format!("invalid key: {e}")))?;
            match kind {
                TraditionalProviderKind::GoogleTranslate => {
                    google_translate(&api_key, source_text, target_lang)
                }
                TraditionalProviderKind::DeepL => {
                    deepl_translate(&api_key, source_text, target_lang, endpoint.as_deref())
                }
                _ => Err(AeroError::TranslationApiError(
                    "provider not yet implemented".to_string(),
                )),
            }
        }
        TranslationProvider::Ai { .. } => Err(AeroError::TranslationApiError(
            "not a traditional provider".to_string(),
        )),
    }
}

fn google_translate(api_key: &str, text: &str, target_lang: &str) -> Result<String, AeroError> {
    let url = format!("https://translation.googleapis.com/language/translate/v2?key={api_key}");
    let body = serde_json::json!({
        "q": text,
        "target": target_lang,
    });
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data["data"]["translations"][0]["translatedText"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}

fn deepl_translate(
    api_key: &str,
    text: &str,
    target_lang: &str,
    endpoint: Option<&str>,
) -> Result<String, AeroError> {
    let base = endpoint.unwrap_or("https://api-free.deepl.com");
    let url = format!("{base}/v2/translate");
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
        .form(&[("text", text), ("target_lang", target_lang)])
        .send()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data["translations"][0]["text"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}
