//! Fenced PostgreSQL claim adapter for durable time-triggered runs.

mod concurrency;
mod connection;
mod dispatch;
mod materialization;
mod materialization_contract;
mod pending;
mod receipts;
mod recovery;
mod runs;
mod schedules;

pub use connection::{
    SchedulerPostgresEndpointV1, SchedulerRecoveryDatabaseV1, SchedulerStoreConnectionErrorV1,
    scheduler_storage_binding_from_runtime,
};
pub use dispatch::{SchedulerDispatchClaimErrorV1, SchedulerDispatchClaimV1};
pub use materialization_contract::{
    SchedulerDispatchAdmissionV1, SchedulerMaterializationErrorV1,
    SchedulerMaterializationOutcomeV1, SchedulerMaterializationSourceV1,
};
pub use pending::{
    SchedulerPendingFireErrorV1, SchedulerPendingFireOutcomeV1, SchedulerPendingFireV1,
};
pub use receipts::{
    SchedulerReceiptConsumeErrorV1, SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumerV1,
    SchedulerRunAcceptanceErrorV1, SchedulerRunAcceptanceOutcomeV1, SchedulerRunAcceptanceV1,
    SchedulerRunTerminalResultErrorV1, SchedulerRunTerminalResultOutcomeV1,
    SchedulerRunTerminalResultV1,
};
pub use recovery::{SchedulerRecoveryErrorV1, SchedulerRecoveryReplayReportV1};
pub use runs::{
    FixedDelayCompletionOutcomeV1, RetryFailureOutcomeV1, SchedulerPostgresStoreV1,
    SchedulerRunClaimErrorV1, SchedulerRunClaimV1,
};
pub use schedules::{
    SchedulerDueScheduleV1, SchedulerScheduleStoreErrorV1, SchedulerScheduleUpsertOutcomeV1,
    SchedulerScheduleUpsertV1,
};
