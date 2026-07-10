use serde_json::{Value, json};

use crate::engines::enrichment::errors::EnrichmentEngineError;
use crate::engines::enrichment::models::{
    EnrichmentCandidateDraft, PreferenceDraft, validate_confidence, validate_non_empty,
};

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
            source: format!("personas.is_favorite:{person_id}"),
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
