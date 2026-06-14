use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailRfc822ParseError {
    #[error("RFC822 message must contain headers and body")]
    MalformedRfc822,
}
