use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::Persona;
use crate::domains::personas::api::rows::row_to_persona;

impl PersonaProjectionStore {
    pub async fn list_personas(&self, limit: i64) -> Result<Vec<Persona>, PersonaProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                is_address_book,
                created_at,
                updated_at
            FROM personas
            ORDER BY updated_at DESC, created_at DESC, person_id
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn get_persona(
        &self,
        persona_id: &str,
    ) -> Result<Option<Persona>, PersonaProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                is_address_book,
                created_at,
                updated_at
            FROM personas
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_persona).transpose()
    }
}
