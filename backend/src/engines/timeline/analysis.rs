use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

use super::errors::TimelineEngineError;
use super::models::{
    TimelineChange, TimelineChangeDiff, TimelineEventDraft, TimelineGap, TimelineRecencySignal,
};
use super::policy::validate_event;
use super::validation::validate_non_empty;

pub(super) fn recency_signal(
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
        validate_event(event)?;
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

pub(super) fn timeline_gaps(
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
        validate_event(event)?;
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

pub(super) fn change_diff(
    previous_events: &[TimelineEventDraft<'_>],
    current_events: &[TimelineEventDraft<'_>],
    entity_kind: &str,
    entity_id: &str,
) -> Result<TimelineChangeDiff, TimelineEngineError> {
    validate_non_empty("entity_kind", entity_kind)?;
    validate_non_empty("entity_id", entity_id)?;

    let entity_kind = entity_kind.trim();
    let entity_id = entity_id.trim();
    let previous = events_by_source(previous_events, entity_kind, entity_id)?;
    let current = events_by_source(current_events, entity_kind, entity_id)?;

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

fn events_by_source<'a>(
    events: &'a [TimelineEventDraft<'a>],
    entity_kind: &str,
    entity_id: &str,
) -> Result<BTreeMap<String, &'a TimelineEventDraft<'a>>, TimelineEngineError> {
    let mut by_source = BTreeMap::new();
    for event in events {
        validate_event(event)?;
        if event.entity_kind.trim() == entity_kind && event.entity_id.trim() == entity_id {
            by_source.insert(event.source.trim().to_owned(), event);
        }
    }
    Ok(by_source)
}
