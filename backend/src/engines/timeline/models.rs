use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

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
    pub(super) fn from_event(event: &TimelineEventDraft<'_>) -> Self {
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
    pub(super) fn from_event(event: &TimelineEventDraft<'_>) -> Self {
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
