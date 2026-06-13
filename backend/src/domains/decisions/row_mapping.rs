use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::DecisionStoreError;
use super::models::{Decision, DecisionEntityKind, DecisionReviewState, DecisionStatus};

pub(super) fn row_to_decision(row: PgRow) -> Result<Decision, DecisionStoreError> {
    let decided_by_entity_kind = row
        .try_get::<Option<String>, _>("decided_by_entity_kind")?
        .map(parse_entity_kind)
        .transpose()?;

    Ok(Decision {
        decision_id: row.try_get("decision_id")?,
        title: row.try_get("title")?,
        status: parse_status(row.try_get("status")?)?,
        rationale: row.try_get("rationale")?,
        alternatives: row.try_get("alternatives")?,
        decided_by_entity_kind,
        decided_by_entity_id: row.try_get("decided_by_entity_id")?,
        decided_at: row.try_get("decided_at")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<DecisionEntityKind, DecisionStoreError> {
    DecisionEntityKind::parse(value)
}

fn parse_status(value: String) -> Result<DecisionStatus, DecisionStoreError> {
    match value.as_str() {
        "active" => Ok(DecisionStatus::Active),
        "superseded" => Ok(DecisionStatus::Superseded),
        "reversed" => Ok(DecisionStatus::Reversed),
        "deprecated" => Ok(DecisionStatus::Deprecated),
        _ => Err(DecisionStoreError::UnknownStatus(value)),
    }
}

fn parse_review_state(value: String) -> Result<DecisionReviewState, DecisionStoreError> {
    DecisionReviewState::parse(value)
}
