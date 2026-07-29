#![forbid(unsafe_code)]

pub mod admission;
pub mod body_materializer;
pub mod client_port;
mod client_realtime;
mod client_status;
pub mod communications_query_client;
mod contracts;
pub mod coordinator;
mod event_runtime;
mod module_request_port;
pub mod provider_event_admission;
pub mod provider_events;
pub mod runtime;
mod submit_port;

pub const PACKAGE: &str = "hermes-communication-delivery-intent-runtime";
