use lettre::address::{Address, Envelope};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig, TlsMode};

/// Sends a pre-built MIME message via SMTP.
pub async fn send_message(
    config: &AccountConfig,
    message_bytes: Vec<u8>,
) -> Result<(), AeroError> {
    let creds = build_credentials(config);

    let tls_parameters = TlsParameters::new(config.smtp.host.clone())
        .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?;

    let mailer = match config.smtp.tls_mode {
        TlsMode::Required => {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)
                .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
                .port(config.smtp.port)
                .tls(Tls::Required(tls_parameters))
                .credentials(creds)
                .build()
        }
        TlsMode::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp.host)
                .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
                .port(config.smtp.port)
                .tls(Tls::Required(tls_parameters))
                .credentials(creds)
                .build()
        }
        TlsMode::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp.host.clone())
                .port(config.smtp.port)
                .credentials(creds)
                .build()
        }
    };

    // Parse the envelope from the MIME message bytes
    let envelope = parse_envelope_from_mime(&message_bytes)?;

    mailer
        .send_raw(&envelope, &message_bytes)
        .await
        .map_err(|e| {
            // Check if it's an authentication failure by looking at the error status code
            // SMTP 5xx codes indicate permanent errors, including auth failures
            if let Some(status) = e.status() {
                let code: u16 = status.into();
                if (500..=535).contains(&code) || code == 454 {
                    return AeroError::SmtpAuthFailed(e.to_string());
                }
            }
            AeroError::SmtpConnectionFailed(e.to_string())
        })?;

    Ok(())
}

fn build_credentials(config: &AccountConfig) -> Credentials {
    match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            Credentials::new(config.name.clone(), password.to_string())
        }
        AuthConfig::OAuth2 { access_token, .. } => {
            // XOAUTH2 format: base64("user={user}\x01auth=Bearer {token}\x01\x01")
            let xoauth2 = format!(
                "user={}\x01auth=Bearer {}\x01\x01",
                config.name, access_token
            );
            Credentials::new(config.name.clone(), xoauth2)
        }
    }
}

/// Extracts sender and recipients from MIME headers to build an SMTP envelope.
fn parse_envelope_from_mime(message_bytes: &[u8]) -> Result<Envelope, AeroError> {
    let text = std::str::from_utf8(message_bytes)
        .map_err(|e| AeroError::MailBuilderFailed(format!("invalid utf8: {e}")))?;

    let mut from_addr: Option<Address> = None;
    let mut to_addrs: Vec<Address> = Vec::new();

    for line in text.lines() {
        // Headers and body are separated by a blank line
        if line.is_empty() {
            break;
        }

        let lower = line.to_lowercase();
        if lower.starts_with("from:") {
            let addr = extract_address(line)?;
            from_addr = Some(addr);
        } else if lower.starts_with("to:") {
            let addrs = extract_addresses(line)?;
            to_addrs.extend(addrs);
        } else if lower.starts_with("cc:") {
            let addrs = extract_addresses(line)?;
            to_addrs.extend(addrs);
        } else if lower.starts_with("bcc:") {
            let addrs = extract_addresses(line)?;
            to_addrs.extend(addrs);
        }
    }

    if to_addrs.is_empty() {
        return Err(AeroError::InvalidRecipient(
            "no recipients found in message".to_string(),
        ));
    }

    let from_addr = from_addr.ok_or_else(|| {
        AeroError::InvalidRecipient("missing From header in message".to_string())
    })?;

    Envelope::new(Some(from_addr), to_addrs)
        .map_err(|e| AeroError::MailBuilderFailed(format!("invalid envelope: {e}")))
}

/// Extracts a single email address from a header line like "From: Name <email@example.com>".
fn extract_address(line: &str) -> Result<Address, AeroError> {
    let content = line
        .splitn(2, ':')
        .nth(1)
        .unwrap_or("")
        .trim();

    // Try to extract email between angle brackets
    if let Some(start) = content.find('<') {
        if let Some(end) = content.find('>') {
            let email = content[start + 1..end].trim();
            return email.parse().map_err(|_| {
                AeroError::InvalidRecipient(format!("invalid address in: {content}"))
            });
        }
    }

    // Fallback: try parsing the whole content as an address
    content.parse().map_err(|_| {
        AeroError::InvalidRecipient(format!("invalid address in: {content}"))
    })
}

/// Extracts multiple comma-separated email addresses from a header line.
fn extract_addresses(line: &str) -> Result<Vec<Address>, AeroError> {
    let content = line
        .splitn(2, ':')
        .nth(1)
        .unwrap_or("")
        .trim();

    let mut addresses = Vec::new();

    for part in content.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Try to extract email between angle brackets
        let email = if let Some(start) = part.find('<') {
            if let Some(end) = part.find('>') {
                part[start + 1..end].trim()
            } else {
                part
            }
        } else {
            part
        };

        let addr: Address = email.parse().map_err(|_| {
            AeroError::InvalidRecipient(format!("invalid address in: {part}"))
        })?;
        addresses.push(addr);
    }

    Ok(addresses)
}
