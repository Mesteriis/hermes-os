use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::super::errors::CommunicationIngestionError;
use super::super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IngestionCheckpoint {
    pub account_id: String,
    pub stream_id: String,
    pub checkpoint: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewIngestionCheckpoint {
    pub account_id: String,
    pub stream_id: String,
    pub checkpoint: Value,
}

impl NewIngestionCheckpoint {
    pub fn new(
        account_id: impl Into<String>,
        stream_id: impl Into<String>,
        checkpoint: Value,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            stream_id: stream_id.into(),
            checkpoint,
        }
    }

    pub(in crate::domains::mail::core) fn validate(
        &self,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("stream_id", &self.stream_id)?;
        validate_object("checkpoint", &self.checkpoint)
    }
}
