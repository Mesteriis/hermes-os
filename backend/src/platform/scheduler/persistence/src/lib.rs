//! Scheduler-owned PostgreSQL schema bundle; Storage Control applies it.

mod schema;
mod store;

pub use schema::scheduler_storage_bundle_v1;
pub use store::{
    FixedDelayCompletionOutcomeV1, RetryFailureOutcomeV1, SchedulerDispatchAdmissionV1,
    SchedulerDispatchClaimErrorV1, SchedulerDispatchClaimV1, SchedulerDueScheduleV1,
    SchedulerMaterializationErrorV1, SchedulerMaterializationOutcomeV1,
    SchedulerMaterializationSourceV1, SchedulerPendingFireErrorV1, SchedulerPendingFireOutcomeV1,
    SchedulerPendingFireV1, SchedulerPostgresEndpointV1, SchedulerPostgresStoreV1,
    SchedulerReceiptConsumeErrorV1, SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumerV1,
    SchedulerRunAcceptanceErrorV1, SchedulerRunAcceptanceOutcomeV1, SchedulerRunAcceptanceV1,
    SchedulerRunClaimErrorV1, SchedulerRunClaimV1, SchedulerRunTerminalResultErrorV1,
    SchedulerRunTerminalResultOutcomeV1, SchedulerRunTerminalResultV1,
    SchedulerScheduleStoreErrorV1, SchedulerScheduleUpsertOutcomeV1, SchedulerScheduleUpsertV1,
    SchedulerStoreConnectionErrorV1, scheduler_storage_binding_from_runtime,
};
