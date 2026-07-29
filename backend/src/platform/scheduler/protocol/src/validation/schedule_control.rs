use crate::v1::{
    EnsureOneShotScheduleV1, JobKindV1, SchedulerScheduleControlCommandV1,
    SchedulerScheduleControlOutcomeV1, SchedulerScheduleControlResultV1,
    scheduler_schedule_control_command_v1::Operation,
};

const ID_BYTES: usize = 16;
const SHA256_BYTES: usize = 32;
const MAX_TOKEN_BYTES: usize = 128;
const MAX_SCOPE_BYTES: usize = 256;
const MAX_CONCURRENCY_KEY_BYTES: usize = 256;
const MAX_DEADLINE_MILLIS: u64 = 86_400_000;
const MAX_RETRY_ATTEMPTS: u32 = 32;
const MAX_RETRY_BACKOFF_MILLIS: u64 = 86_400_000;
const MAX_ERROR_CODE_BYTES: usize = 96;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlValidationErrorV1 {
    InvalidOperationId,
    MissingOperation,
    InvalidSchedule,
    InvalidJobContract,
    InvalidScope,
    InvalidPolicy,
    InvalidOutcome,
    InvalidErrorCode,
}

pub fn validate_scheduler_schedule_control_command_v1(
    command: &SchedulerScheduleControlCommandV1,
) -> Result<(), SchedulerScheduleControlValidationErrorV1> {
    if !fixed_nonzero(&command.operation_id, ID_BYTES) {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidOperationId);
    }
    match command.operation.as_ref() {
        Some(Operation::EnsureOneShot(request)) => validate_ensure(request),
        Some(Operation::CancelOneShot(request)) => {
            if !fixed_nonzero(&request.schedule_id, ID_BYTES)
                || request.expected_schedule_revision == 0
            {
                return Err(SchedulerScheduleControlValidationErrorV1::InvalidSchedule);
            }
            Ok(())
        }
        None => Err(SchedulerScheduleControlValidationErrorV1::MissingOperation),
    }
}

pub fn validate_scheduler_schedule_control_result_v1(
    result: &SchedulerScheduleControlResultV1,
) -> Result<(), SchedulerScheduleControlValidationErrorV1> {
    if !fixed_nonzero(&result.operation_id, ID_BYTES) {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidOperationId);
    }
    if !fixed_nonzero(&result.schedule_id, ID_BYTES) || result.schedule_revision == 0 {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidSchedule);
    }
    let outcome = SchedulerScheduleControlOutcomeV1::try_from(result.outcome)
        .map_err(|_| SchedulerScheduleControlValidationErrorV1::InvalidOutcome)?;
    match outcome {
        SchedulerScheduleControlOutcomeV1::Ensured
        | SchedulerScheduleControlOutcomeV1::Cancelled
        | SchedulerScheduleControlOutcomeV1::TooLate
            if result.error_code.is_empty() =>
        {
            Ok(())
        }
        SchedulerScheduleControlOutcomeV1::Rejected
            if token(&result.error_code, MAX_ERROR_CODE_BYTES) =>
        {
            Ok(())
        }
        SchedulerScheduleControlOutcomeV1::Unspecified => {
            Err(SchedulerScheduleControlValidationErrorV1::InvalidOutcome)
        }
        _ => Err(SchedulerScheduleControlValidationErrorV1::InvalidErrorCode),
    }
}

fn validate_ensure(
    request: &EnsureOneShotScheduleV1,
) -> Result<(), SchedulerScheduleControlValidationErrorV1> {
    if !fixed_nonzero(&request.schedule_id, ID_BYTES) || request.schedule_revision == 0 {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidSchedule);
    }
    if !valid_job_kind(request.job_kind.as_ref())
        || request.job_contract_revision == 0
        || !fixed_nonzero(&request.job_schema_sha256, SHA256_BYTES)
    {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidJobContract);
    }
    if !token(&request.scope_id, MAX_SCOPE_BYTES)
        || !token(&request.concurrency_key, MAX_CONCURRENCY_KEY_BYTES)
    {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidScope);
    }
    if request.due_at_unix_millis <= 0
        || !(1..=MAX_DEADLINE_MILLIS).contains(&request.deadline_millis)
        || request.max_retry_attempts > MAX_RETRY_ATTEMPTS
        || request.retry_base_backoff_millis > MAX_RETRY_BACKOFF_MILLIS
        || (request.max_retry_attempts == 0 && request.retry_base_backoff_millis != 0)
        || (request.max_retry_attempts > 0 && request.retry_base_backoff_millis == 0)
    {
        return Err(SchedulerScheduleControlValidationErrorV1::InvalidPolicy);
    }
    Ok(())
}

fn valid_job_kind(job_kind: Option<&JobKindV1>) -> bool {
    job_kind.is_some_and(|kind| {
        token(&kind.owner, MAX_TOKEN_BYTES) && token(&kind.name, MAX_TOKEN_BYTES) && kind.major > 0
    })
}

fn fixed_nonzero(value: &[u8], expected: usize) -> bool {
    value.len() == expected && value.iter().any(|byte| *byte != 0)
}

fn token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
}
