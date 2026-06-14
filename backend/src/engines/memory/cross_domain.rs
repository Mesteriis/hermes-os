use std::cmp::Ordering;

use super::errors::MemoryEngineError;
use super::models::{
    CrossDomainMemoryContextPack, MemoryContextSource, MemoryCrossDomainContextItem,
    MemoryEntityRef,
};
use super::validation::{
    validate_memory_context_source, validate_memory_entity_ref, validate_non_empty,
};

pub(super) fn cross_domain_context_pack(
    root_entity_kind: &str,
    root_entity_id: &str,
    related_entities: &[MemoryEntityRef],
    sources: &[MemoryContextSource],
    limit: i64,
) -> Result<CrossDomainMemoryContextPack, MemoryEngineError> {
    validate_non_empty("root entity kind", root_entity_kind)?;
    validate_non_empty("root entity", root_entity_id)?;

    let root_entity_kind = root_entity_kind.trim();
    let root_entity_id = root_entity_id.trim();
    for entity in related_entities {
        validate_memory_entity_ref(entity)?;
    }

    let mut items = Vec::new();
    for source in sources {
        validate_memory_context_source(source)?;
        if source.review_state.trim() != "accepted" {
            continue;
        }

        let entity_kind = source.entity_kind.trim();
        let entity_id = source.entity_id.trim();
        let Some((entity_rank, relation_kind)) =
            context_entity_rank(root_entity_kind, root_entity_id, related_entities, source)
        else {
            continue;
        };

        items.push(RankedCrossDomainMemoryContextItem {
            entity_kind: entity_kind.to_owned(),
            entity_id: entity_id.to_owned(),
            relation_kind,
            entity_rank,
            item_kind: source.item_kind.trim().to_owned(),
            title: source.title.trim().to_owned(),
            body: source.body.trim().to_owned(),
            source: source.source.trim().to_owned(),
            confidence: source.confidence,
            review_state: source.review_state.trim().to_owned(),
        });
    }

    items.sort_by(|left, right| {
        left.entity_rank
            .cmp(&right.entity_rank)
            .then_with(|| {
                right
                    .confidence
                    .partial_cmp(&left.confidence)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.source.cmp(&right.source))
    });
    items.truncate(limit.clamp(1, 50) as usize);

    let entity_citations = entity_citations(&items);
    let source_citations = source_citations(&items);
    let confidence = aggregate_confidence(&items);
    let items = items
        .into_iter()
        .map(|item| MemoryCrossDomainContextItem {
            entity_kind: item.entity_kind,
            entity_id: item.entity_id,
            relation_kind: item.relation_kind,
            item_kind: item.item_kind,
            title: item.title,
            body: item.body,
            source: item.source,
            confidence: item.confidence,
            review_state: item.review_state,
        })
        .collect();

    Ok(CrossDomainMemoryContextPack {
        root_entity_kind: root_entity_kind.to_owned(),
        root_entity_id: root_entity_id.to_owned(),
        items,
        entity_citations,
        source_citations,
        confidence,
        produced_by: "memory_engine".to_owned(),
    })
}

struct RankedCrossDomainMemoryContextItem {
    entity_kind: String,
    entity_id: String,
    relation_kind: String,
    entity_rank: usize,
    item_kind: String,
    title: String,
    body: String,
    source: String,
    confidence: f64,
    review_state: String,
}

fn context_entity_rank(
    root_entity_kind: &str,
    root_entity_id: &str,
    related_entities: &[MemoryEntityRef],
    source: &MemoryContextSource,
) -> Option<(usize, String)> {
    let entity_kind = source.entity_kind.trim();
    let entity_id = source.entity_id.trim();
    if entity_kind == root_entity_kind && entity_id == root_entity_id {
        return Some((0, "self".to_owned()));
    }

    related_entities
        .iter()
        .enumerate()
        .find(|(_, entity)| {
            entity.entity_kind.trim() == entity_kind && entity.entity_id.trim() == entity_id
        })
        .map(|(index, entity)| (index + 1, entity.relation_kind.trim().to_owned()))
}

fn entity_citations(items: &[RankedCrossDomainMemoryContextItem]) -> Vec<String> {
    let mut entity_citations = Vec::new();
    for item in items {
        let entity_citation = format!("{}:{}", item.entity_kind, item.entity_id);
        if !entity_citations.contains(&entity_citation) {
            entity_citations.push(entity_citation);
        }
    }
    entity_citations
}

fn source_citations(items: &[RankedCrossDomainMemoryContextItem]) -> Vec<String> {
    let mut source_citations = Vec::new();
    for item in items {
        if !source_citations.contains(&item.source) {
            source_citations.push(item.source.clone());
        }
    }
    source_citations
}

fn aggregate_confidence(items: &[RankedCrossDomainMemoryContextItem]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }

    let sum: f64 = items.iter().map(|item| item.confidence).sum();
    ((sum / items.len() as f64) * 100.0).round() / 100.0
}
