//! Transport-neutral receipt delivery contract for the Scheduler runtime.

use std::future::Future;

/// A received exact durable-envelope byte sequence that remains unacknowledged
/// until Scheduler has committed its fenced receipt state.
pub trait SchedulerReceiptDeliveryV1 {
    fn exact_bytes(&self) -> &[u8];

    fn acknowledge(
        self,
    ) -> impl Future<Output = Result<(), SchedulerReceiptDeliveryErrorV1>> + Send;
}

/// A Kernel/Event-Hub-authorized receipt input. Scheduler depends only on this
/// contract, never on NATS implementation types or owner runtime clients.
pub trait SchedulerReceiptDeliveryPortV1 {
    type Delivery: SchedulerReceiptDeliveryV1;

    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<Self::Delivery, SchedulerReceiptDeliveryErrorV1>> + Send;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReceiptDeliveryErrorV1 {
    Unavailable,
}
