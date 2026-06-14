use serde_json::Value;

use super::errors::ObligationStoreError;
use super::models::{NewObligation, NewObligationEvidence};

pub(super) fn validate_obligation_with_evidence(
    obligation: &NewObligation,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
    obligation.validate()?;
    if evidence.is_empty() {
        return Err(ObligationStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ObligationStoreError> {
    if value.trim().is_empty() {
        return Err(ObligationStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(
    field_name: &'static str,
    value: f64,
) -> Result<(), ObligationStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ObligationStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ObligationStoreError> {
    if !value.is_object() {
        return Err(ObligationStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
