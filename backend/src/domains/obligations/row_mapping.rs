use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ObligationStoreError;
use super::models::{
    Obligation, ObligationEntityKind, ObligationReviewState, ObligationRiskState, ObligationStatus,
};

pub(super) fn row_to_obligation(row: PgRow) -> Result<Obligation, ObligationStoreError> {
    let beneficiary_entity_kind = row
        .try_get::<Option<String>, _>("beneficiary_entity_kind")?
        .map(parse_entity_kind)
        .transpose()?;

    Ok(Obligation {
        obligation_id: row.try_get("obligation_id")?,
        obligated_entity_kind: parse_entity_kind(row.try_get("obligated_entity_kind")?)?,
        obligated_entity_id: row.try_get("obligated_entity_id")?,
        beneficiary_entity_kind,
        beneficiary_entity_id: row.try_get("beneficiary_entity_id")?,
        statement: row.try_get("statement")?,
        status: parse_status(row.try_get("status")?)?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        due_at: row.try_get("due_at")?,
        condition: row.try_get("condition")?,
        risk_state: parse_risk_state(row.try_get("risk_state")?)?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<ObligationEntityKind, ObligationStoreError> {
    ObligationEntityKind::parse(value)
}

fn parse_status(value: String) -> Result<ObligationStatus, ObligationStoreError> {
    match value.as_str() {
        "open" => Ok(ObligationStatus::Open),
        "fulfilled" => Ok(ObligationStatus::Fulfilled),
        "waived" => Ok(ObligationStatus::Waived),
        "disputed" => Ok(ObligationStatus::Disputed),
        "canceled" => Ok(ObligationStatus::Canceled),
        _ => Err(ObligationStoreError::UnknownStatus(value)),
    }
}

fn parse_review_state(value: String) -> Result<ObligationReviewState, ObligationStoreError> {
    ObligationReviewState::parse(value)
}

fn parse_risk_state(value: String) -> Result<ObligationRiskState, ObligationStoreError> {
    match value.as_str() {
        "none" => Ok(ObligationRiskState::None),
        "watch" => Ok(ObligationRiskState::Watch),
        "at_risk" => Ok(ObligationRiskState::AtRisk),
        "breached" => Ok(ObligationRiskState::Breached),
        _ => Err(ObligationStoreError::UnknownRiskState(value)),
    }
}
