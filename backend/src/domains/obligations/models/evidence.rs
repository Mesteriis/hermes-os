use serde_json::{Value, json};

use super::super::errors::ObligationStoreError;
use super::super::validation::{validate_json_object, validate_non_empty, validate_score};
use super::source_kind::ObligationEvidenceSourceKind;

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligationEvidence {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligationEvidence {
    pub fn new(source_kind: ObligationEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
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

    pub(in crate::domains::obligations) fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}
