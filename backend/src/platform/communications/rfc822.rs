mod body;
mod decoding;
pub mod errors;
mod headers;
pub mod models;
mod multipart;
mod parser;
mod util;
mod wire;

pub use parser::parse_rfc822_message;
