use serde_json::Value;

use super::errors::RelationshipStoreError;
use super::models::{NewRelationship, NewRelationshipEvidence};

pub(super) fn validate_relationship_with_evidence(
    relationship: &NewRelationship,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipStoreError> {
    validate_relationship(relationship)?;
    if evidence.is_empty() {
        return Err(RelationshipStoreError::MissingEvidence);
    }
    for item in evidence {
        validate_evidence(item)?;
    }

    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), RelationshipStoreError> {
    if value.trim().is_empty() {
        return Err(RelationshipStoreError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_relationship(relationship: &NewRelationship) -> Result<(), RelationshipStoreError> {
    validate_non_empty("source_entity_id", &relationship.source_entity_id)?;
    validate_non_empty("target_entity_id", &relationship.target_entity_id)?;
    validate_non_empty("relationship_type", &relationship.relationship_type)?;
    validate_score("trust_score", relationship.trust_score)?;
    validate_score("strength_score", relationship.strength_score)?;
    validate_score("confidence", relationship.confidence)?;
    validate_json_object("relationship metadata", &relationship.metadata)?;
    if relationship.source_entity_kind == relationship.target_entity_kind
        && relationship.source_entity_id == relationship.target_entity_id
    {
        return Err(RelationshipStoreError::IdenticalEndpoints);
    }
    if let (Some(valid_from), Some(valid_to)) = (relationship.valid_from, relationship.valid_to)
        && valid_to < valid_from
    {
        return Err(RelationshipStoreError::InvalidTemporalRange);
    }

    Ok(())
}

fn validate_evidence(evidence: &NewRelationshipEvidence) -> Result<(), RelationshipStoreError> {
    validate_non_empty("source_id", &evidence.source_id)?;
    validate_json_object("evidence metadata", &evidence.metadata)
}

fn validate_score(field_name: &'static str, value: f64) -> Result<(), RelationshipStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(RelationshipStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), RelationshipStoreError> {
    if !value.is_object() {
        return Err(RelationshipStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
