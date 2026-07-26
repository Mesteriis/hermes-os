use crate::v1::{
    JobTriggerKindV1, OwnerJobCommandV1, OwnerJobTriggerKindV1, ScheduledJobCommandV1,
};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerJobCommandValidationErrorV1 {
    InvalidJobKind,
    InvalidRun,
    InvalidScope,
    InvalidTrigger,
    InvalidAcceptedAt,
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

pub fn validate_owner_job_command_v1(
    command: &OwnerJobCommandV1,
) -> Result<(), OwnerJobCommandValidationErrorV1> {
    command
        .job_kind
        .as_ref()
        .filter(|job| token(&job.owner, 64) && token(&job.name, 64) && job.major > 0)
        .map(|_| ())
        .ok_or(OwnerJobCommandValidationErrorV1::InvalidJobKind)?;
    if command.job_run_id.len() != 16 || command.job_run_id.iter().all(|byte| *byte == 0) {
        return Err(OwnerJobCommandValidationErrorV1::InvalidRun);
    }
    if command.scope_id.is_empty()
        || command.scope_id.len() > MAX_SCOPE_BYTES
        || !command.scope_id.is_ascii()
    {
        return Err(OwnerJobCommandValidationErrorV1::InvalidScope);
    }
    OwnerJobTriggerKindV1::try_from(command.trigger_kind)
        .ok()
        .filter(|kind| *kind == OwnerJobTriggerKindV1::UpgradeReconciliation)
        .map(|_| ())
        .ok_or(OwnerJobCommandValidationErrorV1::InvalidTrigger)?;
    if command.accepted_at_unix_millis <= 0 {
        return Err(OwnerJobCommandValidationErrorV1::InvalidAcceptedAt);
    }
    command
        .lease
        .as_ref()
        .filter(|lease| {
            lease.run_id == command.job_run_id
                && lease.epoch > 0
                && lease.expires_at_unix_millis > command.accepted_at_unix_millis
        })
        .map(|_| ())
        .ok_or(OwnerJobCommandValidationErrorV1::InvalidLease)
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
