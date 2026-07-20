//! Binds each browser identity to a distinct local browser signing key.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE hermes_kernel_browser_device_identity
             ADD COLUMN browser_key_public_key BLOB;
         UPDATE hermes_kernel_control_store_metadata SET schema_version = 32 WHERE singleton = 1;",
    )?;
    Ok(())
}
