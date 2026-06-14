use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;

impl PersonProjectionStore {
    pub async fn list_personas(&self, limit: i64) -> Result<Vec<Person>, PersonProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            ORDER BY updated_at DESC, created_at DESC, person_id
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(row_to_person).collect()
    }

    pub async fn get_persona(
        &self,
        persona_id: &str,
    ) -> Result<Option<Person>, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_person).transpose()
    }
}
