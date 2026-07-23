//! Authenticated JetStream connection boundaries.

mod client;
mod event_hub;
mod identity;
mod managed_runtime;
mod pull_delivery;
mod runtime;
mod scheduler_receipt;

pub use client::JetStreamClient;
pub use event_hub::EventHubJetStreamConnection;
pub use identity::{
    NatsPasswordCredentialV1, RuntimeNatsIdentity, RuntimePublishPermitV1, RuntimeSubscribePermitV1,
};
pub use managed_runtime::{
    ManagedRuntimeEventAccessErrorV1, ManagedRuntimeEventAccessV1,
    request_managed_runtime_event_access,
};
pub use pull_delivery::{
    RuntimePullDeliveryErrorV1, RuntimePullDeliveryV1, receive_runtime_pull_delivery,
};
pub use runtime::{
    PublishReceipt, RuntimeJetStreamConnection, RuntimeOutboxPublisherV1, canonical_message_id,
};
pub use scheduler_receipt::{RuntimeSchedulerReceiptDeliveryV1, RuntimeSchedulerReceiptPortV1};
