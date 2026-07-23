//! Generic delivery of opaque bytes from one Kernel-authorized pull consumer.

use futures_util::StreamExt;

use super::{RuntimeJetStreamConnection, RuntimeSubscribePermitV1};

/// One unacknowledged JetStream message. Owner runtimes decide when it is safe
/// to acknowledge after their local inbox transaction has completed.
pub struct RuntimePullDeliveryV1 {
    message: async_nats::jetstream::Message,
}

impl RuntimePullDeliveryV1 {
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        self.message.payload.as_ref()
    }

    pub async fn acknowledge(self) -> Result<(), RuntimePullDeliveryErrorV1> {
        self.message
            .ack()
            .await
            .map_err(|_| RuntimePullDeliveryErrorV1::Unavailable)
    }
}

/// Receives a single delivery from exactly the Event Hub consumer bound to the
/// current runtime identity and grant epoch.
pub async fn receive_runtime_pull_delivery(
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
) -> Result<RuntimePullDeliveryV1, RuntimePullDeliveryErrorV1> {
    let consumer = connection
        .open_pull_consumer(permit)
        .await
        .map_err(|_| RuntimePullDeliveryErrorV1::Unavailable)?;
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await
        .map_err(|_| RuntimePullDeliveryErrorV1::Unavailable)?;
    messages
        .next()
        .await
        .ok_or(RuntimePullDeliveryErrorV1::Unavailable)?
        .map(|message| RuntimePullDeliveryV1 { message })
        .map_err(|_| RuntimePullDeliveryErrorV1::Unavailable)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePullDeliveryErrorV1 {
    Unavailable,
}
