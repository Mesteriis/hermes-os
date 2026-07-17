//! Bounded, private telemetry segment storage.

mod retention;
mod segments;

pub use retention::TelemetryRetentionV1;
pub use segments::TelemetrySegmentStore;
