//! Versioned, owner-neutral Scheduler contracts.

mod contracts;
mod transport;
pub mod validation;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/hermes.scheduler.v1.rs"));
}

pub use contracts::command::{ScheduledJobCommandBuildErrorV1, build_scheduled_job_command_v1};
pub use contracts::job::{JobContractBindingV1, JobKindErrorV1, JobKindV1};
pub use contracts::run::{JobRunErrorV1, JobRunIdV1, ScheduleRunLeaseV1};
pub use contracts::schedule::{
    ConcurrencyKeyV1, MisfirePolicyV1, OpaqueScheduleScopeV1, OverlapPolicyV1, RetryPolicyV1,
    ScheduleCodecErrorV1, ScheduleErrorV1, ScheduleIdV1, SchedulePolicyV1, ScheduleRevisionV1,
    ScheduleSpecV1, ScheduleTriggerV1,
};
pub use transport::{
    SchedulerReceiptDeliveryErrorV1, SchedulerReceiptDeliveryPortV1, SchedulerReceiptDeliveryV1,
};
pub use validation::{
    SchedulerCommandValidationErrorV1, SchedulerReceiptValidationErrorV1,
    validate_job_run_receipt_v1, validate_scheduled_job_command_v1,
};
