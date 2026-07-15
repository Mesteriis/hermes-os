use serde_json::Value;

use crate::engines::enrichment::errors::EnrichmentEngineError;

#[derive(Clone, Debug, PartialEq)]
pub struct PreferenceDraft {
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnrichmentCandidateDraft {
    pub entity_kind: String,
    pub entity_id: String,
    pub source: String,
    pub extracted_claim: String,
    pub data: Value,
    pub confidence: f64,
    pub review_state: String,
    pub freshness: String,
    pub conflict_marker: bool,
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EnrichmentEngineError> {
    if value.trim().is_empty() {
        return Err(EnrichmentEngineError::EmptyField(field));
    }
    Ok(())
}

pub fn validate_confidence(confidence: f64) -> Result<(), EnrichmentEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(EnrichmentEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
