use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE hermes_kernel_module_registration_capability (
            registration_id TEXT NOT NULL REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE,
            capability_id TEXT NOT NULL,
            approved INTEGER NOT NULL CHECK (approved IN (0, 1)),
            PRIMARY KEY (registration_id, capability_id)
        ) STRICT;
        UPDATE hermes_kernel_control_store_metadata SET schema_version = 5 WHERE singleton = 1;",
    )?;
    Ok(())
}
