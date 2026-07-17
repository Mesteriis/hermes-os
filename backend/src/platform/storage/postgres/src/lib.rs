//! PostgreSQL adapter boundary; no driver is linked until the managed artifact exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostgresAdapterStateV1 {
    Unconfigured,
    Ready,
}
