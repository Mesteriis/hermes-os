//! Scheduler-owned exact-byte dispatch records.

mod read;
mod relay;
mod request;
mod write;

pub use request::{SchedulerDispatchClaimErrorV1, SchedulerDispatchClaimV1};
pub(crate) use write::insert_dispatch;
