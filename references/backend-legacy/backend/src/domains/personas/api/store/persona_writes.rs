use super::PersonaProjectionStore;
use super::owner::assign_owner_persona_in_transaction;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::Persona;
use crate::domains::personas::api::rows::row_to_persona;
use crate::domains::personas::api::validation::validate_display_name;
use crate::domains::personas::core::evidence::link_persona_entity_in_transaction;

impl PersonaProjectionStore {
    pub async fn update_persona(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE personas
                SET display_name = $2, updated_at = now()
                WHERE persona_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonaProjectionError::PersonaNotFound(
                    persona_id.to_owned(),
                ));
            }

            sqlx::query(
                r#"
                UPDATE graph_nodes
                SET label = $2, updated_at = now()
                WHERE node_kind = $3 AND stable_key = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .bind("persona")
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            assign_owner_persona_in_transaction(&mut transaction, persona_id).await?;
        }

        let row = sqlx::query(
            r#"
            SELECT
                persona_id,
                display_name,
                email_address,
                person_type,
                is_self,
                is_address_book,
                created_at,
                updated_at
            FROM personas
            WHERE persona_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonaProjectionError::PersonaNotFound(persona_id.to_owned()))?;

        let person = row_to_persona(row)?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn update_persona_with_observation(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
        observation_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE personas
                SET display_name = $2, updated_at = now()
                WHERE persona_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonaProjectionError::PersonaNotFound(
                    persona_id.to_owned(),
                ));
            }

            sqlx::query(
                r#"
                UPDATE graph_nodes
                SET label = $2, updated_at = now()
                WHERE node_kind = $3 AND stable_key = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .bind("persona")
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            assign_owner_persona_in_transaction(&mut transaction, persona_id).await?;
        }

        let row = sqlx::query(
            r#"
            SELECT
                persona_id,
                display_name,
                email_address,
                person_type,
                is_self,
                is_address_book,
                created_at,
                updated_at
            FROM personas
            WHERE persona_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonaProjectionError::PersonaNotFound(persona_id.to_owned()))?;

        let person = row_to_persona(row)?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona_id,
            Some("persona_update"),
            None,
        )
        .await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn set_address_book_membership_with_observation(
        &self,
        persona_id: &str,
        is_address_book: bool,
        observation_id: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE personas
            SET is_address_book = $2, updated_at = now()
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
        .bind(is_address_book)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonaProjectionError::PersonaNotFound(persona_id.to_owned()))?;

        let person = row_to_persona(row)?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona_id,
            Some("address_book_membership"),
            Some(serde_json::json!({
                "is_address_book": is_address_book,
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(person)
    }
}
