use chrono::{DateTime, Utc};

use super::errors::TimelineEngineError;
use super::models::TimelineEventDraft;
use super::validation::validate_non_empty;

pub(super) fn bounded_entity_limit(limit: i64) -> i64 {
    limit.clamp(1, 100)
}

pub(super) fn validate_event(event: &TimelineEventDraft<'_>) -> Result<(), TimelineEngineError> {
    validate_non_empty("entity_kind", event.entity_kind)?;
    validate_non_empty("entity_id", event.entity_id)?;
    validate_non_empty("event_type", event.event_type)?;
    validate_non_empty("title", event.title)?;
    validate_non_empty("source", event.source)?;

    let _occurred_at: DateTime<Utc> = event.occurred_at;

    Ok(())
}
