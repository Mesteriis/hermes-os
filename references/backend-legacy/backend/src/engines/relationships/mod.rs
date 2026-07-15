use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelationshipSubject {
    pub entity_kind: String,
    pub entity_id: String,
}

impl RelationshipSubject {
    pub fn new(entity_kind: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelationshipCandidate {
    pub candidate_id: String,
    pub source: RelationshipSubject,
    pub target: RelationshipSubject,
    pub relationship_type: String,
    pub confidence: f64,
    pub evidence_observation_ids: Vec<String>,
}

impl RelationshipCandidate {
    pub fn linked_entities_candidate(
        source: RelationshipSubject,
        target: RelationshipSubject,
        relationship_type: impl Into<String>,
        confidence: f64,
        evidence_observation_ids: Vec<String>,
    ) -> Result<Self, RelationshipEngineError> {
        validate_subject(&source)?;
        validate_subject(&target)?;
        validate_confidence(confidence)?;
        validate_evidence(&evidence_observation_ids)?;
        let relationship_type = relationship_type.into();
        validate_non_empty("relationship_type", &relationship_type)?;

        Ok(Self {
            candidate_id: format!(
                "relationship_candidate:v1:{}:{}:{}:{}:{}",
                source.entity_kind,
                source.entity_id,
                relationship_type.trim(),
                target.entity_kind,
                target.entity_id
            ),
            source,
            target,
            relationship_type: relationship_type.trim().to_owned(),
            confidence,
            evidence_observation_ids,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RelationshipEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("relationship candidate evidence is required")]
    MissingEvidence,

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(String),
}

fn validate_subject(subject: &RelationshipSubject) -> Result<(), RelationshipEngineError> {
    validate_non_empty("entity_kind", &subject.entity_kind)?;
    validate_non_empty("entity_id", &subject.entity_id)?;
    Ok(())
}

fn validate_evidence(evidence_observation_ids: &[String]) -> Result<(), RelationshipEngineError> {
    if evidence_observation_ids.is_empty() {
        return Err(RelationshipEngineError::MissingEvidence);
    }
    for observation_id in evidence_observation_ids {
        validate_non_empty("evidence_observation_id", observation_id)?;
    }
    Ok(())
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), RelationshipEngineError> {
    if value.trim().is_empty() {
        return Err(RelationshipEngineError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), RelationshipEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(RelationshipEngineError::InvalidConfidence(
            confidence.to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_candidate_requires_evidence() {
        let error = RelationshipCandidate::linked_entities_candidate(
            RelationshipSubject::new("persona", "persona:ivan"),
            RelationshipSubject::new("organization", "org:v1:acme"),
            "works_at",
            0.77,
            vec![],
        )
        .expect_err("missing evidence must be rejected");

        assert_eq!(error, RelationshipEngineError::MissingEvidence);
    }
}
