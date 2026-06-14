use chrono::{DateTime, Utc};

use super::errors::TimelineEngineError;
use super::models::{TimelineEntry, TimelineEventDraft};
use super::policy::{bounded_entity_limit, validate_event};

pub(super) fn cross_domain_timeline(
    events: &[TimelineEventDraft<'_>],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<TimelineEntry>, TimelineEngineError> {
    if period_start > period_end {
        return Err(TimelineEngineError::InvalidPeriod);
    }

    let limit = bounded_entity_limit(limit) as usize;
    let mut timeline = Vec::new();

    for event in events {
        validate_event(event)?;
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
