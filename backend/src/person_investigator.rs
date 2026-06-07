use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::person_enrichment::{EnrichedPerson, PersonEnrichmentError};
use crate::person_memory::{PersonFact, PersonMemoryCard, PersonMemoryError, RelationshipEvent};

/// Assembled dossier for a person — all known context in one structure.
#[derive(Clone, Debug, Serialize)]
pub struct PersonDossier {
    pub person: EnrichedPerson,
    pub facts: Vec<PersonFact>,
    pub memory_cards: Vec<PersonMemoryCard>,
    pub timeline: Vec<RelationshipEvent>,
    pub identities: Vec<Value>,
    pub expertise: Vec<Value>,
    pub promises: Vec<Value>,
    pub risks: Vec<Value>,
    pub summary: String,
}

/// Brief meeting preparation summary.
#[derive(Clone, Debug, Serialize)]
pub struct MeetingPrep {
    pub person_id: String,
    pub display_name: String,
    pub last_interaction_days: Option<i64>,
    pub open_promises: i64,
    pub open_risks: i64,
    pub recent_topics: Vec<String>,
    pub communication_tips: Vec<String>,
    pub shared_projects: Vec<String>,
}

#[derive(Clone)]
pub struct PersonInvestigator {
    pool: PgPool,
}

impl PersonInvestigator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn assemble_dossier(
        &self,
        person_id: &str,
    ) -> Result<PersonDossier, InvestigatorError> {
        use crate::person_enrichment::PersonEnrichmentStore;
        use crate::person_memory::{
            PersonFactStore, PersonMemoryCardStore, RelationshipEventStore,
        };

        let enrichment = PersonEnrichmentStore::new(self.pool.clone());
        let facts = PersonFactStore::new(self.pool.clone());
        let cards = PersonMemoryCardStore::new(self.pool.clone());
        let timeline = RelationshipEventStore::new(self.pool.clone());

        let person = enrichment
            .get_enriched(person_id)
            .await?
            .ok_or(InvestigatorError::PersonNotFound)?;

        let facts_list = facts.list(person_id).await.unwrap_or_default();
        let cards_list = cards.list(person_id).await.unwrap_or_default();
        let timeline_list = timeline.timeline(person_id, 50).await.unwrap_or_default();

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

        Ok(PersonDossier {
            person,
            facts: facts_list,
            memory_cards: cards_list,
            timeline: timeline_list,
            identities: vec![],
            expertise: vec![],
            promises: vec![],
            risks: vec![],
            summary: summary_parts.join(" | "),
        })
    }

    pub async fn meeting_prep(&self, person_id: &str) -> Result<MeetingPrep, InvestigatorError> {
        use crate::person_enrichment::PersonEnrichmentStore;
        use crate::person_trust::{PersonPromiseStore, PersonRiskStore};

        let enrichment = PersonEnrichmentStore::new(self.pool.clone());
        let person = enrichment
            .get_enriched(person_id)
            .await?
            .ok_or(InvestigatorError::PersonNotFound)?;

        let last_interaction_days = person
            .last_interaction_at
            .map(|dt| (chrono::Utc::now() - dt).num_days());

        let promises = PersonPromiseStore::new(self.pool.clone());
        let risks = PersonRiskStore::new(self.pool.clone());
        let open_promises = promises
            .list(person_id)
            .await
            .unwrap_or_default()
            .iter()
            .filter(|p| p.status == "pending")
            .count() as i64;
        let open_risks = risks
            .list(person_id)
            .await
            .unwrap_or_default()
            .iter()
            .filter(|r| r.resolved_at.is_none())
            .count() as i64;

        let mut tips = person
            .frequent_topics
            .iter()
            .map(|t| format!("Discuss topic: {t}"))
            .collect::<Vec<_>>();
        if let Some(tone) = &person.tone {
            tips.push(format!("Match tone: {tone}"));
        }
        if let Some(style) = &person.writing_style {
            tips.push(format!("Style: {style}"));
        }

        Ok(MeetingPrep {
            person_id: person.person_id,
            display_name: person.display_name,
            last_interaction_days,
            open_promises,
            open_risks,
            recent_topics: person.frequent_topics,
            communication_tips: tips,
            shared_projects: person.linked_projects,
        })
    }
}

impl From<PersonEnrichmentError> for InvestigatorError {
    fn from(e: PersonEnrichmentError) -> Self {
        match e {
            PersonEnrichmentError::NotFound => InvestigatorError::PersonNotFound,
            PersonEnrichmentError::Sqlx(e) => InvestigatorError::Sqlx(e),
        }
    }
}

impl From<PersonMemoryError> for InvestigatorError {
    fn from(e: PersonMemoryError) -> Self {
        match e {
            PersonMemoryError::NotFound => InvestigatorError::PersonNotFound,
            PersonMemoryError::Sqlx(e) => InvestigatorError::Sqlx(e),
        }
    }
}

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("person not found")]
    PersonNotFound,
}
