//! Pure admission mapping for module-originated Scheduler control commands.

mod one_shot;

pub use one_shot::{
    SchedulerApprovedJobV1, SchedulerOneShotScheduleErrorV1, SchedulerOneShotScheduleV1,
    map_approved_one_shot_schedule_v1,
};
