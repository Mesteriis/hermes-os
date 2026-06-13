use serde_json::{Value, json};
use thiserror::Error;

pub struct EnrichmentEngine;

impl EnrichmentEngine {
    pub fn persona_favorite_preference(
        person_id: &str,
        is_favorite: bool,
    ) -> Option<PreferenceDraft> {
        if !is_favorite {
            return None;
        }

        Some(PreferenceDraft {
            preference_type: "ui:favorite".to_owned(),
            value: "true".to_owned(),
            source: format!("persons.is_favorite:{person_id}"),
            confidence: 1.0,
        })
    }

    pub fn persona_observation_candidate(
        person_id: &str,
        source: &str,
        extracted_claim: &str,
        data: Value,
        confidence: f64,
    ) -> Result<EnrichmentCandidateDraft, EnrichmentEngineError> {
        validate_non_empty("affected entity", person_id)?;
        validate_non_empty("source", source)?;
        validate_non_empty("extracted claim", extracted_claim)?;
        validate_confidence(confidence)?;

        let Value::Object(mut data_object) = data else {
            return Err(EnrichmentEngineError::InvalidData);
        };
        let conflict_marker = data_object
            .get("conflict_marker")
            .or_else(|| data_object.get("conflict"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        data_object.insert(
            "_enrichment".to_owned(),
            json!({
                "affected_entity_kind": "persona",
                "affected_entity_id": person_id,
                "extracted_claim": extracted_claim,
                "source": source,
                "review_state": "pending",
                "freshness": "current",
                "conflict_marker": conflict_marker,
            }),
        );

        Ok(EnrichmentCandidateDraft {
            entity_kind: "persona".to_owned(),
            entity_id: person_id.to_owned(),
            source: source.to_owned(),
            extracted_claim: extracted_claim.to_owned(),
            data: Value::Object(data_object),
            confidence,
            review_state: "pending".to_owned(),
            freshness: "current".to_owned(),
            conflict_marker,
        })
    }
}

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

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EnrichmentEngineError> {
    if value.trim().is_empty() {
        return Err(EnrichmentEngineError::EmptyField(field));
    }
    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), EnrichmentEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(EnrichmentEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq)]
pub enum EnrichmentEngineError {
    #[error("enrichment candidate {0} must not be empty")]
    EmptyField(&'static str),
    #[error("enrichment candidate confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
    #[error("enrichment candidate data must be a JSON object")]
    InvalidData,
}
