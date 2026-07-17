//! SQLCipher copy and record-envelope re-encryption for root-key rotation.

use std::fs::File;
use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, backup::Backup};

use crate::database::connection::{configure, create_keyed_connection, open_keyed_connection};
use crate::database::store::VaultStoreError;
use crate::records::secret as secret_record;

pub(super) fn stage(
    source_path: &Path,
    staged_path: &Path,
    current_sql_key: &[u8; 32],
    next_sql_key: &[u8; 32],
    current_record_key: &[u8; 32],
    next_record_key: &[u8; 32],
) -> Result<(), VaultStoreError> {
    let source = open_keyed_connection(source_path, current_sql_key)?;
    configure(&source)?;
    let mut staged = create_keyed_connection(staged_path, next_sql_key)?;
    configure(&staged)?;
    Backup::new(&source, &mut staged)
        .and_then(|backup| backup.run_to_completion(128, Duration::ZERO, None))
        .map_err(VaultStoreError::Sqlite)?;
    reencrypt_records(&mut staged, current_record_key, next_record_key)?;
    validate_integrity(&staged)?;
    drop(staged);
    File::open(staged_path)
        .and_then(|file| file.sync_all())
        .map_err(|_| VaultStoreError::InsecurePath)
}

fn reencrypt_records(
    connection: &mut Connection,
    current_record_key: &[u8; 32],
    next_record_key: &[u8; 32],
) -> Result<(), VaultStoreError> {
    let records = read_records(connection)?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    for record in records {
        let encrypted = secret_record::reencrypt_stored_record(
            &record.record_id,
            &record.owner,
            &record.configuration,
            &record.purpose,
            record.class,
            record.revision,
            record.key_epoch,
            &record.nonce,
            &record.ciphertext,
            current_record_key,
            next_record_key,
        )
        .map_err(VaultStoreError::Record)?;
        transaction
            .execute(
                "UPDATE vault_secret_records SET nonce = ?1, ciphertext = ?2 WHERE record_id = ?3",
                rusqlite::params![
                    encrypted.nonce.as_slice(),
                    encrypted.ciphertext,
                    record.record_id,
                ],
            )
            .map_err(VaultStoreError::Sqlite)?;
    }
    transaction.commit().map_err(VaultStoreError::Sqlite)
}

fn read_records(connection: &Connection) -> Result<Vec<StoredRecord>, VaultStoreError> {
    let mut statement = connection
        .prepare(
            "SELECT record_id, logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision, key_epoch, nonce, ciphertext
             FROM vault_secret_records ORDER BY record_id",
        )
        .map_err(VaultStoreError::Sqlite)?;
    statement
        .query_map([], |row| {
            Ok(StoredRecord {
                record_id: row.get(0)?,
                owner: row.get(1)?,
                configuration: row.get(2)?,
                purpose: row.get(3)?,
                class: row.get(4)?,
                revision: row.get(5)?,
                key_epoch: row.get(6)?,
                nonce: row.get(7)?,
                ciphertext: row.get(8)?,
            })
        })
        .map_err(VaultStoreError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(VaultStoreError::Sqlite)
}

fn validate_integrity(connection: &Connection) -> Result<(), VaultStoreError> {
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(VaultStoreError::Sqlite)?;
    if integrity == "ok" {
        Ok(())
    } else {
        Err(VaultStoreError::InsecurePath)
    }
}

struct StoredRecord {
    record_id: Vec<u8>,
    owner: String,
    configuration: String,
    purpose: String,
    class: i64,
    revision: i64,
    key_epoch: i64,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}
