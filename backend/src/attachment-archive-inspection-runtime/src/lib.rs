#![forbid(unsafe_code)]

pub mod admission;
mod blob;
mod event_decode;
mod outbox;
pub mod runtime;
pub mod settings;

pub const PACKAGE: &str = "hermes-attachment-archive-inspection-runtime";
