use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::Persona;
use crate::domains::personas::api::rows::row_to_persona;

impl PersonaProjectionStore {
    pub async fn upsert_review_person(
        &self,
        persona_id: &str,
        display_name: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person =
            Self::upsert_review_person_in_transaction(&mut transaction, persona_id, display_name)
                .await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_review_person_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona_id: &str,
        display_name: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let persona_id = persona_id.trim();
        let display_name = display_name.trim();
        if persona_id.is_empty() {
            return Err(PersonaProjectionError::PersonaNotFound(
                "review promoted persona_id must not be empty".to_owned(),
            ));
        }
        if display_name.is_empty() {
            return Err(PersonaProjectionError::EmptyDisplayName);
        }

        let row = sqlx::query(
            r#"
            INSERT INTO personas (
                persona_id,
                display_name,
                person_type,
                is_self
            )
            VALUES ($1, $2, 'human', false)
            ON CONFLICT (persona_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                updated_at = now()
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
        .bind(display_name)
        .fetch_one(&mut **transaction)
        .await?;
        let person = row_to_persona(row)?;

        sqlx::query(
            r#"
            INSERT INTO persona_interaction_contexts (
                interaction_context_id,
                source_persona_id,
                name
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (interaction_context_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = now()
            "#,
        )
        .bind(persona_id)
        .bind(persona_id)
        .bind(display_name)
        .execute(&mut **transaction)
        .await?;

        Ok(person)
    }
}
