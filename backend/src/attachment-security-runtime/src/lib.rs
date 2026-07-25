//! Managed process root for the owner-neutral Attachment Security engine.

pub mod admission;
mod event_decode;
mod outbox;
pub mod runtime;
mod scan;
pub mod settings;

pub const PACKAGE: &str = "hermes-attachment-security-runtime";
