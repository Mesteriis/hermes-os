use std::cmp::Ordering;

use chrono::{DateTime, Utc};

use super::errors::MemoryEngineError;
use super::models::{MemoryFactState, MemoryStaleCandidate};
use super::validation::{validate_memory_fact_state, validate_non_empty};

pub(super) fn stale_memory_candidates(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    facts: &[MemoryFactState],
    as_of: DateTime<Utc>,
    stale_after_days: i64,
) -> Result<Vec<MemoryStaleCandidate>, MemoryEngineError> {
    validate_non_empty("affected entity kind", affected_entity_kind)?;
    validate_non_empty("affected entity", affected_entity_id)?;
    if stale_after_days <= 0 {
        return Err(MemoryEngineError::InvalidStaleThreshold);
    }

    let affected_entity_kind = affected_entity_kind.trim();
    let affected_entity_id = affected_entity_id.trim();
    let stale_cutoff = as_of - chrono::Duration::days(stale_after_days);
    let mut candidates = Vec::new();

    for fact in facts {
        validate_memory_fact_state(fact)?;
        if fact.affected_entity_kind.trim() != affected_entity_kind
            || fact.affected_entity_id.trim() != affected_entity_id
            || fact.review_state.trim() != "accepted"
        {
            continue;
        }

        if fact
            .last_verified_at
            .is_some_and(|verified_at| verified_at >= stale_cutoff)
        {
            continue;
        }

        candidates.push(MemoryStaleCandidate {
            affected_entity_kind: affected_entity_kind.to_owned(),
            affected_entity_id: affected_entity_id.to_owned(),
            fact_type: fact.fact_type.trim().to_owned(),
            value: fact.value.trim().to_owned(),
            source: fact.source.trim().to_owned(),
            confidence: fact.confidence,
            last_verified_at: fact.last_verified_at,
            review_state: "suggested".to_owned(),
            produced_by: "memory_engine".to_owned(),
        });
    }

    candidates.sort_by(|left, right| {
        compare_optional_time(left.last_verified_at, right.last_verified_at)
            .then_with(|| left.source.cmp(&right.source))
    });

    Ok(candidates)
}

fn compare_optional_time(left: Option<DateTime<Utc>>, right: Option<DateTime<Utc>>) -> Ordering {
    left.cmp(&right)
}
