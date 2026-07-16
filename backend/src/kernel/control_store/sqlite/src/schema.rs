use rusqlite::{Connection, OptionalExtension};

use crate::StoreError;

pub const SCHEMA_VERSION: i64 = 9;

pub fn migrate_schema(connection: &Connection) -> Result<(), StoreError> {
    let schema_version = connection
        .query_row(
            "SELECT schema_version FROM hermes_kernel_control_store_metadata WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or(StoreError::MissingMetadata)?;
    match schema_version {
        SCHEMA_VERSION => Ok(()),
        2 => {
            connection.execute_batch("CREATE TABLE hermes_kernel_initial_owner_identity (singleton INTEGER PRIMARY KEY CHECK (singleton = 1), owner_id TEXT NOT NULL, device_id TEXT NOT NULL, public_key_sec1 BLOB NOT NULL CHECK (length(public_key_sec1) = 65)) STRICT; UPDATE hermes_kernel_control_store_metadata SET schema_version = 3 WHERE singleton = 1;")?;
            Ok(())
        }
        3 => {
            connection.execute_batch("CREATE TABLE hermes_kernel_module_registration (registration_id TEXT PRIMARY KEY, module_id TEXT NOT NULL, owner_id TEXT NOT NULL, descriptor_sha256 BLOB NOT NULL CHECK (length(descriptor_sha256) = 32), state TEXT NOT NULL CHECK (state IN ('pending', 'approved', 'suspended', 'revoked', 'blocked_incompatible')), grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1)) STRICT; UPDATE hermes_kernel_control_store_metadata SET schema_version = 4 WHERE singleton = 1;")?;
            Ok(())
        }
        4 => {
            connection.execute_batch("CREATE TABLE hermes_kernel_module_registration_capability (registration_id TEXT NOT NULL REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE, capability_id TEXT NOT NULL, approved INTEGER NOT NULL CHECK (approved IN (0, 1)), PRIMARY KEY (registration_id, capability_id)) STRICT; UPDATE hermes_kernel_control_store_metadata SET schema_version = 5 WHERE singleton = 1;")?;
            Ok(())
        }
        5 => {
            connection.execute_batch("CREATE TABLE hermes_kernel_external_runtime_attestation (registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE, runtime_id TEXT NOT NULL, runtime_generation INTEGER NOT NULL CHECK (runtime_generation >= 1), grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1), distribution_sha256 BLOB NOT NULL CHECK (length(distribution_sha256) = 32)) STRICT; UPDATE hermes_kernel_control_store_metadata SET schema_version = 6 WHERE singleton = 1;")?;
            Ok(())
        }
        6 => {
            connection.execute_batch("CREATE TABLE hermes_kernel_settings_schema_binding (registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_module_registration(registration_id) ON DELETE CASCADE, schema_major INTEGER NOT NULL CHECK (schema_major >= 1), schema_revision INTEGER NOT NULL CHECK (schema_revision >= 1), schema_sha256 BLOB NOT NULL CHECK (length(schema_sha256) = 32), desired_revision INTEGER NOT NULL CHECK (desired_revision >= 0), effective_revision INTEGER NOT NULL CHECK (effective_revision >= 0)) STRICT; UPDATE hermes_kernel_control_store_metadata SET schema_version = 7 WHERE singleton = 1;")?;
            Ok(())
        }
        7 => {
            connection.execute_batch("CREATE TABLE hermes_kernel_settings_desired_snapshot (registration_id TEXT PRIMARY KEY REFERENCES hermes_kernel_settings_schema_binding(registration_id) ON DELETE CASCADE, revision INTEGER NOT NULL CHECK (revision >= 1), snapshot_bytes BLOB NOT NULL) STRICT; UPDATE hermes_kernel_control_store_metadata SET schema_version = 8 WHERE singleton = 1;")?;
            Ok(())
        }
        8 => {
            connection.execute_batch("ALTER TABLE hermes_kernel_settings_schema_binding ADD COLUMN apply_state TEXT NOT NULL DEFAULT 'current' CHECK (apply_state IN ('current', 'pending_validation', 'pending_apply', 'applying', 'awaiting_external_restart', 'blocked_config')); ALTER TABLE hermes_kernel_settings_schema_binding ADD COLUMN sanitized_reason_code TEXT; UPDATE hermes_kernel_settings_schema_binding SET apply_state=CASE WHEN desired_revision=effective_revision THEN 'current' ELSE 'pending_validation' END; UPDATE hermes_kernel_control_store_metadata SET schema_version = 9 WHERE singleton = 1;")?;
            Ok(())
        }
        1 => {
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "
                ALTER TABLE hermes_kernel_control_store_metadata
                    ADD COLUMN identity_epoch INTEGER NOT NULL DEFAULT 1 CHECK (identity_epoch >= 1);
                ALTER TABLE hermes_kernel_control_store_metadata
                    ADD COLUMN grant_epoch INTEGER NOT NULL DEFAULT 1 CHECK (grant_epoch >= 1);
                UPDATE hermes_kernel_control_store_metadata SET schema_version = 2 WHERE singleton = 1;
                ",
            )?;
            transaction.commit()?;
            Ok(())
        }
        version => Err(StoreError::UnsupportedSchema(version)),
    }
}
