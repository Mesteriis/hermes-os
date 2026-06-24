use chrono::{DateTime, Utc};
use serde_json::Value;

use super::errors::EventEnvelopeError;
use super::models::NewEventEnvelope;
use super::validation::{validate_non_empty, validate_object};

pub struct NewEventEnvelopeBuilder {
    pub(super) event_id: String,
    pub(super) event_type: String,
    pub(super) schema_version: i32,
    pub(super) occurred_at: DateTime<Utc>,
    pub(super) source: Value,
    pub(super) actor: Option<Value>,
    pub(super) subject: Value,
    pub(super) payload: Value,
    pub(super) provenance: Value,
    pub(super) causation_id: Option<String>,
    pub(super) correlation_id: Option<String>,
}

impl NewEventEnvelopeBuilder {
    pub fn schema_version(mut self, schema_version: i32) -> Self {
        self.schema_version = schema_version;
        self
    }

    pub fn actor(mut self, actor: Value) -> Self {
        self.actor = Some(actor);
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    pub fn build(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        validate_non_empty("event_id", &self.event_id)?;
        validate_non_empty("event_type", &self.event_type)?;

        if self.schema_version <= 0 {
            return Err(EventEnvelopeError::InvalidSchemaVersion);
        }

        validate_object("source", &self.source)?;
        validate_object("subject", &self.subject)?;
        validate_object("payload", &self.payload)?;
        validate_object("provenance", &self.provenance)?;

        if let Some(actor) = &self.actor {
            validate_object("actor", actor)?;
        }

        let event_id = self.event_id.trim().to_owned();
        let correlation_id = self
            .correlation_id
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| event_id.clone());

        Ok(NewEventEnvelope {
            event_id,
            event_type: self.event_type.trim().to_owned(),
            schema_version: self.schema_version,
            occurred_at: self.occurred_at,
            source: self.source,
            actor: self.actor,
            subject: self.subject,
            payload: self.payload,
            provenance: self.provenance,
            causation_id: self.causation_id,
            correlation_id: Some(correlation_id),
        })
    }
}
