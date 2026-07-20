//! Owner-local data grants reconciled from PostgreSQL's authoritative catalog.

use sqlx::{query, query_scalar};

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

use super::StorageRoleSpecV1;

pub async fn reconcile_owner_data_privileges(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    for table_name in owner_table_names(connector, spec).await? {
        grant_table_dml(connector, spec, &table_name).await?;
    }
    for sequence_name in owner_sequence_names(connector, spec).await? {
        grant_sequence_usage(connector, spec, &sequence_name).await?;
    }
    Ok(())
}

pub(crate) async fn revoke_owner_data_privileges(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    for table_name in owner_table_names(connector, spec).await? {
        revoke_table_dml(connector, spec, &table_name).await?;
    }
    for sequence_name in owner_sequence_names(connector, spec).await? {
        revoke_sequence_usage(connector, spec, &sequence_name).await?;
    }
    Ok(())
}

async fn owner_table_names(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<Vec<String>, PostgresAdapterErrorV1> {
    owner_relation_names(connector, spec, "r").await
}

async fn owner_sequence_names(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<Vec<String>, PostgresAdapterErrorV1> {
    owner_relation_names(connector, spec, "S").await
}

async fn owner_relation_names(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    relation_kind: &str,
) -> Result<Vec<String>, PostgresAdapterErrorV1> {
    query_scalar::<_, String>(
        "SELECT relation.relname \
         FROM pg_catalog.pg_class relation \
         JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace \
         WHERE namespace.nspname = $1 AND relation.relkind = $2 \
             AND pg_get_userbyid(relation.relowner) = $3 \
             AND LEFT(relation.relname, char_length($4) + 1) = ($4 || '_') \
         ORDER BY relation.relname",
    )
    .bind(spec.storage_schema())
    .bind(relation_kind)
    .bind(spec.ddl_owner())
    .bind(spec.owner_id())
    .fetch_all(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::Query)
}

async fn grant_table_dml(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    table_name: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = format!(
        "GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE {}.{table_name} TO {}",
        spec.storage_schema(),
        spec.runtime_principal(),
    );
    execute_grant(connector, &statement).await
}

async fn grant_sequence_usage(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    sequence_name: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = format!(
        "GRANT USAGE, SELECT ON SEQUENCE {}.{sequence_name} TO {}",
        spec.storage_schema(),
        spec.runtime_principal(),
    );
    execute_grant(connector, &statement).await
}

async fn revoke_table_dml(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    table_name: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = format!(
        "REVOKE SELECT, INSERT, UPDATE, DELETE ON TABLE {}.{table_name} FROM {}",
        spec.storage_schema(),
        spec.runtime_principal(),
    );
    execute_grant(connector, &statement).await
}

async fn revoke_sequence_usage(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    sequence_name: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = format!(
        "REVOKE USAGE, SELECT ON SEQUENCE {}.{sequence_name} FROM {}",
        spec.storage_schema(),
        spec.runtime_principal(),
    );
    execute_grant(connector, &statement).await
}

async fn execute_grant(
    connector: &PostgresAdminConnectorV1,
    statement: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    query(statement)
        .execute(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    Ok(())
}
