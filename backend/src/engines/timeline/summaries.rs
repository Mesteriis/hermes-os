use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

use super::errors::TimelineEngineError;
use super::models::{TimelineEventDraft, TimelinePeriodSummary};
use super::policy::validate_event;

pub(super) fn period_summary(
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
        validate_event(event)?;
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
