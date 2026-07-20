//! Sanitized role-state audit used by Storage Control reconciliation.

use sqlx::query_as;

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

use super::StorageRoleSpecV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageRoleAuditV1 {
    pub can_login: bool,
    pub inherits_privileges: bool,
    pub can_create_database: bool,
    pub can_create_roles: bool,
    pub is_superuser: bool,
    pub bypasses_row_security: bool,
    pub connection_limit: i32,
    pub search_path_isolated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageDataPrivilegeAuditV1 {
    pub owner_table_count: u64,
    pub owner_tables_owned_by_ddl: bool,
    pub owner_tables_have_dml: bool,
    pub foreign_tables_with_dml: u64,
}

pub async fn read_storage_role_audit(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<StorageRoleAuditV1, PostgresAdapterErrorV1> {
    let row = query_as::<_, (bool, bool, bool, bool, bool, bool, i32, bool)>(
        "SELECT rolcanlogin, rolinherit, rolcreatedb, rolcreaterole, rolsuper, rolbypassrls, rolconnlimit, \
                EXISTS(SELECT 1 FROM unnest(COALESCE(rolconfig, ARRAY[]::text[])) setting \
                       WHERE setting = 'search_path=pg_catalog') \
         FROM pg_roles WHERE rolname = $1",
    )
    .bind(spec.runtime_principal())
    .fetch_optional(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::Query)?
    .ok_or(PostgresAdapterErrorV1::Query)?;

    Ok(StorageRoleAuditV1 {
        can_login: row.0,
        inherits_privileges: row.1,
        can_create_database: row.2,
        can_create_roles: row.3,
        is_superuser: row.4,
        bypasses_row_security: row.5,
        connection_limit: row.6,
        search_path_isolated: row.7,
    })
}

pub async fn read_storage_data_privilege_audit(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<StorageDataPrivilegeAuditV1, PostgresAdapterErrorV1> {
    let row = query_as::<_, (i64, i64, i64, i64)>(
        "SELECT \
             COUNT(*) FILTER (WHERE LEFT(relation.relname, char_length($1) + 1) = ($1 || '_')), \
             COUNT(*) FILTER (WHERE LEFT(relation.relname, char_length($1) + 1) = ($1 || '_') \
                 AND pg_get_userbyid(relation.relowner) = $3), \
             COUNT(*) FILTER (WHERE LEFT(relation.relname, char_length($1) + 1) = ($1 || '_') \
                 AND has_table_privilege($2, relation.oid, \
                     'SELECT, INSERT, UPDATE, DELETE')), \
             COUNT(*) FILTER (WHERE LEFT(relation.relname, char_length($1) + 1) <> ($1 || '_') \
                 AND has_table_privilege($2, relation.oid, \
                     'SELECT, INSERT, UPDATE, DELETE')) \
         FROM pg_catalog.pg_class relation \
         JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace \
         WHERE namespace.nspname = $4 AND relation.relkind = 'r'",
    )
    .bind(spec.owner_id())
    .bind(spec.runtime_principal())
    .bind(spec.ddl_owner())
    .bind(spec.storage_schema())
    .fetch_one(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::Query)?;
    Ok(StorageDataPrivilegeAuditV1 {
        owner_table_count: row.0 as u64,
        owner_tables_owned_by_ddl: row.0 == row.1,
        owner_tables_have_dml: row.0 == row.2,
        foreign_tables_with_dml: row.3 as u64,
    })
}
