#![forbid(unsafe_code)]

mod client_port;
mod delivery_port;
mod worker;

pub use client_port::{get_status_payload_v1, start_bulk_delivery_payload_v1};
pub use delivery_port::{
    DeliveryIntentRequestErrorV1, DeliveryIntentRequestPortV1, DeliveryIntentRequestV1,
    DeliveryIntentResponseV1,
};
pub use worker::{BulkDeliveryWorkerErrorV1, process_next_target_v1};

pub const PACKAGE: &str = "hermes-communication-bulk-action-runtime";
