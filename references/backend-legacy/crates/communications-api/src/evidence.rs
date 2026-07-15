use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;

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

    pub fn validate(&self) -> Result<(), CommunicationEvidenceError> {
        required_non_empty("account_id", &self.account_id)?;
        required_non_empty("stream_id", &self.stream_id)?;
        if !self.checkpoint.is_object() {
            return Err(CommunicationEvidenceError::NonObjectJson("checkpoint"));
        }
        Ok(())
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CommunicationEvidenceError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Debug, Error)]
#[error("communication evidence port error: {0}")]
pub struct CommunicationEvidencePortError(pub String);

impl CommunicationEvidencePortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub type CommunicationEvidencePortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommunicationEvidencePortError>> + Send + 'a>>;

pub trait CommunicationRawEvidenceCommandPort: Send + Sync {
    fn record_raw_source<'a>(
        &'a self,
        record: &'a NewRawCommunicationRecord,
    ) -> CommunicationEvidencePortFuture<'a, StoredRawCommunicationRecord>;
}

pub trait IngestionCheckpointQueryPort: Send + Sync {
    fn checkpoint<'a>(
        &'a self,
        account_id: &'a str,
        stream_id: &'a str,
    ) -> CommunicationEvidencePortFuture<'a, Option<IngestionCheckpoint>>;
}

pub trait IngestionCheckpointCommandPort: IngestionCheckpointQueryPort {
    fn save_checkpoint<'a>(
        &'a self,
        checkpoint: &'a NewIngestionCheckpoint,
    ) -> CommunicationEvidencePortFuture<'a, IngestionCheckpoint>;

    fn delete_checkpoint<'a>(
        &'a self,
        account_id: &'a str,
        stream_id: &'a str,
    ) -> CommunicationEvidencePortFuture<'a, bool>;

    fn delete_checkpoints_with_stream_prefix<'a>(
        &'a self,
        account_id: &'a str,
        stream_prefix: &'a str,
    ) -> CommunicationEvidencePortFuture<'a, u64>;
}

/// Atomic evidence boundary used by ingestion workflows.
///
/// Implementations preserve the raw-evidence and checkpoint consistency
/// guarantees; callers do not receive storage or transaction capabilities.
pub trait CommunicationEvidencePort:
    CommunicationRawEvidenceCommandPort + IngestionCheckpointCommandPort
{
}

impl<T> CommunicationEvidencePort for T where
    T: CommunicationRawEvidenceCommandPort + IngestionCheckpointCommandPort + ?Sized
{
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StoredRawCommunicationRecord {
    pub raw_record_id: String,
    pub observation_id: String,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub captured_at: DateTime<Utc>,
    pub payload: Value,
    pub provenance: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewRawCommunicationRecord {
    pub raw_record_id: String,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
    pub provenance: Value,
}

impl NewRawCommunicationRecord {
    pub fn new(
        raw_record_id: impl Into<String>,
        account_id: impl Into<String>,
        record_kind: impl Into<String>,
        provider_record_id: impl Into<String>,
        source_fingerprint: impl Into<String>,
        import_batch_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            raw_record_id: raw_record_id.into(),
            account_id: account_id.into(),
            record_kind: record_kind.into(),
            provider_record_id: provider_record_id.into(),
            source_fingerprint: source_fingerprint.into(),
            import_batch_id: import_batch_id.into(),
            occurred_at: None,
            payload,
            provenance: json!({}),
        }
    }

    pub fn occurred_at(mut self, occurred_at: DateTime<Utc>) -> Self {
        self.occurred_at = Some(occurred_at);
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub fn validate(&self) -> Result<(), CommunicationEvidenceError> {
        required_non_empty("raw_record_id", &self.raw_record_id)?;
        required_non_empty("account_id", &self.account_id)?;
        required_non_empty("record_kind", &self.record_kind)?;
        required_non_empty("provider_record_id", &self.provider_record_id)?;
        required_non_empty("source_fingerprint", &self.source_fingerprint)?;
        required_non_empty("import_batch_id", &self.import_batch_id)?;
        if !self.payload.is_object() {
            return Err(CommunicationEvidenceError::NonObjectJson("payload"));
        }
        if !self.provenance.is_object() {
            return Err(CommunicationEvidenceError::NonObjectJson("provenance"));
        }
        Ok(())
    }
}

fn required_non_empty(field: &'static str, value: &str) -> Result<(), CommunicationEvidenceError> {
    if value.trim().is_empty() {
        return Err(CommunicationEvidenceError::EmptyField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CommunicationEvidenceError, NewIngestionCheckpoint};

    #[test]
    fn checkpoint_requires_identity_and_object_payload() {
        let empty_account = NewIngestionCheckpoint::new(" ", "events", json!({}));
        assert_eq!(
            empty_account.validate(),
            Err(CommunicationEvidenceError::EmptyField("account_id"))
        );

        let scalar_payload = NewIngestionCheckpoint::new("account-1", "events", json!(1));
        assert_eq!(
            scalar_payload.validate(),
            Err(CommunicationEvidenceError::NonObjectJson("checkpoint"))
        );
    }

    #[test]
    fn raw_record_requires_stable_identity_and_object_payloads() {
        let invalid = super::NewRawCommunicationRecord::new(
            "raw-1",
            "account-1",
            "message",
            "provider-1",
            "fingerprint-1",
            "batch-1",
            json!("not-an-object"),
        );
        assert_eq!(
            invalid.validate(),
            Err(CommunicationEvidenceError::NonObjectJson("payload"))
        );
    }
}
