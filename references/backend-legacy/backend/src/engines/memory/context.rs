use std::cmp::Ordering;

use super::errors::MemoryEngineError;
use super::models::{MemoryCardDraft, MemoryContextItem, MemoryContextPack, MemoryFactDraft};
use super::validation::{validate_memory_card, validate_memory_fact, validate_non_empty};

pub(super) fn context_pack(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    facts: &[MemoryFactDraft],
    cards: &[MemoryCardDraft],
    limit: i64,
) -> Result<MemoryContextPack, MemoryEngineError> {
    validate_non_empty("affected entity kind", affected_entity_kind)?;
    validate_non_empty("affected entity", affected_entity_id)?;

    let affected_entity_kind = affected_entity_kind.trim();
    let affected_entity_id = affected_entity_id.trim();
    let mut items = Vec::new();

    for card in cards {
        validate_memory_card(card)?;
        items.push(MemoryContextItem {
            item_kind: "memory_card".to_owned(),
            title: card.title.trim().to_owned(),
            body: card.description.trim().to_owned(),
            source: card.source.trim().to_owned(),
            confidence: card.confidence,
            review_state: "accepted".to_owned(),
        });
    }

    for fact in facts {
        validate_memory_fact(fact)?;
        if fact.affected_entity_kind.trim() != affected_entity_kind
            || fact.affected_entity_id.trim() != affected_entity_id
        {
            continue;
        }

        items.push(MemoryContextItem {
            item_kind: "fact".to_owned(),
            title: fact.fact_type.trim().to_owned(),
            body: fact.value.trim().to_owned(),
            source: fact.source.trim().to_owned(),
            confidence: fact.confidence,
            review_state: fact.review_state.trim().to_owned(),
        });
    }

    items.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.item_kind.cmp(&right.item_kind))
            .then_with(|| left.source.cmp(&right.source))
    });
    items.truncate(limit.clamp(1, 50) as usize);

    let mut source_citations = Vec::new();
    for item in &items {
        if !source_citations.contains(&item.source) {
            source_citations.push(item.source.clone());
        }
    }

    let confidence = aggregate_confidence(&items);

    Ok(MemoryContextPack {
        affected_entity_kind: affected_entity_kind.to_owned(),
        affected_entity_id: affected_entity_id.to_owned(),
        items,
        source_citations,
        confidence,
        produced_by: "memory_engine".to_owned(),
    })
}

fn aggregate_confidence(items: &[MemoryContextItem]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }

    let sum: f64 = items.iter().map(|item| item.confidence).sum();
    ((sum / items.len() as f64) * 100.0).round() / 100.0
}
