//! Authenticated JetStream connection boundaries.

mod client;
mod event_hub;
mod identity;
mod runtime;
mod scheduler_receipt;

pub use client::JetStreamClient;
pub use event_hub::EventHubJetStreamConnection;
pub use identity::{
    NatsPasswordCredentialV1, RuntimeNatsIdentity, RuntimePublishPermitV1, RuntimeSubscribePermitV1,
};
pub use runtime::{
    PublishReceipt, RuntimeJetStreamConnection, RuntimeOutboxPublisherV1, canonical_message_id,
};
pub use scheduler_receipt::{RuntimeSchedulerReceiptDeliveryV1, RuntimeSchedulerReceiptPortV1};
