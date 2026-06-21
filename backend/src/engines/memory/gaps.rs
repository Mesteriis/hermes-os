use super::errors::MemoryEngineError;
use super::models::{MemoryFactDraft, MemoryGap};
use super::validation::{validate_memory_fact, validate_non_empty};

pub(super) fn memory_gaps(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    required_fact_types: &[&str],
    facts: &[MemoryFactDraft],
) -> Result<Vec<MemoryGap>, MemoryEngineError> {
    validate_non_empty("affected entity kind", affected_entity_kind)?;
    validate_non_empty("affected entity", affected_entity_id)?;

    let affected_entity_kind = affected_entity_kind.trim();
    let affected_entity_id = affected_entity_id.trim();
    let required = unique_required_fact_types(required_fact_types)?;
    let present = accepted_fact_types_for_entity(affected_entity_kind, affected_entity_id, facts)?;

    let gaps = required
        .into_iter()
        .filter(|fact_type| !present.contains(fact_type))
        .map(|fact_type| MemoryGap {
            affected_entity_kind: affected_entity_kind.to_owned(),
            affected_entity_id: affected_entity_id.to_owned(),
            missing_fact_type: fact_type.clone(),
            source: format!(
                "memory_engine:gap:{affected_entity_kind}:{affected_entity_id}:{fact_type}"
            ),
            review_state: "suggested".to_owned(),
            produced_by: "memory_engine".to_owned(),
        })
        .collect();

    Ok(gaps)
}

fn unique_required_fact_types(
    required_fact_types: &[&str],
) -> Result<Vec<String>, MemoryEngineError> {
    let mut required = Vec::new();
    for fact_type in required_fact_types {
        validate_non_empty("fact type", fact_type)?;
        let fact_type = fact_type.trim().to_owned();
        if !required.contains(&fact_type) {
            required.push(fact_type);
        }
    }
    Ok(required)
}

fn accepted_fact_types_for_entity(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    facts: &[MemoryFactDraft],
) -> Result<Vec<String>, MemoryEngineError> {
    let mut present = Vec::new();
    for fact in facts {
        validate_memory_fact(fact)?;
        if fact.affected_entity_kind.trim() == affected_entity_kind
            && fact.affected_entity_id.trim() == affected_entity_id
            && fact.review_state.trim() == "accepted"
        {
            let fact_type = fact.fact_type.trim().to_owned();
            if !present.contains(&fact_type) {
                present.push(fact_type);
            }
        }
    }
    Ok(present)
}
