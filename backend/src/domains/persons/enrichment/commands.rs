use sqlx::Row;

use crate::domains::persons::core::link_persons_entity_in_transaction;
use crate::domains::persons::intelligence::CommunicationFingerprint;

use super::errors::PersonEnrichmentError;
use super::materialization::{
    sync_favorite_preference_in_transaction, sync_notes_memory_card_in_transaction,
};
use super::models::EnrichedPerson;
use super::relationship_materialization::materialize_trust_relationship_in_transaction;
use super::rows::{ENRICHED_PERSON_COLUMNS, row_to_enriched};
use super::store::PersonEnrichmentStore;

impl PersonEnrichmentStore {
    pub async fn enrich_person(
        &self,
        person_id: &str,
        fingerprint: &CommunicationFingerprint,
    ) -> Result<EnrichedPerson, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let sql = format!(
            "UPDATE persons SET \
             language = COALESCE($2, persons.language), \
             tone = COALESCE($3, persons.tone), \
             trust_score = COALESCE($4, persons.trust_score), \
             avg_response_hours = COALESCE($5, persons.avg_response_hours), \
             writing_style = COALESCE($6, persons.writing_style), \
             updated_at = now() \
             WHERE person_id = $1 RETURNING {ENRICHED_PERSON_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(person_id)
            .bind(fingerprint.detected_language.as_deref())
            .bind(fingerprint.typical_tone.as_deref())
            .bind(fingerprint.trust_score)
            .bind(fingerprint.avg_response_hours)
            .bind(fingerprint.writing_style.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;

        let Some(row) = row else {
            return Err(PersonEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        materialize_trust_relationship_in_transaction(&mut transaction, &enriched, fingerprint)
            .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn enrich_person_with_observation(
        &self,
        person_id: &str,
        fingerprint: &CommunicationFingerprint,
        observation_id: &str,
    ) -> Result<EnrichedPerson, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let sql = format!(
            "UPDATE persons SET \
             language = COALESCE($2, persons.language), \
             tone = COALESCE($3, persons.tone), \
             trust_score = COALESCE($4, persons.trust_score), \
             avg_response_hours = COALESCE($5, persons.avg_response_hours), \
             writing_style = COALESCE($6, persons.writing_style), \
             updated_at = now() \
             WHERE person_id = $1 RETURNING {ENRICHED_PERSON_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(person_id)
            .bind(fingerprint.detected_language.as_deref())
            .bind(fingerprint.typical_tone.as_deref())
            .bind(fingerprint.trust_score)
            .bind(fingerprint.avg_response_hours)
            .bind(fingerprint.writing_style.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;

        let Some(row) = row else {
            return Err(PersonEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        materialize_trust_relationship_in_transaction(&mut transaction, &enriched, fingerprint)
            .await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            person_id,
            Some("profile_enrichment"),
            Some(serde_json::json!({
                "manual_entrypoint": "post_person_fingerprint"
            })),
        )
        .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn toggle_favorite(&self, person_id: &str) -> Result<bool, PersonEnrichmentError> {
        self.toggle_favorite_with_source(person_id, &format!("persons.is_favorite:{person_id}"))
            .await
    }

    pub async fn toggle_favorite_with_source(
        &self,
        person_id: &str,
        source: &str,
    ) -> Result<bool, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let is_favorite =
            Self::toggle_favorite_in_transaction(&mut transaction, person_id, source).await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    pub async fn toggle_favorite_with_observation(
        &self,
        person_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let is_favorite =
            Self::toggle_favorite_in_transaction(&mut transaction, person_id, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "favorite_toggle",
            person_id,
            None,
            Some(serde_json::json!({
                "is_favorite": is_favorite
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    async fn toggle_favorite_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        person_id: &str,
        source: &str,
    ) -> Result<bool, PersonEnrichmentError> {
        let row = sqlx::query(
            "UPDATE persons SET is_favorite = NOT is_favorite, updated_at = now() \
             WHERE person_id = $1 RETURNING is_favorite",
        )
        .bind(person_id)
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let is_favorite = row.try_get("is_favorite").unwrap_or(false);
        sync_favorite_preference_in_transaction(transaction, person_id, is_favorite, source)
            .await?;
        Ok(is_favorite)
    }

    pub async fn set_notes(
        &self,
        person_id: &str,
        notes: &str,
    ) -> Result<(), PersonEnrichmentError> {
        self.set_notes_with_source(person_id, notes, &format!("persons.notes:{person_id}"))
            .await
    }

    pub async fn set_notes_with_source(
        &self,
        person_id: &str,
        notes: &str,
        source: &str,
    ) -> Result<(), PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::set_notes_in_transaction(&mut transaction, person_id, notes, source).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn set_notes_with_observation(
        &self,
        person_id: &str,
        notes: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<(), PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::set_notes_in_transaction(&mut transaction, person_id, notes, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "notes",
            person_id,
            None,
            Some(serde_json::json!({
                "manual_entrypoint": "put_person_notes"
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn set_notes_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        person_id: &str,
        notes: &str,
        source: &str,
    ) -> Result<(), PersonEnrichmentError> {
        sqlx::query("UPDATE persons SET notes = $2, updated_at = now() WHERE person_id = $1")
            .bind(person_id)
            .bind(notes)
            .execute(&mut **transaction)
            .await?;
        sync_notes_memory_card_in_transaction(transaction, person_id, notes, source).await?;
        Ok(())
    }
}
