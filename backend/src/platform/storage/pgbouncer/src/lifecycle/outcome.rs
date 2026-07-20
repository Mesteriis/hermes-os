//! Sanitized PgBouncer admin outcome for lifecycle orchestration.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolLifecycleOutcomeV1 {
    Applied,
    Rejected,
    Unavailable,
}
