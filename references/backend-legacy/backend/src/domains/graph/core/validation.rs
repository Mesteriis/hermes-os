use serde_json::Value;

use super::errors::GraphStoreError;
use super::models::{NewGraphEdge, NewGraphEvidence};

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), GraphStoreError> {
    if value.trim().is_empty() {
        return Err(GraphStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_edge_with_evidence(
    edge: &NewGraphEdge,
    evidence: &[NewGraphEvidence],
) -> Result<(), GraphStoreError> {
    edge.validate()?;
    if evidence.is_empty() {
        return Err(GraphStoreError::SystemEdgeRequiresEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), GraphStoreError> {
    if !value.is_object() {
        return Err(GraphStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
