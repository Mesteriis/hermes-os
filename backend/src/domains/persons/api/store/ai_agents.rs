use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::{Person, PersonaType};
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::api::validation::{
    ai_agent_email_address, ai_agent_person_id, normalize_ai_agent_id, validate_display_name,
};

impl PersonProjectionStore {
    pub async fn upsert_ai_agent_persona(
        &self,
        agent_id: &str,
        display_name: &str,
    ) -> Result<Person, PersonProjectionError> {
        let normalized_agent_id = normalize_ai_agent_id(agent_id)?;
        validate_display_name(display_name)?;
        let person_id = ai_agent_person_id(&normalized_agent_id);
        let email_address = ai_agent_email_address(&normalized_agent_id);
        let mut transaction = self.pool().begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address,
                person_type,
                is_self
            )
            VALUES ($1, $2, $3, 'ai_agent', false)
            ON CONFLICT (person_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                email_address = EXCLUDED.email_address,
                person_type = 'ai_agent',
                is_self = false,
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
        .bind(&email_address)
        .bind(&email_address)
        .fetch_one(&mut *transaction)
        .await?;

        let person = row_to_person(row)?;
        sqlx::query(
            r#"
            INSERT INTO person_identities (
                person_id,
                identity_type,
                identity_value,
                source,
                confidence,
                status,
                metadata
            )
            VALUES (
                $1,
                'email',
                $2,
                'ai_agent_registry',
                1.0,
                'active',
                jsonb_build_object('agent_id', $3, 'persona_type', 'ai_agent')
            )
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                person_id = EXCLUDED.person_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                last_verified_at = now(),
                updated_at = now()
            "#,
        )
        .bind(&person.person_id)
        .bind(&email_address)
        .bind(&normalized_agent_id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(person)
    }
}
