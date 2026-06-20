use super::PersonProjectionStore;
use super::owner::assign_owner_persona_in_transaction;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::api::validation::validate_display_name;
use crate::domains::persons::core::link_persons_entity_in_transaction;

impl PersonProjectionStore {
    pub async fn update_persona(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE persons
                SET display_name = $2, updated_at = now()
                WHERE person_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonProjectionError::PersonNotFound(persona_id.to_owned()));
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
            .bind("person")
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            assign_owner_persona_in_transaction(&mut transaction, persona_id).await?;
        }

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
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(persona_id.to_owned()))?;

        let person = row_to_person(row)?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn update_persona_with_observation(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
        observation_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE persons
                SET display_name = $2, updated_at = now()
                WHERE person_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonProjectionError::PersonNotFound(persona_id.to_owned()));
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
            .bind("person")
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            assign_owner_persona_in_transaction(&mut transaction, persona_id).await?;
        }

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
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(persona_id.to_owned()))?;

        let person = row_to_person(row)?;
        link_persons_entity_in_transaction(
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
}
