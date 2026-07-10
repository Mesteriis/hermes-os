use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::Persona;
use crate::domains::personas::api::rows::row_to_persona;
use crate::domains::personas::api::validation::{
    normalize_email_address, persona_id_for_email, validate_display_name,
};
use crate::domains::personas::core::link_persona_entity_in_transaction;

impl PersonaProjectionStore {
    pub async fn upsert_email_persona(
        &self,
        email_address: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        self.upsert_email_persona_internal(email_address, None)
            .await
    }

    pub async fn upsert_email_persona_with_observation(
        &self,
        email_address: &str,
        observation_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        self.upsert_email_persona_internal(email_address, Some(observation_id))
            .await
    }

    pub async fn upsert_address_book_persona(
        &self,
        display_name: Option<&str>,
        email_address: Option<&str>,
        fallback_person_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = if let Some(email_address) = email_address {
            let (person, _) = Self::upsert_address_book_email_persona_in_transaction(
                &mut transaction,
                display_name,
                email_address,
                "address_book_sync",
            )
            .await?;
            person
        } else {
            Self::upsert_address_book_persona_without_email_in_transaction(
                &mut transaction,
                display_name,
                fallback_person_id,
            )
            .await?
        };
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn upsert_address_book_email_for_existing_persona(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        email_address: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = Self::upsert_address_book_email_for_existing_persona_in_transaction(
            &mut transaction,
            persona_id,
            display_name,
            email_address,
        )
        .await?;
        transaction.commit().await?;
        Ok(person)
    }

    async fn upsert_email_persona_internal(
        &self,
        email_address: &str,
        observation_id: Option<&str>,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let (person, identity_id) =
            Self::upsert_email_persona_in_transaction(&mut transaction, email_address).await?;
        if let Some(observation_id) = observation_id {
            Self::link_email_persona_projection_in_transaction(
                &mut transaction,
                observation_id,
                &person,
                &identity_id,
                email_address,
                "email_sync_projection",
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_email_persona_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
    ) -> Result<(Persona, String), PersonaProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let persona_id = persona_id_for_email(&normalized_email);

        let row = sqlx::query(
            r#"
            INSERT INTO personas (
                persona_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (email_address)
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
        .bind(&persona_id)
        .bind(&normalized_email)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;

        let person = row_to_persona(row)?;
        let identity_row = sqlx::query(
            r#"
            INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
            VALUES ($1, 'email', $2, 'email_sync', 1.0, 'active')
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                persona_id = EXCLUDED.persona_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                last_verified_at = now(),
                updated_at = now()
            RETURNING id::text
            "#,
        )
        .bind(&person.persona_id)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;
        let identity_id = identity_row.try_get("id")?;

        Ok((person, identity_id))
    }

    pub(crate) async fn upsert_address_book_email_persona_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        display_name: Option<&str>,
        email_address: &str,
        identity_source: &str,
    ) -> Result<(Persona, String), PersonaProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let fallback_display_name = normalized_email.as_str();
        let display_name = display_name
            .map(validate_display_name)
            .transpose()?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| fallback_display_name.to_owned());
        let persona_id = persona_id_for_email(&normalized_email);

        let row = sqlx::query(
            r#"
            INSERT INTO personas (
                persona_id,
                display_name,
                email_address,
                is_address_book
            )
            VALUES ($1, $2, $3, true)
            ON CONFLICT (email_address)
            DO UPDATE SET
                display_name = CASE
                    WHEN personas.display_name = personas.email_address
                      OR trim(personas.display_name) = ''
                    THEN EXCLUDED.display_name
                    ELSE personas.display_name
                END,
                is_address_book = true,
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
        .bind(&persona_id)
        .bind(&display_name)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;

        let person = row_to_persona(row)?;
        let identity_row = sqlx::query(
            r#"
            INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
            VALUES ($1, 'email', $2, $3, 1.0, 'active')
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                persona_id = EXCLUDED.persona_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                last_verified_at = now(),
                updated_at = now()
            RETURNING id::text
            "#,
        )
        .bind(&person.persona_id)
        .bind(&normalized_email)
        .bind(identity_source)
        .fetch_one(&mut **transaction)
        .await?;
        let identity_id = identity_row.try_get("id")?;

        Ok((person, identity_id))
    }

    pub(crate) async fn upsert_address_book_email_for_existing_persona_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona_id: &str,
        display_name: Option<&str>,
        email_address: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let display_name = display_name
            .map(validate_display_name)
            .transpose()?
            .filter(|value| !value.trim().is_empty());

        let row = sqlx::query(
            r#"
            UPDATE personas
            SET
                display_name = CASE
                    WHEN $2::text IS NOT NULL
                     AND (
                        trim(personas.display_name) = ''
                        OR personas.display_name = personas.email_address
                        OR personas.email_address IS NULL
                     )
                    THEN $2
                    ELSE personas.display_name
                END,
                email_address = CASE
                    WHEN personas.email_address IS NULL
                     AND NOT EXISTS (
                        SELECT 1
                        FROM personas existing
                        WHERE existing.email_address = $3
                          AND existing.persona_id <> personas.persona_id
                     )
                    THEN $3
                    ELSE personas.email_address
                END,
                is_address_book = true,
                updated_at = now()
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
        .bind(display_name.as_deref())
        .bind(&normalized_email)
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or_else(|| PersonaProjectionError::PersonaNotFound(persona_id.to_owned()))?;

        let person = row_to_persona(row)?;
        sqlx::query(
            r#"
            INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
            VALUES ($1, 'email', $2, 'address_book_sync', 1.0, 'active')
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                source = persona_identities.source,
                confidence = GREATEST(persona_identities.confidence, EXCLUDED.confidence),
                last_verified_at = now(),
                updated_at = now()
            "#,
        )
        .bind(&person.persona_id)
        .bind(&normalized_email)
        .execute(&mut **transaction)
        .await?;

        Ok(person)
    }

    pub(crate) async fn upsert_address_book_persona_without_email_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        display_name: Option<&str>,
        fallback_person_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        Self::upsert_persona_without_email_in_transaction(
            transaction,
            display_name,
            fallback_person_id,
            true,
        )
        .await
    }

    pub(crate) async fn upsert_persona_without_email_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        display_name: Option<&str>,
        fallback_person_id: &str,
        is_address_book: bool,
    ) -> Result<Persona, PersonaProjectionError> {
        let fallback_person_id = fallback_person_id.trim();
        if fallback_person_id.is_empty() {
            return Err(PersonaProjectionError::PersonaNotFound(
                "fallback persona_id must not be empty".to_owned(),
            ));
        }
        let display_name = display_name
            .map(validate_display_name)
            .transpose()?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| fallback_person_id.to_owned());

        let row = sqlx::query(
            r#"
            INSERT INTO personas (
                persona_id,
                display_name,
                email_address,
                is_address_book
            )
            VALUES ($1, $2, NULL, $3)
            ON CONFLICT (persona_id)
            DO UPDATE SET
                display_name = CASE
                    WHEN trim(personas.display_name) = ''
                    THEN EXCLUDED.display_name
                    ELSE personas.display_name
                END,
                is_address_book = personas.is_address_book OR EXCLUDED.is_address_book,
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
        .bind(fallback_person_id)
        .bind(&display_name)
        .bind(is_address_book)
        .fetch_one(&mut **transaction)
        .await?;

        row_to_persona(row)
    }

    pub(crate) async fn link_email_persona_projection_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation_id: &str,
        person: &Persona,
        identity_id: &str,
        identity_value: &str,
        relationship_kind: &str,
    ) -> Result<(), PersonaProjectionError> {
        link_persona_entity_in_transaction(
            transaction,
            observation_id,
            "persona",
            person.persona_id.clone(),
            Some(relationship_kind),
            Some(serde_json::json!({
                "projection": "persona",
                "identity_type": "email",
                "identity_value": identity_value,
            })),
        )
        .await?;
        link_persona_entity_in_transaction(
            transaction,
            observation_id,
            "identity",
            identity_id.to_owned(),
            Some(relationship_kind),
            Some(serde_json::json!({
                "projection": "identity",
                "persona_id": person.persona_id,
                "identity_type": "email",
                "identity_value": identity_value,
            })),
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn link_persona_projection_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation_id: &str,
        person: &Persona,
        relationship_kind: &str,
    ) -> Result<(), PersonaProjectionError> {
        link_persona_entity_in_transaction(
            transaction,
            observation_id,
            "persona",
            person.persona_id.clone(),
            Some(relationship_kind),
            Some(serde_json::json!({
                "projection": "persona",
                "identity_type": null,
            })),
        )
        .await?;
        Ok(())
    }
}
