use serde_json::{Value, json};

use super::super::errors::ObligationStoreError;
use super::super::validation::{validate_json_object, validate_non_empty, validate_score};
use super::source_kind::ObligationEvidenceSourceKind;

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligationEvidence {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligationEvidence {
    pub fn new(source_kind: ObligationEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            observation_id: None,
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn observation(observation_id: impl Into<String>) -> Self {
        let observation_id = observation_id.into();
        Self {
            source_kind: ObligationEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_observation_id<T: Into<String>>(mut self, observation_id: Option<T>) -> Self {
        self.observation_id = observation_id.map(Into::into);
        self
    }

    pub(in crate::domains::obligations) fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        if self.source_kind == ObligationEvidenceSourceKind::Observation
            && self.observation_id.as_deref() != Some(self.source_id.as_str())
        {
            return Err(ObligationStoreError::InvalidObservationEvidenceSource);
        }
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}
