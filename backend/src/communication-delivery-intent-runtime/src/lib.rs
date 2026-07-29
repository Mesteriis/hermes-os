#![forbid(unsafe_code)]

pub mod admission;
pub mod body_materializer;
pub mod coordinator;
pub mod provider_events;
pub mod runtime;

pub const PACKAGE: &str = "hermes-communication-delivery-intent-runtime";
