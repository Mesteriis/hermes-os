//! Managed process root for the owner-neutral Attachment Security engine.

pub mod admission;
mod delegation;
mod event_decode;
mod outbox;
pub mod runtime;
mod scan;
pub mod settings;

pub use scan::AttachmentSecurityScanAdapterErrorV1;

pub const PACKAGE: &str = "hermes-attachment-security-runtime";
