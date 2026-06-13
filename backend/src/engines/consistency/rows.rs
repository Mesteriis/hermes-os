use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ConsistencyError;
use super::models::{
    ContradictionObservation, ContradictionReviewState, ContradictionSeverity,
    ContradictionSourceKind,
};

pub(super) fn row_to_observation(row: PgRow) -> Result<ContradictionObservation, ConsistencyError> {
    Ok(ContradictionObservation {
        observation_id: row.try_get("observation_id")?,
        old_source_kind: parse_source_kind(row.try_get("old_source_kind")?)?,
        old_source_id: row.try_get("old_source_id")?,
        new_source_kind: parse_source_kind(row.try_get("new_source_kind")?)?,
        new_source_id: row.try_get("new_source_id")?,
        affected_entities: row.try_get("affected_entities")?,
        conflict_type: row.try_get("conflict_type")?,
        old_claim: row.try_get("old_claim")?,
        new_claim: row.try_get("new_claim")?,
        confidence: row.try_get("confidence")?,
        severity: parse_severity(row.try_get("severity")?)?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        metadata: row.try_get("metadata")?,
        reviewed_by: row.try_get("reviewed_by")?,
        reviewed_at: row.try_get("reviewed_at")?,
        resolution: row.try_get("resolution")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn parse_source_kind(
    value: String,
) -> Result<ContradictionSourceKind, ConsistencyError> {
    match value.as_str() {
        "communication" => Ok(ContradictionSourceKind::Communication),
        "document" => Ok(ContradictionSourceKind::Document),
        "event" => Ok(ContradictionSourceKind::Event),
        "memory" => Ok(ContradictionSourceKind::Memory),
        "knowledge" => Ok(ContradictionSourceKind::Knowledge),
        "decision" => Ok(ContradictionSourceKind::Decision),
        "obligation" => Ok(ContradictionSourceKind::Obligation),
        "task" => Ok(ContradictionSourceKind::Task),
        "relationship" => Ok(ContradictionSourceKind::Relationship),
        "raw_record" => Ok(ContradictionSourceKind::RawRecord),
        _ => Err(ConsistencyError::UnknownSourceKind(value)),
    }
}

pub(super) fn parse_severity(value: String) -> Result<ContradictionSeverity, ConsistencyError> {
    match value.as_str() {
        "low" => Ok(ContradictionSeverity::Low),
        "medium" => Ok(ContradictionSeverity::Medium),
        "high" => Ok(ContradictionSeverity::High),
        "critical" => Ok(ContradictionSeverity::Critical),
        _ => Err(ConsistencyError::UnknownSeverity(value)),
    }
}

pub(super) fn parse_review_state(
    value: String,
) -> Result<ContradictionReviewState, ConsistencyError> {
    ContradictionReviewState::parse(value)
}
