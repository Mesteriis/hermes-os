use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonaEnrichmentError;
use super::models::EnrichedPersona;

pub(in crate::domains::personas::enrichment) const ENRICHED_PERSONA_COLUMNS: &str = "persona_id, display_name, email_address, language, tone, trust_score, avg_response_hours, \
     preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, \
     persona_metadata, is_favorite, is_address_book, notes, created_at, updated_at";

pub(in crate::domains::personas::enrichment) fn row_to_enriched(
    row: PgRow,
) -> Result<EnrichedPersona, PersonaEnrichmentError> {
    Ok(EnrichedPersona {
        persona_id: row.try_get("persona_id")?,
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
        persona_metadata: row.try_get("persona_metadata")?,
        is_favorite: row.try_get("is_favorite")?,
        is_address_book: row.try_get("is_address_book")?,
        notes: row.try_get("notes")?,
        linked_projects: vec![],
        linked_documents: vec![],
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
