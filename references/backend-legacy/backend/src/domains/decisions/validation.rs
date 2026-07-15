use serde_json::Value;
use sqlx::postgres::PgPool;

use super::constants::{MAX_REFRESH_LIMIT, MIN_REFRESH_LIMIT};
use super::errors::DecisionStoreError;
use super::ids::decision_id;
use super::models::decision::NewDecision;
use super::models::evidence::NewDecisionEvidence;
use super::models::impacted_entity::NewDecisionImpactedEntity;
use super::models::states::DecisionReviewState;

pub(super) fn validate_decision_with_evidence(
    decision: &NewDecision,
    evidence: &[NewDecisionEvidence],
    impacted_entities: &[NewDecisionImpactedEntity],
) -> Result<(), DecisionStoreError> {
    decision.validate()?;
    if evidence.is_empty() {
        return Err(DecisionStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }
    for item in impacted_entities {
        item.validate()?;
    }

    Ok(())
}

pub(super) async fn preserve_existing_review_state(
    pool: &PgPool,
    decision: &mut NewDecision,
) -> Result<(), DecisionStoreError> {
    let existing_review_state: Option<String> =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(decision_id(decision))
            .fetch_optional(pool)
            .await?;
    let Some(existing_review_state) = existing_review_state else {
        return Ok(());
    };
    let existing_review_state = DecisionReviewState::parse(existing_review_state)?;
    if existing_review_state != DecisionReviewState::Suggested {
        decision.review_state = existing_review_state;
    }

    Ok(())
}

pub(super) fn validate_refresh_limit(limit: i64) -> Result<i64, DecisionStoreError> {
    if !(MIN_REFRESH_LIMIT..=MAX_REFRESH_LIMIT).contains(&limit) {
        return Err(DecisionStoreError::InvalidLimit);
    }

    Ok(limit)
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), DecisionStoreError> {
    if value.trim().is_empty() {
        return Err(DecisionStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(
    field_name: &'static str,
    value: f64,
) -> Result<(), DecisionStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(DecisionStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), DecisionStoreError> {
    if !value.is_object() {
        return Err(DecisionStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_array(
    field_name: &'static str,
    value: &Value,
) -> Result<(), DecisionStoreError> {
    if !value.is_array() {
        return Err(DecisionStoreError::InvalidJsonArray(field_name));
    }

    Ok(())
}
