use std::fs::File;
use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, backup::Backup};

use crate::database::connection::{configure, create_keyed_connection, open_keyed_connection};
use crate::database::store::VaultStoreError;

pub(crate) fn export_database_snapshot(
    source_path: &Path,
    destination_path: &Path,
    sqlcipher_key: &[u8; 32],
    expected_instance_id: &str,
) -> Result<(), VaultStoreError> {
    let source = open_keyed_connection(source_path, sqlcipher_key)?;
    configure(&source)?;
    validate_database(&source, expected_instance_id)?;
    let mut destination = create_keyed_connection(destination_path, sqlcipher_key)?;
    configure(&destination)?;
    Backup::new(&source, &mut destination)
        .and_then(|backup| backup.run_to_completion(128, Duration::ZERO, None))
        .map_err(VaultStoreError::Sqlite)?;
    validate_database(&destination, expected_instance_id)?;
    drop(destination);
    File::open(destination_path)
        .and_then(|file| file.sync_all())
        .map_err(|_| VaultStoreError::Backup)
}

pub(crate) fn validate_database(
    connection: &Connection,
    expected_instance_id: &str,
) -> Result<(), VaultStoreError> {
    let instance_id: String = connection
        .query_row(
            "SELECT instance_id FROM vault_metadata WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(VaultStoreError::Sqlite)?;
    if instance_id != expected_instance_id {
        return Err(VaultStoreError::Backup);
    }
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(VaultStoreError::Sqlite)?;
    (integrity == "ok")
        .then_some(())
        .ok_or(VaultStoreError::Backup)
}
