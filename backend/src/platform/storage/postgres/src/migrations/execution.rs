//! Per-step DDL and canonical ledger commits share one transaction.

use hermes_storage_migrations::admit_storage_bundle;
use hermes_storage_protocol::v1::StorageBundleV1;
use sqlx::{AssertSqlSafe, query, query_scalar, raw_sql};

use crate::{
    PostgresAdapterErrorV1, PostgresAdminConnectorV1, StorageRoleSpecV1,
    reconcile_owner_data_privileges,
};

pub async fn apply_storage_bundle(
    connector: &PostgresAdminConnectorV1,
    roles: &StorageRoleSpecV1,
    bundle: &StorageBundleV1,
) -> Result<(), PostgresAdapterErrorV1> {
    admit_storage_bundle(bundle).map_err(|_| PostgresAdapterErrorV1::Migration)?;
    if bundle.owner_id != roles.owner_id() {
        return Err(PostgresAdapterErrorV1::Migration);
    }
    for step in &bundle.steps {
        apply_step(connector, roles, bundle, step).await?;
    }
    reconcile_owner_data_privileges(connector, roles)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationPrivileges)
}

async fn apply_step(
    connector: &PostgresAdminConnectorV1,
    roles: &StorageRoleSpecV1,
    bundle: &StorageBundleV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let mut transaction = connector
        .pool()
        .begin()
        .await
        .map_err(|_| PostgresAdapterErrorV1::Migration)?;
    let recorded_digest = read_recorded_digest(&mut transaction, bundle, step)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationLedgerRead)?;
    if let Some(digest) = recorded_digest {
        if digest != step.sha256 {
            return Err(PostgresAdapterErrorV1::Migration);
        }
        transaction
            .commit()
            .await
            .map_err(|_| PostgresAdapterErrorV1::MigrationCommit)?;
        return Ok(());
    }
    execute_step_as_owner(&mut transaction, roles, step).await?;
    record_step(&mut transaction, bundle, step)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationLedgerWrite)?;
    transaction
        .commit()
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationCommit)
}

async fn read_recorded_digest(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bundle: &StorageBundleV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<Option<Vec<u8>>, PostgresAdapterErrorV1> {
    query_scalar::<_, Vec<u8>>(
        "SELECT step_digest FROM hermes_platform.storage_migration_ledger WHERE owner_id = $1 AND bundle_revision = $2 AND step_revision = $3",
    )
    .bind(&bundle.owner_id)
    .bind(bundle.revision as i32)
    .bind(step.revision as i32)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| PostgresAdapterErrorV1::Migration)
}

async fn execute_step_as_owner(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    roles: &StorageRoleSpecV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let set_role = format!("SET LOCAL ROLE {}", roles.ddl_owner());
    query(AssertSqlSafe(set_role))
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationOwnerRole)?;
    let sql = std::str::from_utf8(&step.forward_sql_utf8)
        .map_err(|_| PostgresAdapterErrorV1::MigrationStatement)?;
    raw_sql(AssertSqlSafe(sql.to_owned()))
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationStatement)?;
    query("RESET ROLE")
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationResetRole)?;
    Ok(())
}

async fn record_step(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bundle: &StorageBundleV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<(), PostgresAdapterErrorV1> {
    query("INSERT INTO hermes_platform.storage_migration_ledger (owner_id, bundle_revision, step_revision, step_digest) VALUES ($1, $2, $3, $4)")
        .bind(&bundle.owner_id)
        .bind(bundle.revision as i32)
        .bind(step.revision as i32)
        .bind(&step.sha256)
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::Migration)?;
    Ok(())
}
