use std::collections::BTreeMap;
use std::future;

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

use crate::platform::events::{EventStore, ProjectionCursorStore, StoredEventEnvelope};
use crate::platform::projections::{
    ProjectionHandlerError, ProjectionRunnerError, run_projection_batch,
};

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

    pub fn period_summary(
        events: &[TimelineEventDraft<'_>],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<TimelinePeriodSummary, TimelineEngineError> {
        if period_start > period_end {
            return Err(TimelineEngineError::InvalidPeriod);
        }

        let mut summary = TimelinePeriodSummary {
            period_start,
            period_end,
            total_events: 0,
            by_entity_kind: BTreeMap::new(),
            by_event_type: BTreeMap::new(),
        };

        for event in events {
            Self::validate_event(event)?;
            if event.occurred_at < period_start || event.occurred_at > period_end {
                continue;
            }

            summary.total_events += 1;
            *summary
                .by_entity_kind
                .entry(event.entity_kind.trim().to_owned())
                .or_insert(0) += 1;
            *summary
                .by_event_type
                .entry(event.event_type.trim().to_owned())
                .or_insert(0) += 1;
        }

        Ok(summary)
    }

    pub fn recency_signal(
        events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
        as_of: DateTime<Utc>,
    ) -> Result<TimelineRecencySignal, TimelineEngineError> {
        validate_non_empty("entity_kind", entity_kind)?;
        validate_non_empty("entity_id", entity_id)?;

        let entity_kind = entity_kind.trim();
        let entity_id = entity_id.trim();
        let mut latest_event: Option<&TimelineEventDraft<'_>> = None;

        for event in events {
            Self::validate_event(event)?;
            if event.occurred_at > as_of
                || event.entity_kind.trim() != entity_kind
                || event.entity_id.trim() != entity_id
            {
                continue;
            }

            match latest_event {
                Some(current) if current.occurred_at >= event.occurred_at => {}
                _ => latest_event = Some(event),
            }
        }

        let (last_event_at, last_event_type, last_event_source, age_seconds) =
            if let Some(event) = latest_event {
                (
                    Some(event.occurred_at),
                    Some(event.event_type.trim().to_owned()),
                    Some(event.source.trim().to_owned()),
                    Some(as_of.signed_duration_since(event.occurred_at).num_seconds()),
                )
            } else {
                (None, None, None, None)
            };

        Ok(TimelineRecencySignal {
            entity_kind: entity_kind.to_owned(),
            entity_id: entity_id.to_owned(),
            last_event_at,
            last_event_type,
            last_event_source,
            age_seconds,
        })
    }

    pub fn timeline_gaps(
        events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        max_gap_seconds: i64,
    ) -> Result<Vec<TimelineGap>, TimelineEngineError> {
        if period_start > period_end {
            return Err(TimelineEngineError::InvalidPeriod);
        }
        if max_gap_seconds <= 0 {
            return Err(TimelineEngineError::InvalidGapThreshold);
        }
        validate_non_empty("entity_kind", entity_kind)?;
        validate_non_empty("entity_id", entity_id)?;

        let entity_kind = entity_kind.trim();
        let entity_id = entity_id.trim();
        let mut entity_events = Vec::new();

        for event in events {
            Self::validate_event(event)?;
            if event.occurred_at < period_start
                || event.occurred_at > period_end
                || event.entity_kind.trim() != entity_kind
                || event.entity_id.trim() != entity_id
            {
                continue;
            }
            entity_events.push(event);
        }

        entity_events.sort_by_key(|event| event.occurred_at);

        let mut gaps = Vec::new();
        for pair in entity_events.windows(2) {
            let previous = pair[0];
            let next = pair[1];
            let gap_seconds = next
                .occurred_at
                .signed_duration_since(previous.occurred_at)
                .num_seconds();
            if gap_seconds <= max_gap_seconds {
                continue;
            }

            gaps.push(TimelineGap {
                entity_kind: entity_kind.to_owned(),
                entity_id: entity_id.to_owned(),
                gap_start: previous.occurred_at,
                gap_end: next.occurred_at,
                gap_seconds,
                previous_event_source: Some(previous.source.trim().to_owned()),
                next_event_source: Some(next.source.trim().to_owned()),
            });
        }

        Ok(gaps)
    }

    pub fn change_diff(
        previous_events: &[TimelineEventDraft<'_>],
        current_events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
    ) -> Result<TimelineChangeDiff, TimelineEngineError> {
        validate_non_empty("entity_kind", entity_kind)?;
        validate_non_empty("entity_id", entity_id)?;

        let entity_kind = entity_kind.trim();
        let entity_id = entity_id.trim();
        let previous = Self::events_by_source(previous_events, entity_kind, entity_id)?;
        let current = Self::events_by_source(current_events, entity_kind, entity_id)?;

        let mut added = Vec::new();
        for (source, event) in &current {
            if !previous.contains_key(source) {
                added.push(TimelineChange::from_event(event));
            }
        }

        let mut removed = Vec::new();
        for (source, event) in &previous {
            if !current.contains_key(source) {
                removed.push(TimelineChange::from_event(event));
            }
        }

        Ok(TimelineChangeDiff {
            entity_kind: entity_kind.to_owned(),
            entity_id: entity_id.to_owned(),
            added,
            removed,
        })
    }

    pub fn cross_domain_timeline(
        events: &[TimelineEventDraft<'_>],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<TimelineEntry>, TimelineEngineError> {
        if period_start > period_end {
            return Err(TimelineEngineError::InvalidPeriod);
        }

        let limit = Self::bounded_entity_limit(limit) as usize;
        let mut timeline = Vec::new();

        for event in events {
            Self::validate_event(event)?;
            if event.occurred_at < period_start || event.occurred_at > period_end {
                continue;
            }
            timeline.push(TimelineEntry::from_event(event));
        }

        timeline.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then_with(|| left.source.cmp(&right.source))
        });
        timeline.truncate(limit);

        Ok(timeline)
    }

    pub fn replay_event_log(
        stored_events: &[StoredEventEnvelope],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> Result<TimelineReplay, TimelineEngineError> {
        if period_start > period_end {
            return Err(TimelineEngineError::InvalidPeriod);
        }

        let last_replayed_position = stored_events
            .iter()
            .map(|stored| stored.position)
            .max()
            .unwrap_or(0);
        let limit = Self::bounded_entity_limit(limit) as usize;
        let mut entries = Vec::new();

        for stored in stored_events {
            let event = &stored.event;
            validate_non_empty("event_type", &event.event_type)?;
            if event.occurred_at < period_start || event.occurred_at > period_end {
                continue;
            }

            entries.push(TimelineEntry {
                entity_kind: required_json_string(
                    &event.subject,
                    "subject",
                    "kind",
                    &event.event_id,
                )?,
                entity_id: required_json_string(
                    &event.subject,
                    "subject",
                    "entity_id",
                    &event.event_id,
                )?,
                event_type: event.event_type.trim().to_owned(),
                title: optional_json_string(&event.payload, "title")
                    .unwrap_or_else(|| event.event_type.trim().to_owned()),
                occurred_at: event.occurred_at,
                source: event_log_source_ref(event),
            });
        }

        entries.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then_with(|| left.source.cmp(&right.source))
        });
        entries.truncate(limit);

        Ok(TimelineReplay {
            last_replayed_position,
            entries,
        })
    }

    pub async fn run_event_log_projection(
        events: &EventStore,
        cursors: &ProjectionCursorStore,
        projection_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        batch_size: u32,
        timeline_limit: i64,
    ) -> Result<TimelineProjectionRun, TimelineProjectionError> {
        let mut replay_batch = Vec::new();
        let outcome = run_projection_batch(events, cursors, projection_name, batch_size, |event| {
            let validation =
                Self::replay_event_log(std::slice::from_ref(&event), period_start, period_end, 1)
                    .map(|_| ())
                    .map_err(|error| ProjectionHandlerError::new(error.to_string()));
            if validation.is_ok() {
                replay_batch.push(event);
            }
            future::ready(validation)
        })
        .await?;

        let replay =
            Self::replay_event_log(&replay_batch, period_start, period_end, timeline_limit)?;

        Ok(TimelineProjectionRun {
            processed_count: outcome.processed_count,
            last_processed_position: outcome.last_processed_position,
            entries: replay.entries,
        })
    }

    fn events_by_source<'a>(
        events: &'a [TimelineEventDraft<'a>],
        entity_kind: &str,
        entity_id: &str,
    ) -> Result<BTreeMap<String, &'a TimelineEventDraft<'a>>, TimelineEngineError> {
        let mut by_source = BTreeMap::new();
        for event in events {
            Self::validate_event(event)?;
            if event.entity_kind.trim() == entity_kind && event.entity_id.trim() == entity_id {
                by_source.insert(event.source.trim().to_owned(), event);
            }
        }
        Ok(by_source)
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelinePeriodSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub by_entity_kind: BTreeMap<String, usize>,
    pub by_event_type: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineRecencySignal {
    pub entity_kind: String,
    pub entity_id: String,
    pub last_event_at: Option<DateTime<Utc>>,
    pub last_event_type: Option<String>,
    pub last_event_source: Option<String>,
    pub age_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineGap {
    pub entity_kind: String,
    pub entity_id: String,
    pub gap_start: DateTime<Utc>,
    pub gap_end: DateTime<Utc>,
    pub gap_seconds: i64,
    pub previous_event_source: Option<String>,
    pub next_event_source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineChangeDiff {
    pub entity_kind: String,
    pub entity_id: String,
    pub added: Vec<TimelineChange>,
    pub removed: Vec<TimelineChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineChange {
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
}

impl TimelineChange {
    fn from_event(event: &TimelineEventDraft<'_>) -> Self {
        Self {
            event_type: event.event_type.trim().to_owned(),
            occurred_at: event.occurred_at,
            source: event.source.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineEntry {
    pub entity_kind: String,
    pub entity_id: String,
    pub event_type: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
}

impl TimelineEntry {
    fn from_event(event: &TimelineEventDraft<'_>) -> Self {
        Self {
            entity_kind: event.entity_kind.trim().to_owned(),
            entity_id: event.entity_id.trim().to_owned(),
            event_type: event.event_type.trim().to_owned(),
            title: event.title.trim().to_owned(),
            occurred_at: event.occurred_at,
            source: event.source.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineReplay {
    pub last_replayed_position: i64,
    pub entries: Vec<TimelineEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineProjectionRun {
    pub processed_count: usize,
    pub last_processed_position: i64,
    pub entries: Vec<TimelineEntry>,
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TimelineEngineError> {
    if value.trim().is_empty() {
        return Err(TimelineEngineError::EmptyField(field));
    }
    Ok(())
}

fn required_json_string(
    value: &Value,
    object_name: &'static str,
    field_name: &'static str,
    event_id: &str,
) -> Result<String, TimelineEngineError> {
    let field_value = value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| TimelineEngineError::InvalidEventLogField {
            event_id: event_id.to_owned(),
            object_name,
            field_name,
        })?;

    Ok(field_value.to_owned())
}

fn optional_json_string(value: &Value, field_name: &str) -> Option<String> {
    value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn event_log_source_ref(event: &crate::platform::events::EventEnvelope) -> String {
    let Some(kind) = optional_json_string(&event.source, "kind") else {
        return event.event_id.clone();
    };
    let Some(source_id) = optional_json_string(&event.source, "source_id") else {
        return event.event_id.clone();
    };

    format!("{kind}:{source_id}")
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TimelineEngineError {
    #[error("timeline event {0} must not be empty")]
    EmptyField(&'static str),
    #[error("timeline period start must not be after period end")]
    InvalidPeriod,
    #[error("timeline gap threshold must be greater than zero")]
    InvalidGapThreshold,
    #[error("event log event `{event_id}` {object_name}.{field_name} must be a non-empty string")]
    InvalidEventLogField {
        event_id: String,
        object_name: &'static str,
        field_name: &'static str,
    },
}

#[derive(Debug, Error)]
pub enum TimelineProjectionError {
    #[error(transparent)]
    Runner(#[from] ProjectionRunnerError),

    #[error(transparent)]
    Timeline(#[from] TimelineEngineError),
}
