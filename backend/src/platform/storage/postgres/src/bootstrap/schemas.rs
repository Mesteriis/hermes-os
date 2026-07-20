//! Idempotent DDL for the three schemas owned by Storage Control.

use sqlx::query;

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

const CREATE_HERMES_DATA: &str = "CREATE SCHEMA IF NOT EXISTS hermes_data";
const CREATE_HERMES_PLATFORM: &str = "CREATE SCHEMA IF NOT EXISTS hermes_platform";
const CREATE_HERMES_EXTENSIONS: &str = "CREATE SCHEMA IF NOT EXISTS hermes_extensions";
const REVOKE_PUBLIC_CREATE: &str =
    "REVOKE CREATE ON SCHEMA hermes_data, hermes_platform, hermes_extensions FROM PUBLIC";
const CREATE_MIGRATION_LEDGER: &str = "CREATE TABLE IF NOT EXISTS hermes_platform.storage_migration_ledger (owner_id text NOT NULL, bundle_revision integer NOT NULL, step_revision integer NOT NULL, step_digest bytea NOT NULL, applied_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP, PRIMARY KEY (owner_id, bundle_revision, step_revision))";
const CREATE_ROLE_LEDGER: &str = "CREATE TABLE IF NOT EXISTS hermes_platform.storage_role_ledger (owner_id text PRIMARY KEY, ddl_owner text NOT NULL UNIQUE, runtime_principal text NOT NULL UNIQUE, registration_id text, runtime_instance_id text, storage_generation bigint, runtime_generation bigint, grant_epoch bigint, role_epoch bigint, credential_lease_revision bigint, storage_bundle_revision bigint, updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP)";
const ADD_ROLE_LEDGER_REGISTRATION: &str =
    "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS registration_id text";
const ADD_ROLE_LEDGER_RUNTIME: &str = "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS runtime_instance_id text";
const ADD_ROLE_LEDGER_STORAGE_GENERATION: &str = "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS storage_generation bigint";
const ADD_ROLE_LEDGER_RUNTIME_GENERATION: &str = "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS runtime_generation bigint";
const ADD_ROLE_LEDGER_GRANT_EPOCH: &str =
    "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS grant_epoch bigint";
const ADD_ROLE_LEDGER_ROLE_EPOCH: &str =
    "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS role_epoch bigint";
const ADD_ROLE_LEDGER_LEASE_REVISION: &str = "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS credential_lease_revision bigint";
const ADD_ROLE_LEDGER_BUNDLE_REVISION: &str = "ALTER TABLE hermes_platform.storage_role_ledger ADD COLUMN IF NOT EXISTS storage_bundle_revision bigint";

pub async fn ensure_platform_schemas(
    connector: &PostgresAdminConnectorV1,
) -> Result<(), PostgresAdapterErrorV1> {
    for statement in [
        CREATE_HERMES_DATA,
        CREATE_HERMES_PLATFORM,
        CREATE_HERMES_EXTENSIONS,
        REVOKE_PUBLIC_CREATE,
        CREATE_MIGRATION_LEDGER,
        CREATE_ROLE_LEDGER,
        ADD_ROLE_LEDGER_REGISTRATION,
        ADD_ROLE_LEDGER_RUNTIME,
        ADD_ROLE_LEDGER_STORAGE_GENERATION,
        ADD_ROLE_LEDGER_RUNTIME_GENERATION,
        ADD_ROLE_LEDGER_GRANT_EPOCH,
        ADD_ROLE_LEDGER_ROLE_EPOCH,
        ADD_ROLE_LEDGER_LEASE_REVISION,
        ADD_ROLE_LEDGER_BUNDLE_REVISION,
    ] {
        query(statement)
            .execute(connector.pool())
            .await
            .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    }
    Ok(())
}
