use rusqlite::Connection;

use super::schema::ALL_SCHEMAS;
use crate::error::AeroError;

/// Runs all database migrations to ensure the schema is up to date.
///
/// # Errors
///
/// Returns [`AeroError::Database`] if any migration fails.
pub fn run_migrations(conn: &mut Connection) -> Result<(), AeroError> {
    let tx = conn.transaction()?;
    for schema in ALL_SCHEMAS {
        tx.execute_batch(schema)?;
    }
    run_account_migrations(&tx)?;
    run_draft_migrations(&tx)?;
    run_mail_migrations(&tx)?;
    tx.commit()?;
    Ok(())
}

fn column_exists(tx: &rusqlite::Transaction, table: &str, column: &str) -> Result<bool, AeroError> {
    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2")?;
    let mut rows = stmt.query([table, column])?;
    Ok(rows.next()?.is_some())
}

fn run_account_migrations(tx: &rusqlite::Transaction) -> Result<(), AeroError> {
    if !column_exists(tx, "accounts", "email")? {
        tx.execute("ALTER TABLE accounts ADD COLUMN email TEXT", [])?;
        tx.execute(
            "UPDATE accounts SET email = name WHERE email IS NULL OR email = ''",
            [],
        )?;
    }
    Ok(())
}

fn run_draft_migrations(tx: &rusqlite::Transaction) -> Result<(), AeroError> {
    let columns = [
        "bcc_addresses TEXT",
        "reply_context_json TEXT",
        "synced_at INTEGER",
        "remote_uid INTEGER",
    ];
    for column_def in &columns {
        let column_name = column_def.split_whitespace().next().unwrap_or(column_def);
        if !column_exists(tx, "drafts", column_name)? {
            tx.execute(&format!("ALTER TABLE drafts ADD COLUMN {column_def}"), [])?;
        }
    }
    Ok(())
}

fn run_mail_migrations(tx: &rusqlite::Transaction) -> Result<(), AeroError> {
    let columns = [
        "message_id TEXT",
        "is_archived INTEGER DEFAULT 0",
        "is_spam INTEGER DEFAULT 0",
    ];
    for column_def in &columns {
        let column_name = column_def.split_whitespace().next().unwrap_or(column_def);
        if !column_exists(tx, "mails", column_name)? {
            tx.execute(&format!("ALTER TABLE mails ADD COLUMN {column_def}"), [])?;
        }
    }
    Ok(())
}
