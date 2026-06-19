use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;

impl PersonProjectionStore {
    pub async fn upsert_review_person(
        &self,
        person_id: &str,
        display_name: &str,
    ) -> Result<Person, PersonProjectionError> {
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
    ) -> Result<Person, PersonProjectionError> {
        let person_id = person_id.trim();
        let display_name = display_name.trim();
        if person_id.is_empty() {
            return Err(PersonProjectionError::PersonNotFound(
                "review promoted person_id must not be empty".to_owned(),
            ));
        }
        if display_name.is_empty() {
            return Err(PersonProjectionError::EmptyDisplayName);
        }

        let synthetic_email = format!("{person_id}@hermes.invalid");
        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address,
                person_type,
                is_self
            )
            VALUES ($1, $2, $3, 'human', false)
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
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(display_name)
        .bind(&synthetic_email)
        .fetch_one(&mut **transaction)
        .await?;
        let person = row_to_person(row)?;

        sqlx::query(
            r#"
            INSERT INTO person_personas (
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
