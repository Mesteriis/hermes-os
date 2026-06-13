use thiserror::Error;

pub struct MemoryEngine;

impl MemoryEngine {
    pub fn persona_notes_memory_card(person_id: &str, notes: &str) -> Option<MemoryCardDraft> {
        let description = notes.trim();
        if description.is_empty() {
            return None;
        }

        Some(MemoryCardDraft {
            title: "Compatibility notes".to_owned(),
            description: description.to_owned(),
            source: format!("persons.notes:{person_id}"),
            confidence: 1.0,
            importance: 5,
        })
    }

    pub fn persona_fact_memory(
        person_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<MemoryFactDraft, MemoryEngineError> {
        validate_non_empty("affected entity", person_id)?;
        validate_non_empty("fact type", fact_type)?;
        validate_non_empty("value", value)?;
        validate_non_empty("source", source)?;
        validate_confidence(confidence)?;

        Ok(MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: person_id.trim().to_owned(),
            fact_type: fact_type.trim().to_owned(),
            value: value.trim().to_owned(),
            source: source.trim().to_owned(),
            confidence,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCardDraft {
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryFactDraft {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
    pub produced_by: String,
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), MemoryEngineError> {
    if value.trim().is_empty() {
        return Err(MemoryEngineError::EmptyField(field));
    }
    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), MemoryEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(MemoryEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq)]
pub enum MemoryEngineError {
    #[error("memory {0} must not be empty")]
    EmptyField(&'static str),
    #[error("memory confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
}
