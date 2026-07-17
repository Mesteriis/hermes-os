#[path = "../../../../../../src/platform/telemetry/collector/src/storage/retention.rs"]
mod retention;
#[path = "../../../../../../src/platform/telemetry/collector/src/storage/segments.rs"]
mod segments;

pub(crate) use retention::TelemetryRetentionV1;
pub(crate) use segments::TelemetrySegmentStore;
