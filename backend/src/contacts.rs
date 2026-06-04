use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone)]
pub struct ContactProjectionStore {
    pool: PgPool,
}

impl ContactProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_email_contact(
        &self,
        email_address: &str,
    ) -> Result<Contact, ContactProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let contact_id = contact_id_for_email(&normalized_email);

        let row = sqlx::query(
            r#"
            INSERT INTO contacts (
                contact_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (email_address)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                updated_at = now()
            RETURNING
                contact_id,
                display_name,
                email_address,
                created_at,
                updated_at
            "#,
        )
        .bind(&contact_id)
        .bind(&normalized_email)
        .bind(&normalized_email)
        .fetch_one(&self.pool)
        .await?;

        row_to_contact(row)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contact {
    pub contact_id: String,
    pub display_name: String,
    pub email_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_contacts_from_message_participants(
    store: &ContactProjectionStore,
    email_addresses: &[String],
) -> Result<Vec<Contact>, ContactProjectionError> {
    let normalized_email_addresses = normalize_email_addresses(email_addresses)?;
    let mut contacts = Vec::new();

    for email_address in normalized_email_addresses {
        contacts.push(store.upsert_email_contact(&email_address).await?);
    }

    Ok(contacts)
}

fn normalize_email_addresses(
    email_addresses: &[String],
) -> Result<Vec<String>, ContactProjectionError> {
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

fn row_to_contact(row: PgRow) -> Result<Contact, ContactProjectionError> {
    Ok(Contact {
        contact_id: row.try_get("contact_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn normalize_email_address(email_address: &str) -> Result<String, ContactProjectionError> {
    let normalized_email = email_address.trim().to_ascii_lowercase();
    if normalized_email.is_empty() {
        return Err(ContactProjectionError::EmptyEmailAddress);
    }

    Ok(normalized_email)
}

fn contact_id_for_email(normalized_email: &str) -> String {
    let mut encoded = String::from("contact:v1:email:");
    encoded.push_str(&normalized_email.len().to_string());
    encoded.push(':');
    encoded.push_str(normalized_email);
    encoded
}

#[derive(Debug, Error)]
pub enum ContactProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("email address must not be empty")]
    EmptyEmailAddress,
}
