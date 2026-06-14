use sqlx::Row;

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

    pub async fn toggle_favorite(&self, person_id: &str) -> Result<bool, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE persons SET is_favorite = NOT is_favorite, updated_at = now() \
             WHERE person_id = $1 RETURNING is_favorite",
        )
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
}
