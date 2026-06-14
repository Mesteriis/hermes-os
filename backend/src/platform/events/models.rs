use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::builder::NewEventEnvelopeBuilder;

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
