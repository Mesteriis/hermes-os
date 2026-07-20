//! PostgreSQL liveness/readiness query.

mod probe;

pub use probe::{PostgresReadinessV1, read_readiness};
