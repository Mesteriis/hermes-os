use crate::v1::{JobRunOutcomeV1, JobRunReceiptV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReceiptValidationErrorV1 {
    InvalidRun,
    InvalidCommand,
    InvalidLease,
    InvalidOutcome,
    InvalidObservedAt,
}

pub fn validate_job_run_receipt_v1(
    receipt: &JobRunReceiptV1,
) -> Result<(), SchedulerReceiptValidationErrorV1> {
    (receipt.job_run_id.len() == 16)
        .then_some(())
        .ok_or(SchedulerReceiptValidationErrorV1::InvalidRun)?;
    (receipt.command_message_id.len() == 16)
        .then_some(())
        .ok_or(SchedulerReceiptValidationErrorV1::InvalidCommand)?;
    let lease = valid_lease(receipt)?;
    JobRunOutcomeV1::try_from(receipt.outcome)
        .ok()
        .filter(|outcome| *outcome != JobRunOutcomeV1::Unspecified)
        .map(|_| ())
        .ok_or(SchedulerReceiptValidationErrorV1::InvalidOutcome)?;
    (receipt.observed_at_unix_millis < lease.expires_at_unix_millis)
        .then_some(())
        .ok_or(SchedulerReceiptValidationErrorV1::InvalidObservedAt)
}

fn valid_lease(
    receipt: &JobRunReceiptV1,
) -> Result<&crate::v1::JobLeaseV1, SchedulerReceiptValidationErrorV1> {
    receipt
        .lease
        .as_ref()
        .filter(|lease| lease.run_id == receipt.job_run_id && lease.epoch > 0)
        .ok_or(SchedulerReceiptValidationErrorV1::InvalidLease)
}
