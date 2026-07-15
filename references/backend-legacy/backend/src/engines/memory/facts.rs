use super::errors::MemoryEngineError;
use super::models::MemoryFactDraft;
use super::validation::{validate_confidence, validate_non_empty};

pub(super) fn persona_fact_memory(
    persona_id: &str,
    fact_type: &str,
    value: &str,
    source: &str,
    confidence: f64,
) -> Result<MemoryFactDraft, MemoryEngineError> {
    validate_non_empty("affected entity", persona_id)?;
    validate_non_empty("fact type", fact_type)?;
    validate_non_empty("value", value)?;
    validate_non_empty("source", source)?;
    validate_confidence(confidence)?;

    Ok(MemoryFactDraft {
        affected_entity_kind: "persona".to_owned(),
        affected_entity_id: persona_id.trim().to_owned(),
        fact_type: fact_type.trim().to_owned(),
        value: value.trim().to_owned(),
        source: source.trim().to_owned(),
        confidence,
        review_state: "accepted".to_owned(),
        produced_by: "memory_engine".to_owned(),
    })
}
