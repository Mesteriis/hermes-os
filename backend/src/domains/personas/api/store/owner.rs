use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::Persona;
use crate::domains::personas::api::rows::row_to_persona;
use crate::domains::personas::core::link_persona_entity_in_transaction;

impl PersonaProjectionStore {
    pub async fn owner_persona(&self) -> Result<Option<Persona>, PersonaProjectionError> {
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
            WHERE is_self = true
            "#,
        )
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_persona).transpose()
    }

    pub async fn set_owner_persona(
        &self,
        person_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = assign_owner_persona_in_transaction(&mut transaction, person_id).await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn set_owner_persona_with_observation(
        &self,
        person_id: &str,
        observation_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = assign_owner_persona_in_transaction(&mut transaction, person_id).await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            person_id,
            Some("owner_assignment"),
            None,
        )
        .await?;
        transaction.commit().await?;
        Ok(person)
    }
}

pub(super) async fn assign_owner_persona_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
) -> Result<Persona, PersonaProjectionError> {
    sqlx::query(
        r#"
        UPDATE personas
        SET is_self = false, updated_at = now()
        WHERE is_self = true AND person_id <> $1
        "#,
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;

    let row = sqlx::query(
        r#"
        UPDATE personas
        SET is_self = true, updated_at = now()
        WHERE person_id = $1
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
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| PersonaProjectionError::PersonaNotFound(person_id.to_owned()))?;

    row_to_persona(row)
}
