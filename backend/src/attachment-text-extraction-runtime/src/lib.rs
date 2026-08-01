#![forbid(unsafe_code)]
//! Managed workflow composition for bounded attachment text extraction.

mod admission;
mod blob;
mod client_port;
mod client_realtime;
mod contracts;
mod event_decode;
mod outbox;
mod parser;
pub mod runtime;

pub use admission::{
    ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
    attachment_text_extraction_module_descriptor_v1,
    attachment_text_extraction_settings_schema_bytes_v1,
    attachment_text_extraction_settings_schema_v1,
};
pub use parser::{
    AttachmentTextExtractionParserRuntimeV1, AttachmentTextRuntimeParseErrorV1,
    AttachmentTextRuntimeParseResultV1,
};

pub const PACKAGE: &str = "hermes-attachment-text-extraction-runtime";
