use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE hermes_kernel_external_runtime_identity (
            registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE,
            public_key_sec1 BLOB NOT NULL UNIQUE CHECK (length(public_key_sec1) = 65)
        ) STRICT;
        UPDATE hermes_kernel_control_store_metadata SET schema_version = 12 WHERE singleton = 1;",
    )?;
    Ok(())
}
