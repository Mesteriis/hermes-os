//! Errors returned before a migration reaches PostgreSQL.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationAdmissionErrorV1 {
    Owner,
    Syntax,
    Forbidden,
    OwnerScope,
}
