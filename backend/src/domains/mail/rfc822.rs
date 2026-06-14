mod body;
mod decoding;
mod errors;
mod headers;
mod models;
mod multipart;
mod parser;
mod util;
mod wire;

pub use errors::EmailRfc822ParseError;
pub use models::{ParsedEmailAttachment, ParsedEmailAttachmentDisposition, ParsedEmailMessage};
pub use parser::parse_rfc822_message;
