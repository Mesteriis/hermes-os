//! Scheduler wire-command contract tests without owner implementation code.

use hermes_clock_protocol::UtcMillisV1;
use hermes_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, JobRunIdV1, OpaqueScheduleScopeV1,
    ScheduleIdV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleSpecV1,
    SchedulerCommandValidationErrorV1, build_scheduled_job_command_v1, v1,
    validate_scheduled_job_command_v1,
};

#[test]
fn scheduler_command_accepts_a_complete_fenced_time_trigger() {
    assert_eq!(validate_scheduled_job_command_v1(&command()), Ok(()));
}

#[test]
fn scheduler_command_rejects_a_lease_for_another_run() {
    let mut value = command();
    value.lease.as_mut().expect("lease").run_id = vec![9; 16];
    assert_eq!(
        validate_scheduled_job_command_v1(&value),
        Err(SchedulerCommandValidationErrorV1::InvalidLease)
    );
}

#[test]
fn scheduler_command_rejects_unknown_trigger_kind() {
    let mut value = command();
    value.trigger_kind = 99;
    assert_eq!(
        validate_scheduled_job_command_v1(&value),
        Err(SchedulerCommandValidationErrorV1::InvalidTrigger)
    );
}

#[test]
fn scheduler_command_builder_preserves_schedule_and_lease_fences() {
    let schedule = schedule();
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([3; 16]).expect("run"),
        schedule.schedule_id(),
        schedule.revision(),
        2,
        UtcMillisV1::new(2_000),
    )
    .expect("lease");
    let command = build_scheduled_job_command_v1(
        &schedule,
        &lease,
        UtcMillisV1::new(1_000),
        v1::JobTriggerKindV1::Time,
    )
    .expect("command");
    assert_eq!(command.job_run_id, vec![3; 16]);
    assert_eq!(command.lease.expect("lease").epoch, 2);
}

fn command() -> v1::ScheduledJobCommandV1 {
    v1::ScheduledJobCommandV1 {
        job_run_id: vec![1; 16],
        job_kind: Some(v1::JobKindV1 {
            owner: "mail".into(),
            name: "fetch".into(),
            major: 1,
        }),
        schedule_id: vec![2; 16],
        schedule_revision: 1,
        scope_id: "scope:opaque_42".into(),
        trigger_kind: v1::JobTriggerKindV1::Time as i32,
        scheduled_for_unix_millis: 1_000,
        lease: Some(v1::JobLeaseV1 {
            run_id: vec![1; 16],
            epoch: 1,
            expires_at_unix_millis: 2_000,
        }),
    }
}

fn schedule() -> ScheduleSpecV1 {
    let kind = JobKindV1::new("mail".into(), "fetch".into(), 1).expect("kind");
    let binding =
        JobContractBindingV1::new(kind, "mail.fetch".into(), 1, [7; 32]).expect("binding");
    let policy = hermes_scheduler_protocol::SchedulePolicyV1::new(
        hermes_scheduler_protocol::ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        hermes_scheduler_protocol::OverlapPolicyV1::Forbid,
        hermes_scheduler_protocol::MisfirePolicyV1::Skip,
        hermes_scheduler_protocol::RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("policy");
    ScheduleSpecV1::new(
        ScheduleIdV1::new([2; 16]).expect("schedule"),
        ScheduleRevisionV1::new(1).expect("revision"),
        binding,
        OpaqueScheduleScopeV1::new("scope:opaque_42".into()).expect("scope"),
        ConcurrencyKeyV1::new("mailbox:opaque_42".into()).expect("key"),
        true,
        policy,
    )
}
