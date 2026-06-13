use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::persons::intelligence::CommunicationFingerprint;
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
    RelationshipStoreError,
};
use crate::engines::enrichment::EnrichmentEngine;
use crate::engines::memory::MemoryEngine;
use crate::engines::trust::{TrustEngine, TrustEngineError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedPerson {
    pub person_id: String,
    pub display_name: String,
    pub email_address: String,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub trust_score: Option<i16>,
    pub avg_response_hours: Option<f64>,
    pub preferred_channel: Option<String>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub interaction_count: i32,
    pub frequent_topics: Vec<String>,
    pub writing_style: Option<String>,
    pub person_metadata: Value,
    pub is_favorite: bool,
    pub notes: Option<String>,
    pub linked_projects: Vec<String>,
    pub linked_documents: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonEnrichmentStore {
    pool: PgPool,
}

impl PersonEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enrich_person(
        &self,
        person_id: &str,
        fingerprint: &CommunicationFingerprint,
    ) -> Result<EnrichedPerson, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE persons SET
                language = COALESCE($2, persons.language),
                tone = COALESCE($3, persons.tone),
                trust_score = COALESCE($4, persons.trust_score),
                avg_response_hours = COALESCE($5, persons.avg_response_hours),
                writing_style = COALESCE($6, persons.writing_style),
                updated_at = now()
            WHERE person_id = $1
            RETURNING person_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, person_metadata, is_favorite, notes, created_at, updated_at"#,
        ).bind(person_id).bind(fingerprint.detected_language.as_deref()).bind(fingerprint.typical_tone.as_deref()).bind(fingerprint.trust_score).bind(fingerprint.avg_response_hours).bind(fingerprint.writing_style.as_deref()).fetch_optional(&mut *transaction).await?;

        let Some(row) = row else {
            return Err(PersonEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        Self::materialize_trust_relationship_in_transaction(
            &mut transaction,
            &enriched,
            fingerprint,
        )
        .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn get_enriched(
        &self,
        person_id: &str,
    ) -> Result<Option<EnrichedPerson>, PersonEnrichmentError> {
        let row = sqlx::query(
            r#"SELECT c.person_id, c.display_name, c.email_address, c.language, c.tone, c.trust_score, c.avg_response_hours, c.preferred_channel, c.last_interaction_at, c.interaction_count, c.frequent_topics, c.writing_style, c.person_metadata, c.is_favorite, c.notes, c.created_at, c.updated_at
            FROM persons c WHERE c.person_id = $1"#,
        ).bind(person_id).fetch_optional(&self.pool).await?;
        row.map(row_to_enriched).transpose()
    }

    pub async fn list_enriched(
        &self,
        favorites_only: bool,
        limit: i64,
    ) -> Result<Vec<EnrichedPerson>, PersonEnrichmentError> {
        let limit = limit.clamp(1, 100);
        let rows = if favorites_only {
            sqlx::query("SELECT person_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, person_metadata, is_favorite, notes, created_at, updated_at FROM persons WHERE is_favorite = true ORDER BY trust_score DESC NULLS LAST, interaction_count DESC LIMIT $1").bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT person_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, person_metadata, is_favorite, notes, created_at, updated_at FROM persons ORDER BY interaction_count DESC, trust_score DESC NULLS LAST LIMIT $1").bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter().map(row_to_enriched).collect()
    }

    pub async fn toggle_favorite(&self, person_id: &str) -> Result<bool, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("UPDATE persons SET is_favorite = NOT is_favorite, updated_at = now() WHERE person_id = $1 RETURNING is_favorite")
            .bind(person_id)
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let is_favorite = row.try_get("is_favorite").unwrap_or(false);
        sync_favorite_preference_in_transaction(&mut transaction, person_id, is_favorite).await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    pub async fn set_notes(
        &self,
        person_id: &str,
        notes: &str,
    ) -> Result<(), PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("UPDATE persons SET notes = $2, updated_at = now() WHERE person_id = $1")
            .bind(person_id)
            .bind(notes)
            .execute(&mut *transaction)
            .await?;
        sync_notes_memory_card_in_transaction(&mut transaction, person_id, notes).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn search_persons(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<EnrichedPerson>, PersonEnrichmentError> {
        let pattern = format!("%{}%", query.trim().to_lowercase());
        let rows = sqlx::query("SELECT person_id, display_name, email_address, language, tone, trust_score, avg_response_hours, preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, person_metadata, is_favorite, notes, created_at, updated_at FROM persons WHERE lower(display_name) LIKE $1 OR lower(email_address) LIKE $1 ORDER BY interaction_count DESC LIMIT $2").bind(&pattern).bind(limit.clamp(1, 100)).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enriched).collect()
    }
}

async fn owner_persona_id_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Option<String>, PersonEnrichmentError> {
    let row = sqlx::query("SELECT person_id FROM persons WHERE is_self = true")
        .fetch_optional(&mut **transaction)
        .await?;
    row.map(|row| row.try_get("person_id"))
        .transpose()
        .map_err(Into::into)
}

async fn sync_notes_memory_card_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    notes: &str,
) -> Result<(), PersonEnrichmentError> {
    let source = format!("persons.notes:{person_id}");
    sqlx::query("DELETE FROM person_memory_cards WHERE person_id = $1 AND source = $2")
        .bind(person_id)
        .bind(&source)
        .execute(&mut **transaction)
        .await?;

    let Some(memory_card) = MemoryEngine::persona_notes_memory_card(person_id, notes) else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO person_memory_cards (person_id, title, description, source, confidence, importance)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(person_id)
    .bind(memory_card.title)
    .bind(memory_card.description)
    .bind(memory_card.source)
    .bind(memory_card.confidence)
    .bind(memory_card.importance)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn sync_favorite_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    is_favorite: bool,
) -> Result<(), PersonEnrichmentError> {
    if let Some(preference) = EnrichmentEngine::persona_favorite_preference(person_id, is_favorite)
    {
        sqlx::query(
            "INSERT INTO person_preferences (person_id, preference_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (person_id, preference_type)
             DO UPDATE SET value = $3, source = $4, confidence = $5, updated_at = now()",
        )
        .bind(person_id)
        .bind(preference.preference_type)
        .bind(preference.value)
        .bind(preference.source)
        .bind(preference.confidence)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

impl PersonEnrichmentStore {
    async fn materialize_trust_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        enriched: &EnrichedPerson,
        fingerprint: &CommunicationFingerprint,
    ) -> Result<(), PersonEnrichmentError> {
        let Some(trust_score) = fingerprint.trust_score else {
            return Ok(());
        };
        let Some(owner_persona_id) = owner_persona_id_in_transaction(transaction).await? else {
            return Ok(());
        };
        if owner_persona_id == enriched.person_id {
            return Ok(());
        }

        let trust_signal = TrustEngine::persona_compatibility_score_signal(trust_score);
        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: owner_persona_id.clone(),
            target_entity_kind: RelationshipEntityKind::Persona,
            target_entity_id: enriched.person_id.clone(),
            relationship_type: trust_signal.relationship_type.to_owned(),
            trust_score: trust_signal.trust_score,
            strength_score: trust_signal.strength_score,
            confidence: trust_signal.confidence,
            review_state: RelationshipReviewState::Suggested,
            valid_from: None,
            valid_to: None,
            metadata: json!({
                "compatibility_source": "persons.trust_score",
                "source": "person_enrichment",
                "owner_persona_id": owner_persona_id,
                "person_id": enriched.person_id,
                "trust_score": trust_score,
            }),
        };
        let evidence_source_id = format!("person_enrichment:{}:trust_score", enriched.person_id);
        let evidence_excerpt = format!("trust_score={trust_score}");
        let source_reliability = TrustEngine::source_reliability_signal(
            &evidence_source_id,
            &evidence_excerpt,
            trust_signal.trust_score,
        )?;
        let evidence = NewRelationshipEvidence::new(
            RelationshipEvidenceSourceKind::RawRecord,
            evidence_source_id.clone(),
        )
        .excerpt(evidence_excerpt)
        .metadata(json!({
            "compatibility_source": "persons.trust_score",
            "source": "person_enrichment",
            "person_id": enriched.person_id,
            "trust_score": trust_score,
            "detected_language": fingerprint.detected_language,
            "typical_tone": fingerprint.typical_tone,
            "writing_style": fingerprint.writing_style,
            "trust_source_reliability": {
                "signal_type": source_reliability.kind.as_str(),
                "affected_source": source_reliability.affected_source,
                "evidence": source_reliability.evidence,
                "confidence": source_reliability.confidence,
                "direction": source_reliability.direction.as_str(),
                "explanation": source_reliability.explanation,
            },
        }));

        RelationshipStore::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;

        Ok(())
    }
}

fn row_to_enriched(row: PgRow) -> Result<EnrichedPerson, PersonEnrichmentError> {
    Ok(EnrichedPerson {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        language: row.try_get("language")?,
        tone: row.try_get("tone")?,
        trust_score: row.try_get("trust_score")?,
        avg_response_hours: row.try_get("avg_response_hours")?,
        preferred_channel: row.try_get("preferred_channel")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
        interaction_count: row.try_get("interaction_count")?,
        frequent_topics: serde_json::from_value(row.try_get("frequent_topics")?)
            .unwrap_or_default(),
        writing_style: row.try_get("writing_style")?,
        person_metadata: row.try_get("person_metadata")?,
        is_favorite: row.try_get("is_favorite")?,
        notes: row.try_get("notes")?,
        linked_projects: vec![],
        linked_documents: vec![],
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum PersonEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error(transparent)]
    Trust(#[from] TrustEngineError),
    #[error("person not found")]
    NotFound,
}
