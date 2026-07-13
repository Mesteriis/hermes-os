use std::fmt;

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
