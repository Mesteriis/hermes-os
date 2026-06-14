use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::api::validation::{normalize_email_address, person_id_for_email};

impl PersonProjectionStore {
    pub async fn upsert_email_person(
        &self,
        email_address: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person =
            Self::upsert_email_person_in_transaction(&mut transaction, email_address).await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_email_person_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
    ) -> Result<Person, PersonProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let person_id = person_id_for_email(&normalized_email);

        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (email_address)
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
        .bind(&person_id)
        .bind(&normalized_email)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;

        let person = row_to_person(row)?;
        sqlx::query(
            r#"
            INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
            VALUES ($1, 'email', $2, 'email_sync', 1.0, 'active')
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                person_id = EXCLUDED.person_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                last_verified_at = now(),
                updated_at = now()
            "#,
        )
        .bind(&person.person_id)
        .bind(&normalized_email)
        .execute(&mut **transaction)
        .await?;

        Ok(person)
    }
}
