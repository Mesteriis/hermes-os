use std::collections::BTreeSet;

use super::models::DossierSectionItem;
use crate::domains::personas::enrichment::EnrichedPersona;
use crate::domains::personas::expertise::PersonaExpertise;
use crate::domains::personas::memory::{
    PersonaFact, PersonaMemoryCard, PersonaPreference, RelationshipEvent,
};

pub(super) fn fact_section(facts: &[PersonaFact], fact_type: &str) -> Vec<DossierSectionItem> {
    facts
        .iter()
        .filter(|fact| fact.is_active && fact.fact_type == fact_type)
        .map(|fact| DossierSectionItem {
            label: fact.fact_type.clone(),
            value: fact.value.clone(),
            source_refs: vec![fact.source.clone()],
            confidence: Some(fact.confidence),
        })
        .collect()
}

pub(super) fn expertise_section(expertise: &[PersonaExpertise]) -> Vec<DossierSectionItem> {
    expertise
        .iter()
        .map(|item| DossierSectionItem {
            label: item.domain.clone().unwrap_or_else(|| "skill".to_owned()),
            value: item.skill.clone(),
            source_refs: vec![item.source.clone()],
            confidence: Some(item.confidence),
        })
        .collect()
}

pub(super) fn communication_pattern_section(
    person: &EnrichedPersona,
    preferences: &[PersonaPreference],
) -> Vec<DossierSectionItem> {
    let mut items = Vec::new();
    let root_source = format!("personas:{}", person.person_id);

    if let Some(language) = &person.language {
        items.push(DossierSectionItem {
            label: "language".to_owned(),
            value: language.clone(),
            source_refs: vec![root_source.clone()],
            confidence: None,
        });
    }
    if let Some(tone) = &person.tone {
        items.push(DossierSectionItem {
            label: "tone".to_owned(),
            value: tone.clone(),
            source_refs: vec![root_source.clone()],
            confidence: None,
        });
    }
    if let Some(writing_style) = &person.writing_style {
        items.push(DossierSectionItem {
            label: "writing_style".to_owned(),
            value: writing_style.clone(),
            source_refs: vec![root_source.clone()],
            confidence: None,
        });
    }

    for preference in preferences {
        if preference.preference_type.starts_with("communication:")
            || preference
                .preference_type
                .starts_with("interaction_context:")
        {
            items.push(DossierSectionItem {
                label: preference.preference_type.clone(),
                value: preference.value.clone(),
                source_refs: vec![preference.source.clone()],
                confidence: Some(preference.confidence),
            });
        }
    }

    items
}

pub(super) fn ai_observation_section(cards: &[PersonaMemoryCard]) -> Vec<DossierSectionItem> {
    cards
        .iter()
        .filter(|card| card.source.contains("ai") || card.title.to_lowercase().contains("ai"))
        .map(|card| DossierSectionItem {
            label: card.title.clone(),
            value: card.description.clone(),
            source_refs: vec![card.source.clone()],
            confidence: Some(card.confidence),
        })
        .collect()
}

pub(super) fn dossier_source_refs(
    facts: &[PersonaFact],
    cards: &[PersonaMemoryCard],
    preferences: &[PersonaPreference],
    timeline: &[RelationshipEvent],
    expertise: &[PersonaExpertise],
) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for fact in facts {
        add_source_ref(&mut refs, &fact.source);
    }
    for card in cards {
        add_source_ref(&mut refs, &card.source);
    }
    for preference in preferences {
        add_source_ref(&mut refs, &preference.source);
    }
    for event in timeline {
        add_source_ref(&mut refs, &event.source);
    }
    for item in expertise {
        add_source_ref(&mut refs, &item.source);
    }
    refs.into_iter().collect()
}

fn add_source_ref(refs: &mut BTreeSet<String>, source: &str) {
    let source = source.trim();
    if !source.is_empty() {
        refs.insert(source.to_owned());
    }
}
