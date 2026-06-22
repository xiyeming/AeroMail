use chrono::Utc;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig};

/// Refreshes an `OAuth2` access token if it is missing or about to expire.
///
/// When `account_id` and `db` are provided and a refresh occurs, the updated
/// credentials are persisted back to the `accounts` table so subsequent loads
/// see the new tokens.
///
/// # Errors
///
/// Returns an error if the token endpoint is unreachable, the refresh token
/// is missing, or the response does not contain a new access token.
pub async fn ensure_access_token(
    account_id: Option<&str>,
    config: &mut AccountConfig,
    db: Option<&Database>,
) -> Result<(), AeroError> {
    let AuthConfig::OAuth2 {
        access_token,
        refresh_token,
        expires_at,
        token_url,
        client_id,
        client_secret,
    } = &mut config.auth
    else {
        return Ok(());
    };

    // Keep a safety margin of 60 seconds so the token does not expire mid-flight.
    if !access_token.is_empty() && *expires_at > Utc::now().timestamp() + 60 {
        return Ok(());
    }

    if refresh_token.is_empty() {
        return Err(AeroError::OAuth2RefreshFailed(
            "refresh token is missing".to_string(),
        ));
    }

    let token_url = token_url
        .as_deref()
        .ok_or(AeroError::OAuth2ConfigIncomplete)?;
    let client_id = client_id
        .as_deref()
        .ok_or(AeroError::OAuth2ConfigIncomplete)?;

    let mut params = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
        ("client_id", client_id),
    ];
    let secret_str;
    if let Some(secret) = client_secret.as_deref() {
        secret_str = secret;
        params.push(("client_secret", secret_str));
    }

    let response = reqwest::Client::new()
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| AeroError::OAuth2RefreshFailed(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unable to read body".to_string());
        return Err(AeroError::OAuth2RefreshFailed(format!(
            "HTTP {status}: {body}"
        )));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AeroError::OAuth2RefreshFailed(e.to_string()))?;

    let new_access = data["access_token"]
        .as_str()
        .ok_or_else(|| AeroError::OAuth2RefreshFailed("missing access_token".to_string()))?;

    *access_token = new_access.to_string();

    if let Some(new_refresh) = data["refresh_token"].as_str() {
        *refresh_token = new_refresh.to_string();
    }

    if let Some(expires_in) = data["expires_in"].as_i64() {
        *expires_at = Utc::now().timestamp() + expires_in;
    } else {
        *expires_at = Utc::now().timestamp() + 3600;
    }

    if let (Some(id), Some(database)) = (account_id, db) {
        let creds_json = serde_json::to_string(&config.auth)?;
        database.update_account_auth_credentials(id, creds_json.as_bytes())?;
    }

    Ok(())
}

/// Refreshes the `OAuth2` access token for the given account and returns the
/// updated configuration.
///
/// # Errors
///
/// Returns an error if the account cannot be loaded or refreshed.
pub async fn refresh_account_tokens(
    account_id: &str,
    account_manager: &crate::services::account_manager::AccountManager,
) -> Result<AccountConfig, AeroError> {
    let mut config = account_manager.get_account_config(account_id)?;
    ensure_access_token(Some(account_id), &mut config, Some(account_manager.db())).await?;
    Ok(config)
}
