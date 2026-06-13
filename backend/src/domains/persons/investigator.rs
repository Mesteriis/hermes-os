use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::persons::enrichment::{EnrichedPerson, PersonEnrichmentError};
use crate::domains::persons::expertise::PersonExpertise;
use crate::domains::persons::memory::{
    PersonFact, PersonMemoryCard, PersonMemoryError, PersonPreference, RelationshipEvent,
};
use crate::domains::relationships::RelationshipStoreError;

/// Source-backed derived item inside a Persona dossier section.
#[derive(Clone, Debug, Serialize)]
pub struct DossierSectionItem {
    pub label: String,
    pub value: String,
    pub source_refs: Vec<String>,
    pub confidence: Option<f64>,
}

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
    pub interests: Vec<DossierSectionItem>,
    pub projects: Vec<DossierSectionItem>,
    pub organizations: Vec<DossierSectionItem>,
    pub skills: Vec<DossierSectionItem>,
    pub communication_patterns: Vec<DossierSectionItem>,
    pub ai_observations: Vec<DossierSectionItem>,
    pub source_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DossierReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl DossierReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: &str) -> Result<Self, InvestigatorError> {
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(InvestigatorError::InvalidDossierReviewState),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DossierSnapshot {
    pub dossier_snapshot_id: String,
    pub persona_id: String,
    pub dossier: Value,
    pub source_refs: Value,
    pub review_state: DossierReviewState,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
        use crate::domains::persons::enrichment::PersonEnrichmentStore;
        use crate::domains::persons::expertise::PersonExpertiseStore;
        use crate::domains::persons::memory::{
            PersonFactStore, PersonMemoryCardStore, PersonPreferenceStore, RelationshipEventStore,
        };

        let enrichment = PersonEnrichmentStore::new(self.pool.clone());
        let facts = PersonFactStore::new(self.pool.clone());
        let cards = PersonMemoryCardStore::new(self.pool.clone());
        let preferences = PersonPreferenceStore::new(self.pool.clone());
        let timeline = RelationshipEventStore::new(self.pool.clone());
        let expertise = PersonExpertiseStore::new(self.pool.clone());

        let person = enrichment
            .get_enriched(person_id)
            .await?
            .ok_or(InvestigatorError::PersonNotFound)?;

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

    pub async fn assemble_and_cache_dossier(
        &self,
        person_id: &str,
    ) -> Result<(PersonDossier, DossierSnapshot), InvestigatorError> {
        let dossier = self.assemble_dossier(person_id).await?;
        let snapshot = self.cache_dossier_snapshot(&dossier).await?;
        Ok((dossier, snapshot))
    }

    pub async fn cache_dossier_snapshot(
        &self,
        dossier: &PersonDossier,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        let dossier_value = serde_json::to_value(dossier)?;
        let source_refs = serde_json::to_value(&dossier.source_refs)?;
        let snapshot_id = dossier_snapshot_id(&dossier.person.person_id);
        let row = sqlx::query(
            r#"
            INSERT INTO persona_dossier_snapshots (
                dossier_snapshot_id,
                persona_id,
                dossier,
                source_refs,
                review_state,
                generated_at
            )
            VALUES ($1, $2, $3, $4, 'suggested', $5)
            ON CONFLICT (persona_id)
            DO UPDATE SET
                dossier = EXCLUDED.dossier,
                source_refs = EXCLUDED.source_refs,
                generated_at = EXCLUDED.generated_at,
                updated_at = now()
            RETURNING
                dossier_snapshot_id,
                persona_id,
                dossier,
                source_refs,
                review_state,
                reviewed_by,
                reviewed_at,
                metadata,
                generated_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&snapshot_id)
        .bind(&dossier.person.person_id)
        .bind(dossier_value)
        .bind(source_refs)
        .bind(dossier.generated_at)
        .fetch_one(&self.pool)
        .await?;

        row_to_dossier_snapshot(row)
    }

    pub async fn review_dossier_snapshot(
        &self,
        person_id: &str,
        review_state: DossierReviewState,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        let row = sqlx::query(
            r#"
            UPDATE persona_dossier_snapshots
            SET
                review_state = $2,
                reviewed_by = 'owner_persona',
                reviewed_at = now(),
                updated_at = now()
            WHERE persona_id = $1
            RETURNING
                dossier_snapshot_id,
                persona_id,
                dossier,
                source_refs,
                review_state,
                reviewed_by,
                reviewed_at,
                metadata,
                generated_at,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(review_state.as_str())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(InvestigatorError::DossierSnapshotNotFound)?;

        row_to_dossier_snapshot(row)
    }

    pub async fn meeting_prep(&self, person_id: &str) -> Result<MeetingPrep, InvestigatorError> {
        use crate::domains::persons::enrichment::PersonEnrichmentStore;
        use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore};

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

fn dossier_snapshot_id(person_id: &str) -> String {
    format!("persona_dossier:v1:{person_id}")
}

fn row_to_dossier_snapshot(row: PgRow) -> Result<DossierSnapshot, InvestigatorError> {
    Ok(DossierSnapshot {
        dossier_snapshot_id: row.try_get("dossier_snapshot_id")?,
        persona_id: row.try_get("persona_id")?,
        dossier: row.try_get("dossier")?,
        source_refs: row.try_get("source_refs")?,
        review_state: DossierReviewState::parse(
            row.try_get::<String, _>("review_state")?.as_str(),
        )?,
        reviewed_by: row.try_get("reviewed_by")?,
        reviewed_at: row.try_get("reviewed_at")?,
        metadata: row.try_get("metadata")?,
        generated_at: row.try_get("generated_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn fact_section(facts: &[PersonFact], fact_type: &str) -> Vec<DossierSectionItem> {
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

fn expertise_section(expertise: &[PersonExpertise]) -> Vec<DossierSectionItem> {
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

fn communication_pattern_section(
    person: &EnrichedPerson,
    preferences: &[PersonPreference],
) -> Vec<DossierSectionItem> {
    let mut items = Vec::new();
    let root_source = format!("persons:{}", person.person_id);

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

fn ai_observation_section(cards: &[PersonMemoryCard]) -> Vec<DossierSectionItem> {
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

fn dossier_source_refs(
    facts: &[PersonFact],
    cards: &[PersonMemoryCard],
    preferences: &[PersonPreference],
    timeline: &[RelationshipEvent],
    expertise: &[PersonExpertise],
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

impl From<PersonEnrichmentError> for InvestigatorError {
    fn from(e: PersonEnrichmentError) -> Self {
        match e {
            PersonEnrichmentError::NotFound => InvestigatorError::PersonNotFound,
            PersonEnrichmentError::Sqlx(e) => InvestigatorError::Sqlx(e),
            PersonEnrichmentError::Relationship(e) => InvestigatorError::Relationship(e),
            PersonEnrichmentError::Trust(e) => InvestigatorError::Trust(e),
        }
    }
}

impl From<PersonMemoryError> for InvestigatorError {
    fn from(e: PersonMemoryError) -> Self {
        match e {
            PersonMemoryError::NotFound => InvestigatorError::PersonNotFound,
            PersonMemoryError::Sqlx(e) => InvestigatorError::Sqlx(e),
            PersonMemoryError::Timeline(e) => InvestigatorError::Timeline(e),
        }
    }
}

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error(transparent)]
    Trust(#[from] crate::engines::trust::TrustEngineError),
    #[error("person not found")]
    PersonNotFound,
    #[error("dossier snapshot not found")]
    DossierSnapshotNotFound,
    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidDossierReviewState,
}
