//! Durable waiting occurrences for bounded queue and coalescing policies.

mod claim;
mod request;
mod write;

pub use request::{
    SchedulerPendingFireErrorV1, SchedulerPendingFireOutcomeV1, SchedulerPendingFireV1,
};
pub(crate) use write::record_pending_locked;
