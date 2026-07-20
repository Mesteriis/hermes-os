use crate::v1::{JobTriggerKindV1, ScheduledJobCommandV1};

const MAX_SCOPE_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerCommandValidationErrorV1 {
    InvalidJobKind,
    InvalidRun,
    InvalidSchedule,
    InvalidScope,
    InvalidTrigger,
    InvalidLease,
}

pub fn validate_scheduled_job_command_v1(
    command: &ScheduledJobCommandV1,
) -> Result<(), SchedulerCommandValidationErrorV1> {
    valid_job_kind(command)
        .then_some(())
        .ok_or(SchedulerCommandValidationErrorV1::InvalidJobKind)?;
    valid_run_and_schedule(command)?;
    valid_scope_and_trigger(command)?;
    valid_lease(command)
}

fn valid_job_kind(command: &ScheduledJobCommandV1) -> bool {
    command
        .job_kind
        .as_ref()
        .is_some_and(|job| token(&job.owner, 64) && token(&job.name, 64) && job.major > 0)
}

fn valid_run_and_schedule(
    command: &ScheduledJobCommandV1,
) -> Result<(), SchedulerCommandValidationErrorV1> {
    if command.job_run_id.len() != 16 {
        return Err(SchedulerCommandValidationErrorV1::InvalidRun);
    }
    (command.schedule_id.len() == 16 && command.schedule_revision > 0)
        .then_some(())
        .ok_or(SchedulerCommandValidationErrorV1::InvalidSchedule)
}

fn valid_scope_and_trigger(
    command: &ScheduledJobCommandV1,
) -> Result<(), SchedulerCommandValidationErrorV1> {
    if command.scope_id.is_empty()
        || command.scope_id.len() > MAX_SCOPE_BYTES
        || !command.scope_id.is_ascii()
    {
        return Err(SchedulerCommandValidationErrorV1::InvalidScope);
    }
    JobTriggerKindV1::try_from(command.trigger_kind)
        .ok()
        .filter(|kind| *kind != JobTriggerKindV1::Unspecified)
        .map(|_| ())
        .ok_or(SchedulerCommandValidationErrorV1::InvalidTrigger)
}

fn valid_lease(command: &ScheduledJobCommandV1) -> Result<(), SchedulerCommandValidationErrorV1> {
    command
        .lease
        .as_ref()
        .filter(|lease| {
            lease.run_id == command.job_run_id
                && lease.epoch > 0
                && lease.expires_at_unix_millis > command.scheduled_for_unix_millis
        })
        .map(|_| ())
        .ok_or(SchedulerCommandValidationErrorV1::InvalidLease)
}

fn token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}
