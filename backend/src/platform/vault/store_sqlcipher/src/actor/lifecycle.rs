//! Atomic active-record to tombstone lifecycle transitions.

use rusqlite::{Connection, OptionalExtension, Transaction};

use crate::database::store::{VaultStoreError, VaultStoreResult};
use crate::records::secret::SecretRecordScope;

#[derive(Clone, Copy)]
pub(super) enum SecretLifecycleMutation {
    Retire,
    Delete,
}

pub(super) fn mutate(
    connection: &mut Connection,
    scope: &SecretRecordScope,
    changed_at_unix_seconds: u64,
    mutation: SecretLifecycleMutation,
) -> VaultStoreResult<()> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    apply(&transaction, scope, changed_at_unix_seconds, mutation)?;
    transaction.commit().map_err(VaultStoreError::Sqlite)
}

pub(super) fn apply(
    transaction: &Transaction<'_>,
    scope: &SecretRecordScope,
    changed_at_unix_seconds: u64,
    mutation: SecretLifecycleMutation,
) -> VaultStoreResult<()> {
    let changed_at_unix_seconds =
        i64::try_from(changed_at_unix_seconds).map_err(|_| VaultStoreError::LifecycleConflict)?;
    if changed_at_unix_seconds <= 0 {
        return Err(VaultStoreError::LifecycleConflict);
    }
    let (owner, configuration, purpose, class, revision) = scope.metadata();
    let deleted = transaction
        .execute(
            "DELETE FROM vault_secret_records
             WHERE logical_owner_id = ?1 AND configuration_instance_id = ?2 AND purpose_id = ?3
               AND secret_class = ?4 AND secret_revision = ?5",
            rusqlite::params![owner, configuration, purpose, class, revision],
        )
        .map_err(VaultStoreError::Sqlite)?;
    match mutation {
        SecretLifecycleMutation::Retire if deleted == 1 => {
            transaction
                .execute(
                    "INSERT INTO vault_secret_tombstones (
                        logical_owner_id, configuration_instance_id, purpose_id,
                        secret_class, secret_revision, state, changed_at_unix_seconds
                     ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                    rusqlite::params![
                        owner,
                        configuration,
                        purpose,
                        class,
                        revision,
                        changed_at_unix_seconds,
                    ],
                )
                .map_err(|_| VaultStoreError::LifecycleConflict)?;
        }
        SecretLifecycleMutation::Delete if deleted == 1 => {
            transaction
                .execute(
                    "INSERT INTO vault_secret_tombstones (
                        logical_owner_id, configuration_instance_id, purpose_id,
                        secret_class, secret_revision, state, changed_at_unix_seconds
                     ) VALUES (?1, ?2, ?3, ?4, ?5, 2, ?6)",
                    rusqlite::params![
                        owner,
                        configuration,
                        purpose,
                        class,
                        revision,
                        changed_at_unix_seconds,
                    ],
                )
                .map_err(|_| VaultStoreError::LifecycleConflict)?;
        }
        SecretLifecycleMutation::Delete if deleted == 0 => {
            let updated = transaction
                .execute(
                    "UPDATE vault_secret_tombstones
                     SET state = 2, changed_at_unix_seconds = ?1
                     WHERE logical_owner_id = ?2 AND configuration_instance_id = ?3
                       AND purpose_id = ?4 AND secret_class = ?5 AND secret_revision = ?6
                       AND state = 1",
                    rusqlite::params![
                        changed_at_unix_seconds,
                        owner,
                        configuration,
                        purpose,
                        class,
                        revision,
                    ],
                )
                .map_err(VaultStoreError::Sqlite)?;
            if updated == 0
                && tombstone_state(transaction, owner, configuration, purpose, class, revision)?
                    != Some(2)
            {
                return Err(VaultStoreError::LifecycleConflict);
            }
        }
        SecretLifecycleMutation::Retire if deleted == 0 => {
            if tombstone_state(transaction, owner, configuration, purpose, class, revision)?
                != Some(1)
            {
                return Err(VaultStoreError::LifecycleConflict);
            }
        }
        _ => return Err(VaultStoreError::LifecycleConflict),
    }
    Ok(())
}

fn tombstone_state(
    transaction: &Transaction<'_>,
    owner: &str,
    configuration: &str,
    purpose: &str,
    class: i64,
    revision: i64,
) -> VaultStoreResult<Option<i64>> {
    transaction
        .query_row(
            "SELECT state FROM vault_secret_tombstones
             WHERE logical_owner_id = ?1 AND configuration_instance_id = ?2
               AND purpose_id = ?3 AND secret_class = ?4 AND secret_revision = ?5",
            rusqlite::params![owner, configuration, purpose, class, revision],
            |row| row.get(0),
        )
        .optional()
        .map_err(VaultStoreError::Sqlite)
}
