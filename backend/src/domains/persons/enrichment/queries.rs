use super::errors::PersonEnrichmentError;
use super::models::EnrichedPerson;
use super::rows::{ENRICHED_PERSON_COLUMNS, row_to_enriched};
use super::store::PersonEnrichmentStore;

impl PersonEnrichmentStore {
    pub async fn get_enriched(
        &self,
        person_id: &str,
    ) -> Result<Option<EnrichedPerson>, PersonEnrichmentError> {
        let sql = format!("SELECT {ENRICHED_PERSON_COLUMNS} FROM persons WHERE person_id = $1");
        let row = sqlx::query(&sql)
            .bind(person_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_enriched).transpose()
    }

    pub async fn list_enriched(
        &self,
        favorites_only: bool,
        limit: i64,
    ) -> Result<Vec<EnrichedPerson>, PersonEnrichmentError> {
        let limit = limit.clamp(1, 100);
        let sql = if favorites_only {
            format!(
                "SELECT {ENRICHED_PERSON_COLUMNS} FROM persons WHERE is_favorite = true \
                 ORDER BY trust_score DESC NULLS LAST, interaction_count DESC LIMIT $1"
            )
        } else {
            format!(
                "SELECT {ENRICHED_PERSON_COLUMNS} FROM persons \
                 ORDER BY interaction_count DESC, trust_score DESC NULLS LAST LIMIT $1"
            )
        };
        let rows = sqlx::query(&sql).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enriched).collect()
    }

    pub async fn search_persons(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<EnrichedPerson>, PersonEnrichmentError> {
        let pattern = format!("%{}%", query.trim().to_lowercase());
        let sql = format!(
            "SELECT {ENRICHED_PERSON_COLUMNS} FROM persons \
             WHERE lower(display_name) LIKE $1 OR lower(email_address) LIKE $1 \
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
