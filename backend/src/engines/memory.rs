use std::cmp::Ordering;

use chrono::{DateTime, Utc};
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

    pub fn context_pack(
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

        let confidence = if items.is_empty() {
            0.0
        } else {
            let sum: f64 = items.iter().map(|item| item.confidence).sum();
            ((sum / items.len() as f64) * 100.0).round() / 100.0
        };

        Ok(MemoryContextPack {
            affected_entity_kind: affected_entity_kind.to_owned(),
            affected_entity_id: affected_entity_id.to_owned(),
            items,
            source_citations,
            confidence,
            produced_by: "memory_engine".to_owned(),
        })
    }

    pub fn memory_gaps(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        required_fact_types: &[&str],
        facts: &[MemoryFactDraft],
    ) -> Result<Vec<MemoryGap>, MemoryEngineError> {
        validate_non_empty("affected entity kind", affected_entity_kind)?;
        validate_non_empty("affected entity", affected_entity_id)?;

        let affected_entity_kind = affected_entity_kind.trim();
        let affected_entity_id = affected_entity_id.trim();
        let mut required = Vec::new();
        for fact_type in required_fact_types {
            validate_non_empty("fact type", fact_type)?;
            let fact_type = fact_type.trim().to_owned();
            if !required.contains(&fact_type) {
                required.push(fact_type);
            }
        }

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

        let mut gaps = Vec::new();
        for fact_type in required {
            if present.contains(&fact_type) {
                continue;
            }

            gaps.push(MemoryGap {
                affected_entity_kind: affected_entity_kind.to_owned(),
                affected_entity_id: affected_entity_id.to_owned(),
                missing_fact_type: fact_type.clone(),
                source: format!(
                    "memory_engine:gap:{affected_entity_kind}:{affected_entity_id}:{fact_type}"
                ),
                review_state: "suggested".to_owned(),
                produced_by: "memory_engine".to_owned(),
            });
        }

        Ok(gaps)
    }

    pub fn stale_memory_candidates(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        facts: &[MemoryFactState],
        as_of: DateTime<Utc>,
        stale_after_days: i64,
    ) -> Result<Vec<MemoryStaleCandidate>, MemoryEngineError> {
        validate_non_empty("affected entity kind", affected_entity_kind)?;
        validate_non_empty("affected entity", affected_entity_id)?;
        if stale_after_days <= 0 {
            return Err(MemoryEngineError::InvalidStaleThreshold);
        }

        let affected_entity_kind = affected_entity_kind.trim();
        let affected_entity_id = affected_entity_id.trim();
        let stale_cutoff = as_of - chrono::Duration::days(stale_after_days);
        let mut candidates = Vec::new();

        for fact in facts {
            validate_memory_fact_state(fact)?;
            if fact.affected_entity_kind.trim() != affected_entity_kind
                || fact.affected_entity_id.trim() != affected_entity_id
                || fact.review_state.trim() != "accepted"
            {
                continue;
            }

            if fact
                .last_verified_at
                .is_some_and(|verified_at| verified_at >= stale_cutoff)
            {
                continue;
            }

            candidates.push(MemoryStaleCandidate {
                affected_entity_kind: affected_entity_kind.to_owned(),
                affected_entity_id: affected_entity_id.to_owned(),
                fact_type: fact.fact_type.trim().to_owned(),
                value: fact.value.trim().to_owned(),
                source: fact.source.trim().to_owned(),
                confidence: fact.confidence,
                last_verified_at: fact.last_verified_at,
                review_state: "suggested".to_owned(),
                produced_by: "memory_engine".to_owned(),
            });
        }

        candidates.sort_by(|left, right| {
            left.last_verified_at
                .cmp(&right.last_verified_at)
                .then_with(|| left.source.cmp(&right.source))
        });

        Ok(candidates)
    }

    pub fn cross_domain_context_pack(
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

            items.push(CrossDomainMemoryContextItem {
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

        let mut entity_citations = Vec::new();
        let mut source_citations = Vec::new();
        for item in &items {
            let entity_citation = format!("{}:{}", item.entity_kind, item.entity_id);
            if !entity_citations.contains(&entity_citation) {
                entity_citations.push(entity_citation);
            }
            if !source_citations.contains(&item.source) {
                source_citations.push(item.source.clone());
            }
        }

        let confidence = if items.is_empty() {
            0.0
        } else {
            let sum: f64 = items.iter().map(|item| item.confidence).sum();
            ((sum / items.len() as f64) * 100.0).round() / 100.0
        };

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

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryFactState {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextPack {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub items: Vec<MemoryContextItem>,
    pub source_citations: Vec<String>,
    pub confidence: f64,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextItem {
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryGap {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub missing_fact_type: String,
    pub source: String,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryStaleCandidate {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryEntityRef {
    pub entity_kind: String,
    pub entity_id: String,
    pub relation_kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextSource {
    pub entity_kind: String,
    pub entity_id: String,
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrossDomainMemoryContextPack {
    pub root_entity_kind: String,
    pub root_entity_id: String,
    pub items: Vec<MemoryCrossDomainContextItem>,
    pub entity_citations: Vec<String>,
    pub source_citations: Vec<String>,
    pub confidence: f64,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCrossDomainContextItem {
    pub entity_kind: String,
    pub entity_id: String,
    pub relation_kind: String,
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

struct CrossDomainMemoryContextItem {
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

fn validate_memory_card(card: &MemoryCardDraft) -> Result<(), MemoryEngineError> {
    validate_non_empty("title", &card.title)?;
    validate_non_empty("description", &card.description)?;
    validate_non_empty("source", &card.source)?;
    validate_confidence(card.confidence)?;
    Ok(())
}

fn validate_memory_fact(fact: &MemoryFactDraft) -> Result<(), MemoryEngineError> {
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

fn validate_memory_fact_state(fact: &MemoryFactState) -> Result<(), MemoryEngineError> {
    validate_non_empty("affected entity kind", &fact.affected_entity_kind)?;
    validate_non_empty("affected entity", &fact.affected_entity_id)?;
    validate_non_empty("fact type", &fact.fact_type)?;
    validate_non_empty("value", &fact.value)?;
    validate_non_empty("source", &fact.source)?;
    validate_non_empty("review state", &fact.review_state)?;
    validate_confidence(fact.confidence)?;
    Ok(())
}

fn validate_memory_entity_ref(entity: &MemoryEntityRef) -> Result<(), MemoryEngineError> {
    validate_non_empty("entity kind", &entity.entity_kind)?;
    validate_non_empty("entity", &entity.entity_id)?;
    validate_non_empty("relation kind", &entity.relation_kind)?;
    Ok(())
}

fn validate_memory_context_source(source: &MemoryContextSource) -> Result<(), MemoryEngineError> {
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
    #[error("memory stale threshold days must be greater than zero")]
    InvalidStaleThreshold,
}
