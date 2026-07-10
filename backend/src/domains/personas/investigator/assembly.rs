use chrono::Utc;
use sqlx::postgres::PgPool;

use super::errors::InvestigatorError;
use super::models::PersonaDossier;
use super::sections::{
    ai_observation_section, communication_pattern_section, dossier_source_refs, expertise_section,
    fact_section,
};
use crate::domains::personas::enrichment::PersonaEnrichmentStore;
use crate::domains::personas::expertise::PersonaExpertiseStore;
use crate::domains::personas::memory::{
    PersonaFactStore, PersonaMemoryCardStore, PersonaPreferenceStore, RelationshipEventStore,
};

pub(super) async fn assemble_dossier(
    pool: &PgPool,
    person_id: &str,
) -> Result<PersonaDossier, InvestigatorError> {
    let enrichment = PersonaEnrichmentStore::new(pool.clone());
    let facts = PersonaFactStore::new(pool.clone());
    let cards = PersonaMemoryCardStore::new(pool.clone());
    let preferences = PersonaPreferenceStore::new(pool.clone());
    let timeline = RelationshipEventStore::new(pool.clone());
    let expertise = PersonaExpertiseStore::new(pool.clone());

    let person = enrichment
        .get_enriched(person_id)
        .await?
        .ok_or(InvestigatorError::PersonaNotFound)?;

    let facts_list = facts.list(person_id).await.unwrap_or_default();
    let cards_list = cards.list(person_id).await.unwrap_or_default();
    let preferences_list = preferences.list(person_id).await.unwrap_or_default();
    let timeline_list = timeline.timeline(person_id, 50).await.unwrap_or_default();
    let expertise_list = expertise.list(person_id).await.unwrap_or_default();

    let mut summary_parts: Vec<String> = Vec::new();
    if let Some(tone) = &person.tone {
        summary_parts.push(format!("Tone: {tone}"));
    }
    if let Some(lang) = &person.language {
        summary_parts.push(format!("Language: {lang}"));
    }
    if person.interaction_count > 0 {
        summary_parts.push(format!("{} interactions", person.interaction_count));
    }
    if !person.frequent_topics.is_empty() {
        summary_parts.push(format!("Topics: {}", person.frequent_topics.join(", ")));
    }
    for card in &cards_list {
        if card.importance >= 7 {
            summary_parts.push(format!("Key: {}", card.title));
        }
    }

    let interests = fact_section(&facts_list, "interest");
    let projects = fact_section(&facts_list, "project");
    let organizations = fact_section(&facts_list, "organization");
    let skills = expertise_section(&expertise_list);
    let communication_patterns = communication_pattern_section(&person, &preferences_list);
    let ai_observations = ai_observation_section(&cards_list);
    let source_refs = dossier_source_refs(
        &facts_list,
        &cards_list,
        &preferences_list,
        &timeline_list,
        &expertise_list,
    );

    Ok(PersonaDossier {
        persona: person,
        facts: facts_list,
        memory_cards: cards_list,
        timeline: timeline_list,
        identities: vec![],
        expertise: vec![],
        promises: vec![],
        risks: vec![],
        summary: summary_parts.join(" | "),
        interests,
        projects,
        organizations,
        skills,
        communication_patterns,
        ai_observations,
        source_refs,
        generated_at: Utc::now(),
    })
}
