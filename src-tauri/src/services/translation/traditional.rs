use std::collections::HashMap;

use tracing::{debug, instrument};

use crate::error::AeroError;
use crate::models::translation::{TraditionalProviderKind, TranslationProvider};

/// Dispatches a translation request to the appropriate traditional provider.
///
/// # Errors
///
/// Returns an error if the provider type is unsupported or the API call fails.
#[instrument(skip(provider), fields(provider_id = %match provider { TranslationProvider::Traditional { id, .. } | TranslationProvider::Ai { id, .. } => id.as_str() }, kind = ?match provider { TranslationProvider::Traditional { kind, .. } => Some(kind), TranslationProvider::Ai { .. } => None }), err(Debug))]
pub async fn translate(
    provider: &TranslationProvider,
    source_text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    match provider {
        TranslationProvider::Traditional {
            kind,
            api_key_encrypted,
            endpoint,
            extra,
            ..
        } => {
            let api_key_bytes = crate::services::crypto::decrypt_password(api_key_encrypted)
                .map_err(|e| {
                    AeroError::TranslationApiError(format!("failed to decrypt key: {e}"))
                })?;
            let api_key = String::from_utf8(api_key_bytes)
                .map_err(|e| AeroError::TranslationApiError(format!("invalid key: {e}")))?;
            match kind {
                TraditionalProviderKind::GoogleTranslate => {
                    google_translate(&api_key, source_text, target_lang).await
                }
                TraditionalProviderKind::DeepL => {
                    deepl_translate(&api_key, source_text, target_lang, endpoint.as_deref()).await
                }
                TraditionalProviderKind::AzureTranslator => {
                    azure_translate(
                        &api_key,
                        source_text,
                        target_lang,
                        endpoint.as_deref(),
                        extra,
                    )
                    .await
                }
                TraditionalProviderKind::Baidu => {
                    baidu_translate(
                        &api_key,
                        source_text,
                        target_lang,
                        endpoint.as_deref(),
                        extra,
                    )
                    .await
                }
                TraditionalProviderKind::Custom => {
                    custom_translate(&api_key, source_text, target_lang, endpoint.as_deref()).await
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

#[instrument(skip(api_key), fields(text_len = text.len(), target_lang = %target_lang), err(Debug))]
async fn google_translate(
    api_key: &str,
    text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    let url = format!("https://translation.googleapis.com/language/translate/v2?key={api_key}");
    let body = serde_json::json!({
        "q": text,
        "target": target_lang,
    });
    debug!("calling Google Translate API");
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data["data"]["translations"][0]["translatedText"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}

#[instrument(skip(api_key), fields(text_len = text.len(), target_lang = %target_lang), err(Debug))]
async fn deepl_translate(
    api_key: &str,
    text: &str,
    target_lang: &str,
    endpoint: Option<&str>,
) -> Result<String, AeroError> {
    let base = endpoint.unwrap_or("https://api-free.deepl.com");
    let url = format!("{base}/v2/translate");
    debug!("calling DeepL API");
    let resp = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
        .form(&[("text", text), ("target_lang", target_lang)])
        .send()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data["translations"][0]["text"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}

#[instrument(skip(api_key, extra), fields(text_len = text.len(), target_lang = %target_lang), err(Debug))]
async fn azure_translate(
    api_key: &str,
    text: &str,
    target_lang: &str,
    endpoint: Option<&str>,
    extra: &HashMap<String, String>,
) -> Result<String, AeroError> {
    let base = endpoint.unwrap_or("https://api.cognitive.microsofttranslator.com");
    let target = normalize_azure_lang(target_lang);
    let url = format!("{base}/translate?api-version=3.0&to={target}");
    let body = serde_json::json!([{ "Text": text }]);

    debug!("calling Azure Translator API");
    let mut req = reqwest::Client::new()
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&body);

    if let Some(region) = extra.get("region") {
        req = req.header("Ocp-Apim-Subscription-Region", region);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data[0]["translations"][0]["text"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}

#[instrument(skip(api_key, extra), fields(text_len = text.len(), target_lang = %target_lang), err(Debug))]
async fn baidu_translate(
    api_key: &str,
    text: &str,
    target_lang: &str,
    endpoint: Option<&str>,
    extra: &HashMap<String, String>,
) -> Result<String, AeroError> {
    let app_id = extra.get("app_id").ok_or_else(|| {
        AeroError::TranslationApiError("Baidu app_id is required in extra config".to_string())
    })?;
    let secret = api_key;
    let salt = rand::random::<u64>().to_string();
    let sign_input = format!("{app_id}{text}{salt}{secret}");
    let sign = format!("{:x}", md5::compute(sign_input));
    let target = normalize_baidu_lang(target_lang);
    let base = endpoint.unwrap_or("https://fanyi-api.baidu.com");
    let url = format!("{base}/api/trans/vip/translate");

    debug!("calling Baidu Translate API");
    let resp = reqwest::Client::new()
        .post(&url)
        .form(&[
            ("q", text),
            ("from", "auto"),
            ("to", target),
            ("appid", app_id),
            ("salt", &salt),
            ("sign", &sign),
        ])
        .send()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if let Some(error_code) = data["error_code"].as_str() {
        let error_msg = data["error_msg"].as_str().unwrap_or("unknown error");
        return Err(AeroError::TranslationApiError(format!(
            "Baidu {error_code}: {error_msg}"
        )));
    }
    data["trans_result"][0]["dst"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}

#[instrument(skip(api_key), fields(text_len = text.len(), target_lang = %target_lang), err(Debug))]
async fn custom_translate(
    api_key: &str,
    text: &str,
    target_lang: &str,
    endpoint: Option<&str>,
) -> Result<String, AeroError> {
    let url = endpoint.ok_or_else(|| {
        AeroError::TranslationApiError("Custom provider endpoint is required".to_string())
    })?;
    debug!("calling custom translation API");
    let resp = reqwest::Client::new()
        .post(url)
        .header("Authorization", format!("Bearer {api_key}"))
        .form(&[
            ("q", text),
            ("source", "auto"),
            ("target", target_lang),
            ("api_key", api_key),
        ])
        .send()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!(
            "HTTP {status}: {body}"
        )));
    }
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;

    // LibreTranslate returns [{"translatedText": "..."}]
    if let Some(text) = data[0]["translatedText"].as_str() {
        return Ok(text.to_string());
    }
    // Generic fallback: { "translatedText": "..." }
    if let Some(text) = data["translatedText"].as_str() {
        return Ok(text.to_string());
    }
    Err(AeroError::TranslationApiError(
        "unexpected response".to_string(),
    ))
}

fn normalize_azure_lang(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        "zh-cn" | "zh" | "zh-hans" => "zh-Hans",
        "zh-tw" | "zh-hant" => "zh-Hant",
        _ => lang,
    }
}

fn normalize_baidu_lang(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        "zh-cn" | "zh" | "zh-hans" => "zh",
        "zh-tw" | "zh-hant" => "cht",
        "en" => "en",
        "ja" | "jp" => "jp",
        "ko" => "kor",
        "fr" => "fra",
        "es" => "spa",
        "de" => "de",
        "ru" => "ru",
        "it" => "it",
        "pt" => "pt",
        "ar" => "ara",
        "th" => "th",
        "vi" => "vie",
        "id" => "id",
        "ms" => "may",
        "tr" => "tr",
        _ => lang,
    }
}
