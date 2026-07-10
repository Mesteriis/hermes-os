use sqlx::{Postgres, Transaction};

use crate::engines::enrichment::EnrichmentEngine;
use crate::engines::memory::MemoryEngine;

use super::errors::PersonaEnrichmentError;

pub(in crate::domains::personas::enrichment) async fn sync_notes_memory_card_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    notes: &str,
    source: &str,
) -> Result<(), PersonaEnrichmentError> {
    sqlx::query("DELETE FROM persona_memory_cards WHERE person_id = $1 AND source = $2")
        .bind(person_id)
        .bind(source)
        .execute(&mut **transaction)
        .await?;

    let Some(memory_card) = MemoryEngine::persona_notes_memory_card(person_id, notes) else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO persona_memory_cards (person_id, title, description, source, confidence, importance)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(person_id)
    .bind(memory_card.title)
    .bind(memory_card.description)
    .bind(source)
    .bind(memory_card.confidence)
    .bind(memory_card.importance)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(in crate::domains::personas::enrichment) async fn sync_favorite_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    is_favorite: bool,
    source: &str,
) -> Result<(), PersonaEnrichmentError> {
    if let Some(preference) = EnrichmentEngine::persona_favorite_preference(person_id, is_favorite)
    {
        sqlx::query(
            "INSERT INTO persona_preferences (person_id, preference_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (person_id, preference_type)
             DO UPDATE SET value = $3, source = $4, confidence = $5, updated_at = now()",
        )
        .bind(person_id)
        .bind(preference.preference_type)
        .bind(preference.value)
        .bind(source)
        .bind(preference.confidence)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM persona_preferences WHERE person_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
