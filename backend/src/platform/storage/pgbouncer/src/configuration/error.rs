//! Invalid PgBouncer configuration fields.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolConfigErrorV1 {
    Identifier,
    Endpoint,
    ConnectionLimit,
    FileSystem,
}
