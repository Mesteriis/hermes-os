use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::core::link_persons_entity_in_transaction;

impl PersonProjectionStore {
    pub async fn owner_persona(&self) -> Result<Option<Person>, PersonProjectionError> {
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
            WHERE is_self = true
            "#,
        )
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_person).transpose()
    }

    pub async fn set_owner_persona(
        &self,
        person_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = assign_owner_persona_in_transaction(&mut transaction, person_id).await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn set_owner_persona_with_observation(
        &self,
        person_id: &str,
        observation_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = assign_owner_persona_in_transaction(&mut transaction, person_id).await?;
        link_persons_entity_in_transaction(
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
) -> Result<Person, PersonProjectionError> {
    sqlx::query(
        r#"
        UPDATE persons
        SET is_self = false, updated_at = now()
        WHERE is_self = true AND person_id <> $1
        "#,
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;

    let row = sqlx::query(
        r#"
        UPDATE persons
        SET is_self = true, updated_at = now()
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
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| PersonProjectionError::PersonNotFound(person_id.to_owned()))?;

    row_to_person(row)
}
