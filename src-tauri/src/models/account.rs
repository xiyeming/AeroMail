use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    pub provider: MailProvider,
    pub imap: ServerConfig,
    pub smtp: ServerConfig,
    pub auth: AuthConfig,
    pub advanced: AdvancedConfig,
    pub sync_interval_secs: u64,
    pub excluded_folders: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MailProvider {
    Gmail,
    Outlook,
    QQ,
    Netease163,
    Aliyun,
    TencentExmail,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_mode: TlsMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TlsMode {
    Required,
    StartTls,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    OAuth2 {
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    },
    Password {
        password_encrypted: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub ca_cert_path: Option<String>,
    pub verify_certificate: bool,
    pub connect_timeout_secs: u64,
    pub read_timeout_secs: u64,
    pub keepalive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub name: String,
    pub provider: MailProvider,
    pub imap_host: String,
    pub smtp_host: String,
}
