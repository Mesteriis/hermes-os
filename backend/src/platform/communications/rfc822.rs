mod body;
mod decoding;
pub mod errors;
mod headers;
mod models;
mod multipart;
mod parser;
mod util;
mod wire;

pub use models::{
    ParsedCommunicationSourceMessage, ParsedEmailAttachment, ParsedEmailAttachmentDisposition,
};
pub use parser::parse_rfc822_message;
