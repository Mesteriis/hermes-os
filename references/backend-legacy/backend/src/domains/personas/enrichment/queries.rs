use super::errors::PersonaEnrichmentError;
use super::models::EnrichedPersona;
use super::rows::{ENRICHED_PERSONA_COLUMNS, row_to_enriched};
use super::store::PersonaEnrichmentStore;

impl PersonaEnrichmentStore {
    pub async fn get_enriched(
        &self,
        persona_id: &str,
    ) -> Result<Option<EnrichedPersona>, PersonaEnrichmentError> {
        let sql = format!("SELECT {ENRICHED_PERSONA_COLUMNS} FROM personas WHERE persona_id = $1");
        let row = sqlx::query(&sql)
            .bind(persona_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_enriched).transpose()
    }

    pub async fn list_enriched(
        &self,
        favorites_only: bool,
        limit: i64,
    ) -> Result<Vec<EnrichedPersona>, PersonaEnrichmentError> {
        let limit = limit.clamp(1, 100);
        let sql = if favorites_only {
            format!(
                "SELECT {ENRICHED_PERSONA_COLUMNS} FROM personas WHERE is_favorite = true \
                 ORDER BY trust_score DESC NULLS LAST, interaction_count DESC LIMIT $1"
            )
        } else {
            format!(
                "SELECT {ENRICHED_PERSONA_COLUMNS} FROM personas \
                 ORDER BY interaction_count DESC, trust_score DESC NULLS LAST LIMIT $1"
            )
        };
        let rows = sqlx::query(&sql).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enriched).collect()
    }

    pub async fn search_personas(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<EnrichedPersona>, PersonaEnrichmentError> {
        let pattern = format!("%{}%", query.trim().to_lowercase());
        let sql = format!(
            "SELECT {ENRICHED_PERSONA_COLUMNS} FROM personas \
             WHERE lower(display_name) LIKE $1 OR lower(coalesce(email_address, '')) LIKE $1 \
             ORDER BY interaction_count DESC LIMIT $2"
        );
        let rows = sqlx::query(&sql)
            .bind(&pattern)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_enriched).collect()
    }
}
