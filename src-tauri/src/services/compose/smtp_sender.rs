use std::time::Duration;

use lettre::address::{Address, Envelope};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::client::{Certificate, Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use tracing::{debug, info, instrument};

use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig, TlsMode};

/// Sends a pre-built MIME message via SMTP.
#[instrument(skip_all, fields(host = %config.smtp.host, port = config.smtp.port, tls_mode = ?config.smtp.tls_mode), err(Debug))]
pub async fn send_message(config: &AccountConfig, message_bytes: Vec<u8>) -> Result<(), AeroError> {
    info!("building SMTP transport");
    let creds = build_credentials(config);
    let tls_parameters = build_tls_parameters(config)?;

    let mailer = match config.smtp.tls_mode {
        TlsMode::Required => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)
            .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
            .port(config.smtp.port)
            .tls(Tls::Wrapper(tls_parameters))
            .credentials(creds)
            .authentication(auth_mechanisms(config))
            .timeout(Some(Duration::from_secs(
                config.advanced.connect_timeout_secs,
            )))
            .build(),
        TlsMode::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp.host)
                .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
                .port(config.smtp.port)
                .tls(Tls::Required(tls_parameters))
                .credentials(creds)
                .authentication(auth_mechanisms(config))
                .timeout(Some(Duration::from_secs(
                    config.advanced.connect_timeout_secs,
                )))
                .build()
        }
        TlsMode::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp.host.clone())
                .port(config.smtp.port)
                .credentials(creds)
                .authentication(auth_mechanisms(config))
                .timeout(Some(Duration::from_secs(
                    config.advanced.connect_timeout_secs,
                )))
                .build()
        }
    };

    // Parse the envelope from the MIME message bytes
    let envelope = parse_envelope_from_mime(&message_bytes)?;

    debug!("sending raw message over SMTP");
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

    info!("SMTP send complete");
    Ok(())
}

fn build_tls_parameters(config: &AccountConfig) -> Result<TlsParameters, AeroError> {
    let mut builder = TlsParameters::builder(config.smtp.host.clone());

    if !config.advanced.verify_certificate {
        builder = builder.dangerous_accept_invalid_certs(true);
    }

    if let Some(path) = &config.advanced.ca_cert_path {
        debug!(ca_cert_path = %path, "loading custom CA certificate");
        let cert_bytes = std::fs::read(path)
            .map_err(|e| AeroError::SmtpConnectionFailed(format!("cannot read CA cert: {e}")))?;
        let cert = Certificate::from_pem(&cert_bytes)
            .map_err(|e| AeroError::SmtpConnectionFailed(format!("invalid CA cert: {e}")))?;
        builder = builder.add_root_certificate(cert);
    }

    builder
        .build()
        .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))
}

fn build_credentials(config: &AccountConfig) -> Credentials {
    let username = config.email.as_deref().unwrap_or(&config.name).to_string();
    match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            Credentials::new(username, password.to_string())
        }
        AuthConfig::OAuth2 { access_token, .. } => {
            // XOAUTH2 SASL initial response: base64("user={user}\x01auth=Bearer {token}\x01\x01")
            let xoauth2 = format!("user={username}\x01auth=Bearer {access_token}\x01\x01");
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

/// Unfolds RFC 2822 folded headers by joining continuation lines (lines
/// starting with whitespace) with the preceding line.
fn unfold_headers(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for line in raw.split("\r\n") {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation line - append trimmed content to previous header.
            result.push(' ');
            result.push_str(line.trim_start());
        } else {
            if !result.is_empty() {
                result.push_str("\r\n");
            }
            result.push_str(line);
        }
    }
    result
}

/// Extracts sender and recipients from MIME headers to build an SMTP envelope.
fn parse_envelope_from_mime(message_bytes: &[u8]) -> Result<Envelope, AeroError> {
    let text = std::str::from_utf8(message_bytes)
        .map_err(|e| AeroError::MailBuilderFailed(format!("invalid utf8: {e}")))?;

    // Unfold folded headers before parsing (RFC 2822 §2.2.3).
    let unfolded = unfold_headers(text);

    let mut from_addr: Option<Address> = None;
    let mut to_addrs: Vec<Address> = Vec::new();

    for line in unfolded.lines() {
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
