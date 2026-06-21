use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::{Person, PersonaType};
use crate::domains::persons::api::rows::row_to_person;

impl PersonProjectionStore {
    pub async fn set_persona_type(
        &self,
        person_id: &str,
        persona_type: PersonaType,
    ) -> Result<Person, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            UPDATE persons
            SET person_type = $2, updated_at = now()
            WHERE person_id = $1
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(persona_type.as_str())
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(person_id.to_owned()))?;

        row_to_person(row)
    }
}
