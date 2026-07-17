use rusqlite::{Connection, OptionalExtension, Transaction};

use crate::StoreError;

mod v01_to_v02;
mod v02_to_v03;
mod v03_to_v04;
mod v04_to_v05;
mod v05_to_v06;
mod v06_to_v07;
mod v07_to_v08;
mod v08_to_v09;
mod v09_to_v10;
mod v10_to_v11;
mod v11_to_v12;
mod v12_to_v13;
mod v13_to_v14;
mod v14_to_v15;

pub const SCHEMA_VERSION: i64 = 15;

pub fn migrate_schema(connection: &Connection) -> Result<(), StoreError> {
    loop {
        let version = schema_version(connection)?;
        if version == SCHEMA_VERSION {
            assert_schema_for_version(connection, version)?;
            return Ok(());
        }
        if !(1..SCHEMA_VERSION).contains(&version) {
            return Err(StoreError::UnsupportedSchema(version));
        }

        let transaction = connection.unchecked_transaction()?;
        apply_step(version, &transaction)?;
        let actual = schema_version(&transaction)?;
        let expected = version + 1;
        if actual != expected {
            return Err(StoreError::MigrationInvariant { expected, actual });
        }
        assert_schema_for_version(&transaction, actual)?;
        transaction.commit()?;
    }
}

fn assert_schema_for_version(connection: &Connection, version: i64) -> Result<(), StoreError> {
    if !(1..=SCHEMA_VERSION).contains(&version) || !base_schema_exists(connection)? {
        return Err(StoreError::MigrationSchemaAssertion { version });
    }
    for required_version in 2..=version {
        if !version_feature_exists(connection, required_version)? {
            return Err(StoreError::MigrationSchemaAssertion { version });
        }
    }
    Ok(())
}

fn base_schema_exists(connection: &Connection) -> Result<bool, StoreError> {
    for column in ["singleton", "schema_version", "instance_id", "generation"] {
        if !column_exists(connection, "hermes_kernel_control_store_metadata", column)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn version_feature_exists(connection: &Connection, version: i64) -> Result<bool, StoreError> {
    match version {
        2 => Ok(column_exists(
            connection,
            "hermes_kernel_control_store_metadata",
            "identity_epoch",
        )? && column_exists(
            connection,
            "hermes_kernel_control_store_metadata",
            "grant_epoch",
        )?),
        3 => table_exists(connection, "hermes_kernel_initial_owner_identity"),
        4 => table_exists(connection, "hermes_kernel_module_registration"),
        5 => table_exists(connection, "hermes_kernel_module_registration_capability"),
        6 => table_exists(connection, "hermes_kernel_external_runtime_attestation"),
        7 => table_exists(connection, "hermes_kernel_settings_schema_binding"),
        8 => table_exists(connection, "hermes_kernel_settings_desired_snapshot"),
        9 => Ok(column_exists(
            connection,
            "hermes_kernel_settings_schema_binding",
            "apply_state",
        )? && column_exists(
            connection,
            "hermes_kernel_settings_schema_binding",
            "sanitized_reason_code",
        )?),
        10 => table_exists(connection, "hermes_kernel_owner_pinned_artifact_binding"),
        11 => table_exists(connection, "hermes_kernel_settings_schema_artifact"),
        12 => table_exists(connection, "hermes_kernel_external_runtime_identity"),
        13 => table_exists(connection, "hermes_kernel_server_bootstrap_pairing"),
        14 => Ok(
            table_exists(connection, "hermes_kernel_bundled_managed_launch_binding")?
                && table_exists(connection, "hermes_kernel_managed_launch_record")?,
        ),
        15 => Ok(
            table_exists(connection, "hermes_kernel_platform_managed_process_binding")?
                && table_exists(connection, "hermes_kernel_platform_managed_process_launch")?,
        ),
        _ => Ok(false),
    }
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool, StoreError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?1)",
            [table],
            |row| row.get(0),
        )
        .map_err(StoreError::from)
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> Result<bool, StoreError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info(?1) WHERE name=?2)",
            [table, column],
            |row| row.get(0),
        )
        .map_err(StoreError::from)
}

fn schema_version(connection: &Connection) -> Result<i64, StoreError> {
    connection
        .query_row(
            "SELECT schema_version FROM hermes_kernel_control_store_metadata WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or(StoreError::MissingMetadata)
}

fn apply_step(version: i64, transaction: &Transaction<'_>) -> Result<(), StoreError> {
    match version {
        1 => v01_to_v02::apply(transaction),
        2 => v02_to_v03::apply(transaction),
        3 => v03_to_v04::apply(transaction),
        4 => v04_to_v05::apply(transaction),
        5 => v05_to_v06::apply(transaction),
        6 => v06_to_v07::apply(transaction),
        7 => v07_to_v08::apply(transaction),
        8 => v08_to_v09::apply(transaction),
        9 => v09_to_v10::apply(transaction),
        10 => v10_to_v11::apply(transaction),
        11 => v11_to_v12::apply(transaction),
        12 => v12_to_v13::apply(transaction),
        13 => v13_to_v14::apply(transaction),
        14 => v14_to_v15::apply(transaction),
        unsupported => Err(StoreError::UnsupportedSchema(unsupported)),
    }
}
