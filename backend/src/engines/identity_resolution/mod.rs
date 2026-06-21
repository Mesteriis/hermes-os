use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdentityResolutionSubject {
    pub entity_kind: String,
    pub entity_id: String,
}

impl IdentityResolutionSubject {
    pub fn new(entity_kind: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentityResolutionCandidate {
    pub candidate_id: String,
    pub left: IdentityResolutionSubject,
    pub right: IdentityResolutionSubject,
    pub confidence: f64,
    pub evidence_observation_ids: Vec<String>,
}

impl IdentityResolutionCandidate {
    pub fn same_entity_candidate(
        left: IdentityResolutionSubject,
        right: IdentityResolutionSubject,
        confidence: f64,
        evidence_observation_ids: Vec<String>,
    ) -> Result<Self, IdentityResolutionError> {
        validate_subject(&left)?;
        validate_subject(&right)?;
        if left == right {
            return Err(IdentityResolutionError::SameSubject);
        }
        validate_confidence(confidence)?;
        validate_evidence(&evidence_observation_ids)?;

        Ok(Self {
            candidate_id: format!(
                "identity_resolution_candidate:v1:{}:{}:{}:{}",
                left.entity_kind, left.entity_id, right.entity_kind, right.entity_id
            ),
            left,
            right,
            confidence,
            evidence_observation_ids,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum IdentityResolutionError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("identity resolution candidate must compare two different subjects")]
    SameSubject,

    #[error("identity resolution candidate evidence is required")]
    MissingEvidence,

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(String),
}

fn validate_subject(subject: &IdentityResolutionSubject) -> Result<(), IdentityResolutionError> {
    validate_non_empty("entity_kind", &subject.entity_kind)?;
    validate_non_empty("entity_id", &subject.entity_id)?;
    Ok(())
}

fn validate_evidence(evidence_observation_ids: &[String]) -> Result<(), IdentityResolutionError> {
    if evidence_observation_ids.is_empty() {
        return Err(IdentityResolutionError::MissingEvidence);
    }
    for observation_id in evidence_observation_ids {
        validate_non_empty("evidence_observation_id", observation_id)?;
    }
    Ok(())
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), IdentityResolutionError> {
    if value.trim().is_empty() {
        return Err(IdentityResolutionError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), IdentityResolutionError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(IdentityResolutionError::InvalidConfidence(
            confidence.to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_resolution_candidate_requires_evidence() {
        let error = IdentityResolutionCandidate::same_entity_candidate(
            IdentityResolutionSubject::new("persona", "person:v1:ivan-a"),
            IdentityResolutionSubject::new("persona", "person:v1:ivan-b"),
            0.82,
            vec![],
        )
        .expect_err("missing evidence must be rejected");

        assert_eq!(error, IdentityResolutionError::MissingEvidence);
    }
}
