use chrono::{DateTime, Utc};
use sqlx::Postgres;
use sqlx::postgres::PgPool;

use super::errors::PersonTrustError;
use super::models::PersonPromise;
use super::rows::row_to_promise;

#[derive(Clone)]
pub struct PersonPromiseStore {
    pool: PgPool,
}

impl PersonPromiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPromise>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, description, source_message_id, promised_at,
             due_at, fulfilled_at, status, created_at, updated_at
             FROM person_promises WHERE person_id = $1 ORDER BY promised_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_promise).collect()
    }

    pub async fn create(
        &self,
        person_id: &str,
        description: &str,
        due_at: Option<DateTime<Utc>>,
    ) -> Result<PersonPromise, PersonTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO person_promises (person_id, description, due_at)
             VALUES ($1, $2, $3)
             RETURNING id::text, person_id, description, source_message_id, promised_at,
                       due_at, fulfilled_at, status, created_at, updated_at",
        )
        .bind(person_id)
        .bind(description)
        .bind(due_at)
        .fetch_one(&mut *transaction)
        .await?;
        let promise = row_to_promise(row)?;
        transaction.commit().await?;

        Ok(promise)
    }

    pub async fn fulfill(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_promises
             SET status = 'fulfilled', fulfilled_at = now(), updated_at = now()
             WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_broken(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_promises SET status = 'broken', updated_at = now() WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
