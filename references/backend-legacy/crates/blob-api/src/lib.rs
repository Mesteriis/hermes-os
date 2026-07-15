use std::fmt;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Eq, PartialEq, Serialize)]
pub struct BlobRef {
    pub blob_id: String,
    pub account_id: String,
    pub expires_at: DateTime<Utc>,
    capability: String,
}

impl BlobRef {
    pub fn new(
        blob_id: impl Into<String>,
        account_id: impl Into<String>,
        capability: impl Into<String>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, BlobApiError> {
        Ok(Self {
            blob_id: required_identifier(blob_id.into(), "blob id")?,
            account_id: required_identifier(account_id.into(), "account id")?,
            capability: required_identifier(capability.into(), "blob capability")?,
            expires_at,
        })
    }

    pub fn capability(&self) -> &str {
        &self.capability
    }

    pub fn is_expired_at(&self, at: DateTime<Utc>) -> bool {
        at >= self.expires_at
    }
}

pub type BlobReadFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<u8>, BlobReadError>> + Send + 'a>>;

/// Capability-scoped blob read boundary. Implementations decide where bytes
/// live; callers never receive arbitrary filesystem access.
pub trait BlobReadPort: Send + Sync {
    fn read_bounded<'a>(&'a self, reference: &'a BlobRef, max_bytes: usize) -> BlobReadFuture<'a>;
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BlobReadError {
    #[error("blob capability is expired")]
    Expired,
    #[error("blob capability account mismatch")]
    AccountMismatch,
    #[error("blob is unavailable")]
    Unavailable,
    #[error("blob exceeds the configured size limit")]
    TooLarge,
}

impl fmt::Debug for BlobRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlobRef")
            .field("blob_id", &self.blob_id)
            .field("account_id", &self.account_id)
            .field("expires_at", &self.expires_at)
            .field("capability", &"[REDACTED]")
            .finish()
    }
}

fn required_identifier(value: String, kind: &'static str) -> Result<String, BlobApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(BlobApiError::EmptyField { kind });
    }
    Ok(value.to_owned())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BlobApiError {
    #[error("{kind} must not be empty")]
    EmptyField { kind: &'static str },
}
