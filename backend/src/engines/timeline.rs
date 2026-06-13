use chrono::{DateTime, Utc};
use thiserror::Error;

pub struct TimelineEngine;

impl TimelineEngine {
    pub fn bounded_entity_limit(limit: i64) -> i64 {
        limit.clamp(1, 100)
    }

    pub fn validate_event(event: &TimelineEventDraft<'_>) -> Result<(), TimelineEngineError> {
        validate_non_empty("entity_kind", event.entity_kind)?;
        validate_non_empty("entity_id", event.entity_id)?;
        validate_non_empty("event_type", event.event_type)?;
        validate_non_empty("title", event.title)?;
        validate_non_empty("source", event.source)?;

        let _occurred_at: DateTime<Utc> = event.occurred_at;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TimelineEventDraft<'a> {
    pub entity_kind: &'a str,
    pub entity_id: &'a str,
    pub event_type: &'a str,
    pub title: &'a str,
    pub occurred_at: DateTime<Utc>,
    pub source: &'a str,
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TimelineEngineError> {
    if value.trim().is_empty() {
        return Err(TimelineEngineError::EmptyField(field));
    }
    Ok(())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TimelineEngineError {
    #[error("timeline event {0} must not be empty")]
    EmptyField(&'static str),
}
