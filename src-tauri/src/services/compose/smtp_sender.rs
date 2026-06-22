use lettre::address::{Address, Envelope};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig, TlsMode};

/// Sends a pre-built MIME message via SMTP.
pub async fn send_message(config: &AccountConfig, message_bytes: Vec<u8>) -> Result<(), AeroError> {
    let creds = build_credentials(config);

    let tls_parameters = TlsParameters::new(config.smtp.host.clone())
        .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?;

    let mailer = match config.smtp.tls_mode {
        TlsMode::Required => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)
            .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
            .port(config.smtp.port)
            .tls(Tls::Required(tls_parameters))
            .credentials(creds)
            .authentication(auth_mechanisms(config))
            .build(),
        TlsMode::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp.host)
                .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
                .port(config.smtp.port)
                .tls(Tls::Required(tls_parameters))
                .credentials(creds)
                .authentication(auth_mechanisms(config))
                .build()
        }
        TlsMode::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp.host.clone())
                .port(config.smtp.port)
                .credentials(creds)
                .authentication(auth_mechanisms(config))
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
    let username = config
        .email
        .as_deref()
        .unwrap_or(&config.name)
        .to_string();
    match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            Credentials::new(username, password.to_string())
        }
        AuthConfig::OAuth2 { access_token, .. } => {
            // XOAUTH2 SASL initial response: base64("user={user}\x01auth=Bearer {token}\x01\x01")
            let xoauth2 = format!("user={}\x01auth=Bearer {}\x01\x01", username, access_token);
            Credentials::new(username, xoauth2)
        }
    }
}

fn auth_mechanisms(config: &AccountConfig) -> Vec<Mechanism> {
    match config.auth {
        AuthConfig::OAuth2 { .. } => vec![Mechanism::Xoauth2],
        AuthConfig::Password { .. } => vec![Mechanism::Plain],
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
        let Some(content) = line.split_once(':').map(|x| x.1.trim()) else {
            continue;
        };

        if lower.starts_with("from:") {
            from_addr = Some(parse_address(content)?);
        } else if lower.starts_with("to:") || lower.starts_with("cc:") || lower.starts_with("bcc:")
        {
            to_addrs.extend(parse_addresses(content)?);
        }
    }

    if to_addrs.is_empty() {
        return Err(AeroError::InvalidRecipient(
            "no recipients found in message".to_string(),
        ));
    }

    let from_addr = from_addr
        .ok_or_else(|| AeroError::InvalidRecipient("missing From header in message".to_string()))?;

    Envelope::new(Some(from_addr), to_addrs)
        .map_err(|e| AeroError::MailBuilderFailed(format!("invalid envelope: {e}")))
}

/// Extracts a single email address from a header content value.
fn parse_address(content: &str) -> Result<Address, AeroError> {
    extract_email(content)
        .parse()
        .map_err(|_| AeroError::InvalidRecipient(format!("invalid address in: {content}")))
}

/// Extracts multiple comma-separated email addresses from a header content value.
fn parse_addresses(content: &str) -> Result<Vec<Address>, AeroError> {
    content
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            extract_email(part)
                .parse()
                .map_err(|_| AeroError::InvalidRecipient(format!("invalid address in: {part}")))
        })
        .collect()
}

/// Extracts the bare email address from a value like `"Name" <email@example.com>`.
fn extract_email(part: &str) -> &str {
    part.find('<').map_or(part, |start| {
        part[start + 1..]
            .find('>')
            .map_or(part, |end| part[start + 1..start + 1 + end].trim())
    })
}
