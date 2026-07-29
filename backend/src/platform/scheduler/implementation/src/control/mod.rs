//! Pure admission mapping for module-originated Scheduler control commands.

mod admission;
mod one_shot;

pub use admission::{
    SchedulerAdmittedScheduleControlV1, SchedulerScheduleControlAdmissionErrorV1,
    SchedulerScheduleControlContractV1, SchedulerScheduleControlGrantV1,
    SchedulerScheduleControlOperationV1, admit_schedule_control_command_v1,
};
pub use one_shot::{
    SchedulerApprovedJobV1, SchedulerOneShotScheduleErrorV1, SchedulerOneShotScheduleV1,
    map_approved_one_shot_schedule_v1,
};
