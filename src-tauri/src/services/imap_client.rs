use async_imap::types::Flag;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tracing::warn;

use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig, TlsMode};

/// An authenticated IMAP session over a Tokio TLS stream.
pub type ImapSession = async_imap::Session<TlsStream<TcpStream>>;

struct Xoauth2Auth {
    user: String,
    access_token: String,
}

impl async_imap::Authenticator for &Xoauth2Auth {
    type Response = String;

    fn process(&mut self, _data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

/// Connects to the IMAP server using the provided account configuration.
///
/// # Errors
///
/// Returns an error if the connection or authentication fails.
pub async fn connect_imap(config: &AccountConfig) -> Result<ImapSession, AeroError> {
    let domain = config.imap.host.clone();
    let tls = build_tls_connector(config)?;

    let client = match config.imap.tls_mode {
        TlsMode::Required => connect_tls(&domain, config.imap.port, &tls).await?,
        TlsMode::StartTls => connect_starttls(&domain, config.imap.port, &tls).await?,
        TlsMode::None => {
            return Err(AeroError::InvalidConfig(
                "Plain IMAP without TLS is not supported".into(),
            ));
        }
    };

    authenticate(client, config).await
}

fn build_tls_connector(
    config: &AccountConfig,
) -> Result<tokio_native_tls::TlsConnector, AeroError> {
    let mut tls_builder = native_tls::TlsConnector::builder();
    if !config.advanced.verify_certificate {
        tls_builder.danger_accept_invalid_certs(true);
    }
    if let Some(ref cert_path) = config.advanced.ca_cert_path {
        let cert =
            std::fs::read(cert_path).map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
        let cert = native_tls::Certificate::from_pem(&cert)
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
        tls_builder.add_root_certificate(cert);
    }
    let tls = tls_builder
        .build()
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    Ok(tokio_native_tls::TlsConnector::from(tls))
}

async fn connect_tls(
    domain: &str,
    port: u16,
    tls: &tokio_native_tls::TlsConnector,
) -> Result<async_imap::Client<TlsStream<TcpStream>>, AeroError> {
    let tcp_stream = TcpStream::connect((domain, port))
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let tls_stream = tls
        .connect(domain, tcp_stream)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let mut client = async_imap::Client::new(tls_stream);
    read_greeting(&mut client).await?;
    Ok(client)
}

async fn connect_starttls(
    domain: &str,
    port: u16,
    tls: &tokio_native_tls::TlsConnector,
) -> Result<async_imap::Client<TlsStream<TcpStream>>, AeroError> {
    let tcp_stream = TcpStream::connect((domain, port))
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let mut client = async_imap::Client::new(tcp_stream);
    read_greeting(&mut client).await?;
    client
        .run_command_and_check_ok("STARTTLS", None)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let stream = client.into_inner();
    let tls_stream = tls
        .connect(domain, stream)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    Ok(async_imap::Client::new(tls_stream))
}

async fn read_greeting<T>(client: &mut async_imap::Client<T>) -> Result<(), AeroError>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + std::fmt::Debug,
{
    client
        .read_response()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .ok_or_else(|| AeroError::ImapConnectionFailed("no greeting from server".into()))?;
    Ok(())
}

async fn authenticate(
    client: async_imap::Client<TlsStream<TcpStream>>,
    config: &AccountConfig,
) -> Result<ImapSession, AeroError> {
    let login_user = config.email.clone().unwrap_or_else(|| config.name.clone());

    match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            client
                .login(&login_user, &password)
                .await
                .map_err(|(e, _)| AeroError::ImapAuthFailed(e.to_string()))
        }
        AuthConfig::OAuth2 { access_token, .. } => {
            let auth = Xoauth2Auth {
                user: login_user,
                access_token: access_token.clone(),
            };
            client
                .authenticate("XOAUTH2", &auth)
                .await
                .map_err(|(e, _)| AeroError::ImapAuthFailed(e.to_string()))
        }
    }
}

/// Deletes a mail on the IMAP server by its folder path and UID.
///
/// # Errors
///
/// Returns an error if the IMAP operation fails.
pub async fn delete_mail_on_server(
    session: &mut ImapSession,
    folder_path: &str,
    uid: u32,
) -> Result<(), AeroError> {
    use futures::TryStreamExt;

    session
        .select(folder_path)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let _deleted: Vec<_> = session
        .uid_store(format!("{uid}"), "+FLAGS (\\Deleted)")
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .try_collect()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    session
        .expunge()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    Ok(())
}

/// Moves a mail on the IMAP server by copying it to the target folder and
/// marking the original as deleted.
///
/// # Errors
///
/// Returns an error if the IMAP operation fails.
pub async fn move_mail_on_server(
    session: &mut ImapSession,
    folder_path: &str,
    uid: u32,
    target_folder: &str,
) -> Result<(), AeroError> {
    use futures::TryStreamExt;

    session
        .select(folder_path)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    session
        .uid_copy(format!("{uid}"), target_folder)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let _deleted: Vec<_> = session
        .uid_store(format!("{uid}"), "+FLAGS (\\Deleted)")
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .try_collect()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    session
        .expunge()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    Ok(())
}

/// Converts an async-imap flag iterator into a list of flag strings.
pub fn collect_flags<'a>(flags: impl Iterator<Item = Flag<'a>>) -> Vec<String> {
    flags.map(flag_to_string).collect()
}

fn flag_to_string(flag: Flag<'_>) -> String {
    match flag {
        Flag::Seen => "\\Seen".to_string(),
        Flag::Answered => "\\Answered".to_string(),
        Flag::Flagged => "\\Flagged".to_string(),
        Flag::Deleted => "\\Deleted".to_string(),
        Flag::Draft => "\\Draft".to_string(),
        Flag::Recent => "\\Recent".to_string(),
        Flag::MayCreate => "\\*".to_string(),
        Flag::Custom(cow) => cow.into_owned(),
    }
}

/// Returns `true` if the flag list contains the `\\Seen` flag.
pub fn is_seen_flag<'a>(mut flags: impl Iterator<Item = Flag<'a>>) -> bool {
    flags.any(|f| matches!(f, Flag::Seen))
}

/// Returns `true` if the flag list contains the `\\Flagged` flag.
pub fn is_flagged_flag<'a>(mut flags: impl Iterator<Item = Flag<'a>>) -> bool {
    flags.any(|f| matches!(f, Flag::Flagged))
}

/// Finds the server's drafts folder using common localized names.
///
/// # Errors
///
/// Returns an error if no drafts folder can be located.
pub async fn find_drafts_folder(session: &mut ImapSession) -> Result<String, AeroError> {
    use futures::StreamExt;

    let mut stream = session
        .list(None, Some("*"))
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let mut mailboxes = Vec::new();
    while let Some(name_res) = stream.next().await {
        let name = name_res.map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
        mailboxes.push(name.name().to_string());
    }

    let candidates = [
        "Drafts",
        "Draft",
        "[Gmail]/Drafts",
        "草稿箱",
        "\\u8349\\u7a3f",
    ];
    for candidate in &candidates {
        if let Some(folder) = mailboxes
            .iter()
            .find(|name| name.eq_ignore_ascii_case(candidate))
        {
            return Ok(folder.clone());
        }
    }
    for folder in &mailboxes {
        if folder.to_lowercase().contains("draft") {
            return Ok(folder.clone());
        }
    }
    warn!("Drafts folder not found; candidates were: {:?}", mailboxes);
    Err(AeroError::ImapAppendFailed(
        "Drafts folder not found".to_string(),
    ))
}

/// Appends a raw RFC-2822 message to the given folder with the `\\Draft` flag.
///
/// # Errors
///
/// Returns an error if the append operation fails.
pub async fn append_message(
    session: &mut ImapSession,
    folder: &str,
    message_bytes: &[u8],
) -> Result<(), AeroError> {
    session
        .append(folder, Some("\\Draft"), None, message_bytes)
        .await
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))
}
