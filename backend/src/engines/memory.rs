mod cards;
mod context;
mod cross_domain;
pub mod errors;
mod facts;
mod gaps;
pub mod models;
mod stale;
mod validation;

use chrono::{DateTime, Utc};

use errors::MemoryEngineError;
use models::{
    CrossDomainMemoryContextPack, MemoryCardDraft, MemoryContextItem, MemoryContextPack,
    MemoryContextSource, MemoryCrossDomainContextItem, MemoryEntityRef, MemoryFactDraft,
    MemoryFactState, MemoryGap, MemoryStaleCandidate,
};

pub struct MemoryEngine;

impl MemoryEngine {
    pub fn persona_notes_memory_card(persona_id: &str, notes: &str) -> Option<MemoryCardDraft> {
        cards::persona_notes_memory_card(persona_id, notes)
    }

    pub fn persona_fact_memory(
        persona_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<MemoryFactDraft, MemoryEngineError> {
        facts::persona_fact_memory(persona_id, fact_type, value, source, confidence)
    }

    pub fn context_pack(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        facts: &[MemoryFactDraft],
        cards: &[MemoryCardDraft],
        limit: i64,
    ) -> Result<MemoryContextPack, MemoryEngineError> {
        context::context_pack(
            affected_entity_kind,
            affected_entity_id,
            facts,
            cards,
            limit,
        )
    }

    pub fn memory_gaps(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        required_fact_types: &[&str],
        facts: &[MemoryFactDraft],
    ) -> Result<Vec<MemoryGap>, MemoryEngineError> {
        gaps::memory_gaps(
            affected_entity_kind,
            affected_entity_id,
            required_fact_types,
            facts,
        )
    }

    pub fn stale_memory_candidates(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        facts: &[MemoryFactState],
        as_of: DateTime<Utc>,
        stale_after_days: i64,
    ) -> Result<Vec<MemoryStaleCandidate>, MemoryEngineError> {
        stale::stale_memory_candidates(
            affected_entity_kind,
            affected_entity_id,
            facts,
            as_of,
            stale_after_days,
        )
    }

    pub fn cross_domain_context_pack(
        root_entity_kind: &str,
        root_entity_id: &str,
        related_entities: &[MemoryEntityRef],
        sources: &[MemoryContextSource],
        limit: i64,
    ) -> Result<CrossDomainMemoryContextPack, MemoryEngineError> {
        cross_domain::cross_domain_context_pack(
            root_entity_kind,
            root_entity_id,
            related_entities,
            sources,
            limit,
        )
    }
}
