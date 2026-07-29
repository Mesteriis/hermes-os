#![forbid(unsafe_code)]

pub mod admission;
pub mod body_materializer;
pub mod coordinator;
mod event_runtime;
pub mod provider_event_admission;
pub mod provider_events;
pub mod runtime;

pub const PACKAGE: &str = "hermes-communication-delivery-intent-runtime";
