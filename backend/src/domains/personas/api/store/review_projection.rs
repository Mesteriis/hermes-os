use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::Persona;
use crate::domains::personas::api::rows::row_to_persona;

impl PersonaProjectionStore {
    pub async fn upsert_review_person(
        &self,
        person_id: &str,
        display_name: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person =
            Self::upsert_review_person_in_transaction(&mut transaction, person_id, display_name)
                .await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_review_person_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        person_id: &str,
        display_name: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let person_id = person_id.trim();
        let display_name = display_name.trim();
        if person_id.is_empty() {
            return Err(PersonaProjectionError::PersonaNotFound(
                "review promoted person_id must not be empty".to_owned(),
            ));
        }
        if display_name.is_empty() {
            return Err(PersonaProjectionError::EmptyDisplayName);
        }

        let row = sqlx::query(
            r#"
            INSERT INTO personas (
                person_id,
                display_name,
                person_type,
                is_self
            )
            VALUES ($1, $2, 'human', false)
            ON CONFLICT (person_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                updated_at = now()
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                is_address_book,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(display_name)
        .fetch_one(&mut **transaction)
        .await?;
        let person = row_to_persona(row)?;

        sqlx::query(
            r#"
            INSERT INTO persona_interaction_contexts (
                persona_id,
                person_id,
                name
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (persona_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = now()
            "#,
        )
        .bind(person_id)
        .bind(person_id)
        .bind(display_name)
        .execute(&mut **transaction)
        .await?;

        Ok(person)
    }
}
