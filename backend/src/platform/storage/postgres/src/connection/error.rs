//! Errors safe to expose beyond the PostgreSQL driver boundary.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostgresAdapterErrorV1 {
    Connection,
    Query,
    Bootstrap,
    RoleBinding,
    SessionFence,
    LoginFence,
    LoginFenceUnauthorized,
    LoginFenceMissingRole,
    SchemaFence,
    OwnerPrivilegeFence,
    BackendTerminationFence,
    BackendDrainFence,
    Migration,
    MigrationLedgerRead,
    MigrationOwnerRole,
    MigrationStatement,
    MigrationResetRole,
    MigrationLedgerWrite,
    MigrationCommit,
    MigrationPrivileges,
}
