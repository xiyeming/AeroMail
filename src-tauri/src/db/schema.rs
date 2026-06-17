pub const ACCOUNTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL,
    smtp_host TEXT NOT NULL,
    smtp_port INTEGER NOT NULL,
    tls_mode TEXT NOT NULL,
    auth_type TEXT NOT NULL,
    auth_credentials_encrypted BLOB,
    ca_cert_path TEXT,
    verify_certificate INTEGER DEFAULT 1,
    connect_timeout INTEGER DEFAULT 30,
    read_timeout INTEGER DEFAULT 30,
    keepalive INTEGER DEFAULT 1,
    sync_interval INTEGER DEFAULT 60,
    excluded_folders TEXT,
    created_at INTEGER,
    updated_at INTEGER
)
"#;

pub const FOLDERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    unread_count INTEGER DEFAULT 0,
    total_count INTEGER DEFAULT 0,
    uid_validity INTEGER,
    last_sync_at INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
)
"#;

pub const MAILS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS mails (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    folder_id TEXT NOT NULL,
    uid INTEGER NOT NULL,
    subject TEXT,
    from_name TEXT,
    from_address TEXT,
    to_addresses TEXT,
    cc_addresses TEXT,
    date INTEGER,
    body_html TEXT,
    body_text TEXT,
    is_read INTEGER DEFAULT 0,
    is_starred INTEGER DEFAULT 0,
    flags TEXT,
    created_at INTEGER,
    indexed_at INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
)
"#;

pub const ATTACHMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    mail_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size INTEGER,
    content_id TEXT,
    local_path TEXT,
    is_inline INTEGER DEFAULT 0,
    FOREIGN KEY (mail_id) REFERENCES mails(id) ON DELETE CASCADE
)
"#;

pub const DRAFTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS drafts (
    id TEXT PRIMARY KEY,
    account_id TEXT,
    subject TEXT,
    to_addresses TEXT,
    cc_addresses TEXT,
    body_html TEXT,
    body_text TEXT,
    attachments_json TEXT,
    saved_at INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE SET NULL
)
"#;

pub const SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at INTEGER
)
"#;

pub const ALL_SCHEMAS: &[&str] = &[
    ACCOUNTS_TABLE,
    FOLDERS_TABLE,
    MAILS_TABLE,
    ATTACHMENTS_TABLE,
    DRAFTS_TABLE,
    SETTINGS_TABLE,
];
