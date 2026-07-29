//! Transport-neutral delivery contract for module schedule-control commands.

use std::future::Future;

/// Exact command bytes remain broker-owned until Scheduler commits its inbox,
/// mutation and correlated result outbox in one transaction.
pub trait SchedulerScheduleControlDeliveryV1 {
    fn exact_bytes(&self) -> &[u8];

    fn acknowledge(
        self,
    ) -> impl Future<Output = Result<(), SchedulerScheduleControlDeliveryErrorV1>> + Send;
}

/// Kernel/Event-Hub-authorized command input without a NATS implementation
/// dependency in Scheduler protocol or persistence.
pub trait SchedulerScheduleControlDeliveryPortV1 {
    type Delivery: SchedulerScheduleControlDeliveryV1;

    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<Self::Delivery, SchedulerScheduleControlDeliveryErrorV1>> + Send;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlDeliveryErrorV1 {
    Unavailable,
}
