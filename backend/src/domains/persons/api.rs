use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

#[derive(Clone)]
pub struct PersonProjectionStore {
    pool: PgPool,
}

impl PersonProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_email_person(
        &self,
        email_address: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool.begin().await?;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Person {
    pub person_id: String,
    pub display_name: String,
    pub email_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_persons_from_message_participants(
    store: &PersonProjectionStore,
    email_addresses: &[String],
) -> Result<Vec<Person>, PersonProjectionError> {
    let normalized_email_addresses = normalize_email_addresses(email_addresses)?;
    let mut persons = Vec::new();

    for email_address in normalized_email_addresses {
        persons.push(store.upsert_email_person(&email_address).await?);
    }

    Ok(persons)
}

fn normalize_email_addresses(
    email_addresses: &[String],
) -> Result<Vec<String>, PersonProjectionError> {
    let mut seen = HashSet::new();
    let mut normalized_email_addresses = Vec::new();

    for email_address in email_addresses {
        let normalized_email = normalize_email_address(email_address)?;
        if seen.insert(normalized_email.clone()) {
            normalized_email_addresses.push(normalized_email);
        }
    }

    Ok(normalized_email_addresses)
}

fn row_to_person(row: PgRow) -> Result<Person, PersonProjectionError> {
    Ok(Person {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn normalize_email_address(email_address: &str) -> Result<String, PersonProjectionError> {
    let normalized_email = email_addr_spec(email_address).trim().to_ascii_lowercase();
    if normalized_email.is_empty() {
        return Err(PersonProjectionError::EmptyEmailAddress);
    }
    if !normalized_email.contains('@') {
        return Err(PersonProjectionError::InvalidEmailAddress(normalized_email));
    }

    Ok(normalized_email)
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<') {
        if let Some((addr, _)) = tail.split_once('>') {
            return addr.trim();
        }
    }
    value.trim_matches('"')
}

fn person_id_for_email(normalized_email: &str) -> String {
    let mut encoded = String::from("person:v1:email:");
    encoded.push_str(&normalized_email.len().to_string());
    encoded.push(':');
    encoded.push_str(normalized_email);
    encoded
}

#[derive(Debug, Error)]
pub enum PersonProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("email address must not be empty")]
    EmptyEmailAddress,

    #[error("invalid email address: {0}")]
    InvalidEmailAddress(String),
}
