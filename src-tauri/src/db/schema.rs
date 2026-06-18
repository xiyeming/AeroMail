pub const ACCOUNTS_TABLE: &str = r"
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
";

pub const FOLDERS_TABLE: &str = r"
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
";

pub const MAILS_TABLE: &str = r"
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
";

pub const ATTACHMENTS_TABLE: &str = r"
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
";

pub const DRAFTS_TABLE: &str = r"
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
";

pub const SETTINGS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at INTEGER
)
";

pub const AI_PROVIDERS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS ai_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    api_key_encrypted BLOB NOT NULL,
    base_url TEXT,
    model TEXT NOT NULL,
    max_tokens INTEGER
)
";

pub const AI_CHAT_SESSIONS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS ai_chat_sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    provider_id TEXT NOT NULL,
    model TEXT NOT NULL,
    context_mail_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)
";

pub const AI_CHAT_MESSAGES_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS ai_chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
";

pub const AI_CHAT_MESSAGES_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_ai_messages_session
ON ai_chat_messages(session_id, created_at)
";

pub const TRANSLATION_PROVIDERS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS translation_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    config_json TEXT NOT NULL
)
";

pub const TRANSLATIONS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS translations (
    id TEXT PRIMARY KEY,
    source_hash TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
";

pub const TRANSLATIONS_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_translations_lookup
ON translations(source_hash, target_lang, provider_id)
";

pub const ALL_SCHEMAS: &[&str] = &[
    ACCOUNTS_TABLE,
    FOLDERS_TABLE,
    MAILS_TABLE,
    ATTACHMENTS_TABLE,
    DRAFTS_TABLE,
    SETTINGS_TABLE,
    AI_PROVIDERS_TABLE,
    AI_CHAT_SESSIONS_TABLE,
    AI_CHAT_MESSAGES_TABLE,
    AI_CHAT_MESSAGES_INDEX,
    TRANSLATION_PROVIDERS_TABLE,
    TRANSLATIONS_TABLE,
    TRANSLATIONS_INDEX,
];
