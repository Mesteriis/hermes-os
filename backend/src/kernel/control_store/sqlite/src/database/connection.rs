//! SQLite connection configuration and integrity validation.

use rusqlite::Connection;

use crate::StoreError;

pub(crate) fn configure_writable(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        "PRAGMA journal_mode = DELETE;
         PRAGMA synchronous = FULL;
         PRAGMA foreign_keys = ON;
         PRAGMA trusted_schema = OFF;",
    )
}

pub(crate) fn validate_quick_check(connection: &Connection) -> Result<(), StoreError> {
    let result: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if result == "ok" {
        Ok(())
    } else {
        Err(StoreError::IntegrityCheckFailed(result))
    }
}
