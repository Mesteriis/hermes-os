//! Adds the non-secret Event Hub broker identity to desired topology.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE hermes_kernel_platform_event_hub_topology
           ADD COLUMN nats_endpoint TEXT NOT NULL DEFAULT '';
         ALTER TABLE hermes_kernel_platform_event_hub_topology
           ADD COLUMN nats_username TEXT NOT NULL DEFAULT '';
         ALTER TABLE hermes_kernel_platform_event_hub_topology
           ADD COLUMN credential_revision INTEGER NOT NULL DEFAULT 0;
         UPDATE hermes_kernel_control_store_metadata SET schema_version = 28 WHERE singleton = 1;",
    )?;
    Ok(())
}
