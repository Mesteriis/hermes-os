use super::errors::MemoryEngineError;
use super::models::MemoryFactState;
use super::models::{MemoryCardDraft, MemoryContextSource, MemoryEntityRef, MemoryFactDraft};

pub(super) fn validate_memory_card(card: &MemoryCardDraft) -> Result<(), MemoryEngineError> {
    validate_non_empty("title", &card.title)?;
    validate_non_empty("description", &card.description)?;
    validate_non_empty("source", &card.source)?;
    validate_confidence(card.confidence)?;
    Ok(())
}

pub(super) fn validate_memory_fact(fact: &MemoryFactDraft) -> Result<(), MemoryEngineError> {
    validate_non_empty("affected entity kind", &fact.affected_entity_kind)?;
    validate_non_empty("affected entity", &fact.affected_entity_id)?;
    validate_non_empty("fact type", &fact.fact_type)?;
    validate_non_empty("value", &fact.value)?;
    validate_non_empty("source", &fact.source)?;
    validate_non_empty("review state", &fact.review_state)?;
    validate_non_empty("producer", &fact.produced_by)?;
    validate_confidence(fact.confidence)?;
    Ok(())
}

pub(super) fn validate_memory_fact_state(fact: &MemoryFactState) -> Result<(), MemoryEngineError> {
    validate_non_empty("affected entity kind", &fact.affected_entity_kind)?;
    validate_non_empty("affected entity", &fact.affected_entity_id)?;
    validate_non_empty("fact type", &fact.fact_type)?;
    validate_non_empty("value", &fact.value)?;
    validate_non_empty("source", &fact.source)?;
    validate_non_empty("review state", &fact.review_state)?;
    validate_confidence(fact.confidence)?;
    Ok(())
}

pub(super) fn validate_memory_entity_ref(
    entity: &MemoryEntityRef,
) -> Result<(), MemoryEngineError> {
    validate_non_empty("entity kind", &entity.entity_kind)?;
    validate_non_empty("entity", &entity.entity_id)?;
    validate_non_empty("relation kind", &entity.relation_kind)?;
    Ok(())
}

pub(super) fn validate_memory_context_source(
    source: &MemoryContextSource,
) -> Result<(), MemoryEngineError> {
    validate_non_empty("entity kind", &source.entity_kind)?;
    validate_non_empty("entity", &source.entity_id)?;
    validate_non_empty("item kind", &source.item_kind)?;
    validate_non_empty("title", &source.title)?;
    validate_non_empty("body", &source.body)?;
    validate_non_empty("source", &source.source)?;
    validate_non_empty("review state", &source.review_state)?;
    validate_confidence(source.confidence)?;
    Ok(())
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), MemoryEngineError> {
    if value.trim().is_empty() {
        return Err(MemoryEngineError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn validate_confidence(confidence: f64) -> Result<(), MemoryEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(MemoryEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
