use serde_json::Value;

use super::constants::{MAX_REFRESH_LIMIT, MIN_REFRESH_LIMIT};
use super::errors::ConsistencyError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ConsistencyError> {
    if value.trim().is_empty() {
        return Err(ConsistencyError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_confidence(confidence: f64) -> Result<(), ConsistencyError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(ConsistencyError::InvalidConfidence(confidence));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ConsistencyError> {
    if !value.is_object() {
        return Err(ConsistencyError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_array_or_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ConsistencyError> {
    if !value.is_array() && !value.is_object() {
        return Err(ConsistencyError::InvalidJsonArrayOrObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_refresh_limit(limit: i64) -> i64 {
    limit.clamp(MIN_REFRESH_LIMIT, MAX_REFRESH_LIMIT)
}
