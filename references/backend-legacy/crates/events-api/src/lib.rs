use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub source: Value,
    pub actor: Option<Value>,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StoredEventEnvelope {
    pub position: i64,
    pub event: EventEnvelope,
}

/// Storage-neutral query for canonical event-log reads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventLogQuery {
    pub event_type: Option<String>,
    pub source_code: Option<String>,
    pub subject_kind: Option<String>,
    pub subject_entity_id: Option<String>,
    pub correlation_id: Option<String>,
    pub position_after: Option<i64>,
    pub position_before: Option<i64>,
    pub occurred_after: Option<DateTime<Utc>>,
    pub occurred_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

impl EventLogQuery {
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = some_non_empty(event_type);
        self
    }

    pub fn source_code(mut self, source_code: impl Into<String>) -> Self {
        self.source_code = some_non_empty(source_code);
        self
    }

    pub fn subject_kind(mut self, subject_kind: impl Into<String>) -> Self {
        self.subject_kind = some_non_empty(subject_kind);
        self
    }

    pub fn subject_entity_id(mut self, subject_entity_id: impl Into<String>) -> Self {
        self.subject_entity_id = some_non_empty(subject_entity_id);
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = some_non_empty(correlation_id);
        self
    }

    pub fn position_between(mut self, position_after: i64, position_before: i64) -> Self {
        self.position_after = Some(position_after);
        self.position_before = Some(position_before);
        self
    }

    pub fn position_after(mut self, position_after: i64) -> Self {
        self.position_after = Some(position_after);
        self
    }

    pub fn position_before(mut self, position_before: i64) -> Self {
        self.position_before = Some(position_before);
        self
    }

    pub fn occurred_between(
        mut self,
        occurred_after: DateTime<Utc>,
        occurred_before: DateTime<Utc>,
    ) -> Self {
        self.occurred_after = Some(occurred_after);
        self.occurred_before = Some(occurred_before);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

fn some_non_empty(value: impl Into<String>) -> Option<String> {
    let value = value.into();
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventOutboxItem {
    pub event_id: String,
    pub subject: String,
    pub status: String,
    pub attempts: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DispatchableEventOutboxItem {
    pub event_id: String,
    pub subject: String,
    pub attempts: i32,
    pub event: EventEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewEventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub source: Value,
    pub actor: Option<Value>,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl NewEventEnvelope {
    pub fn builder(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        occurred_at: DateTime<Utc>,
        source: Value,
        subject: Value,
    ) -> NewEventEnvelopeBuilder {
        NewEventEnvelopeBuilder {
            event_id: event_id.into(),
            event_type: event_type.into(),
            schema_version: 1,
            occurred_at,
            source,
            actor: None,
            subject,
            payload: json!({}),
            provenance: json!({}),
            causation_id: None,
            correlation_id: None,
        }
    }
}

pub struct NewEventEnvelopeBuilder {
    event_id: String,
    event_type: String,
    schema_version: i32,
    occurred_at: DateTime<Utc>,
    source: Value,
    actor: Option<Value>,
    subject: Value,
    payload: Value,
    provenance: Value,
    causation_id: Option<String>,
    correlation_id: Option<String>,
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

#[derive(Debug, Error, Eq, PartialEq)]
pub enum EventEnvelopeError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),
    #[error("schema_version must be positive")]
    InvalidSchemaVersion,
    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), EventEnvelopeError> {
    if value.trim().is_empty() {
        return Err(EventEnvelopeError::EmptyField(field_name));
    }
    Ok(())
}

fn validate_object(field_name: &'static str, value: &Value) -> Result<(), EventEnvelopeError> {
    if !value.is_object() {
        return Err(EventEnvelopeError::NonObjectJson(field_name));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{EventEnvelopeError, NewEventEnvelope};

    #[test]
    fn builder_preserves_canonical_wire_fields() {
        let event = NewEventEnvelope::builder(
            "event-1",
            "signal.raw.provider.observed",
            Utc::now(),
            json!({"kind": "provider"}),
            json!({"account_id": "account-1"}),
        )
        .payload(json!({"payload": true}))
        .provenance(json!({"source": "fixture"}))
        .build()
        .expect("valid event");

        assert_eq!(event.schema_version, 1);
        assert_eq!(event.correlation_id.as_deref(), Some("event-1"));
        assert_eq!(event.payload, json!({"payload": true}));
    }

    #[test]
    fn builder_rejects_non_object_payload() {
        let error = NewEventEnvelope::builder(
            "event-1",
            "signal.raw.provider.observed",
            Utc::now(),
            json!({}),
            json!({}),
        )
        .payload(json!("invalid"))
        .build()
        .expect_err("payload must be an object");

        assert_eq!(error, EventEnvelopeError::NonObjectJson("payload"));
    }
}
