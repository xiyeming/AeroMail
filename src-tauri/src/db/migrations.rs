use rusqlite::Connection;

use super::schema::ALL_SCHEMAS;
use crate::error::AeroError;

pub fn run_migrations(conn: &mut Connection) -> Result<(), AeroError> {
    let tx = conn.transaction()?;
    for schema in ALL_SCHEMAS {
        tx.execute_batch(schema)?;
    }
    tx.commit()?;
    Ok(())
}
