//! Deterministic PgBouncer alias fencing order.

use super::PoolLifecycleCommandV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolRevokePlanV1;

impl PoolRevokePlanV1 {
    pub const fn commands() -> [PoolLifecycleCommandV1; 3] {
        [
            PoolLifecycleCommandV1::Pause,
            PoolLifecycleCommandV1::Disable,
            PoolLifecycleCommandV1::Kill,
        ]
    }
}
