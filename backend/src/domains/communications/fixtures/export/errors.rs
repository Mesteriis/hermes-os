use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailFixtureExportError {
    #[error("email sync payload missing raw_rfc822_base64")]
    MissingRawRfc822,

    #[error("email sync payload raw_rfc822_base64 is invalid base64: {0}")]
    InvalidRawBase64(base64::DecodeError),

    #[error("raw RFC822 message does not contain a header/body separator")]
    MalformedRfc822,
}
