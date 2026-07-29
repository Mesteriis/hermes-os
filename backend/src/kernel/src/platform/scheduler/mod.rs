//! Kernel-side composition of the independently supervised Scheduler runtime.

pub(crate) mod admission;
pub(crate) mod catalog;
pub(crate) mod launch;
pub(crate) mod lifecycle;
pub(crate) mod schedule_control;
pub(crate) mod status;

pub(crate) use catalog as scheduler_catalog;
