use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::{Persona, PersonaType};
use crate::domains::personas::api::rows::row_to_persona;

impl PersonaProjectionStore {
    pub async fn set_persona_type(
        &self,
        persona_id: &str,
        persona_type: PersonaType,
    ) -> Result<Persona, PersonaProjectionError> {
        let row = sqlx::query(
            r#"
            UPDATE personas
            SET person_type = $2, updated_at = now()
            WHERE persona_id = $1
            RETURNING
                persona_id,
                display_name,
                email_address,
                person_type,
                is_self,
                is_address_book,
                created_at,
                updated_at
            "#,
        )
        .bind(persona_id)
        .bind(persona_type.as_str())
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| PersonaProjectionError::PersonaNotFound(persona_id.to_owned()))?;

        row_to_persona(row)
    }
}
