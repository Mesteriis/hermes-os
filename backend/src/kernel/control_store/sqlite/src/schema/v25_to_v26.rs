//! Adds durable public configuration for the managed Events authority.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE hermes_kernel_platform_events_authority_configuration (
             singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
             revision INTEGER NOT NULL CHECK (revision >= 1),
             account_public_key TEXT NOT NULL,
             signer_credential_revision INTEGER NOT NULL CHECK (signer_credential_revision >= 1)
         );
         UPDATE hermes_kernel_control_store_metadata SET schema_version = 26 WHERE singleton = 1;",
    )?;
    Ok(())
}
