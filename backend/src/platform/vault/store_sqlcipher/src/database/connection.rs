//! SQLCipher connection creation and fixed SQLite safety configuration.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use rusqlite::Connection;

use crate::database::store::VaultStoreError;

pub(crate) fn open_keyed_connection(
    path: &Path,
    key: &[u8; 32],
) -> Result<Connection, VaultStoreError> {
    let connection = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(VaultStoreError::Sqlite)?;
    apply_raw_key(&connection, key)?;
    Ok(connection)
}

pub(crate) fn create_keyed_connection(
    path: &Path,
    key: &[u8; 32],
) -> Result<Connection, VaultStoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| VaultStoreError::InsecurePath)?;
    file.write_all(&[])
        .and_then(|_| file.sync_all())
        .map_err(|_| VaultStoreError::InsecurePath)?;
    drop(file);
    open_keyed_connection(path, key)
}

pub(crate) fn configure(connection: &Connection) -> Result<(), VaultStoreError> {
    connection
        .execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;
             PRAGMA foreign_keys = ON;
             PRAGMA trusted_schema = OFF;
             PRAGMA temp_store = MEMORY;
             PRAGMA secure_delete = ON;",
        )
        .map_err(VaultStoreError::Sqlite)
}

fn apply_raw_key(connection: &Connection, key: &[u8; 32]) -> Result<(), VaultStoreError> {
    // The SQLCipher FFI accepts the fixed 32-byte derived key before any SQL executes.
    let result = unsafe {
        rusqlite::ffi::sqlite3_key(
            connection.handle(),
            key.as_ptr().cast(),
            i32::try_from(key.len()).expect("fixed Vault key length fits i32"),
        )
    };
    if result == rusqlite::ffi::SQLITE_OK {
        Ok(())
    } else {
        Err(VaultStoreError::InsecurePath)
    }
}
