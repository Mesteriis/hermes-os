use super::PersonaProjectionStore;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::models::{Persona, PersonaType};
use crate::domains::personas::api::rows::row_to_persona;
use crate::domains::personas::api::validation::{
    ai_agent_email_address, ai_agent_persona_id, normalize_ai_agent_id, validate_display_name,
};
use crate::platform::graph::{GraphNodeKind, node_id};

impl PersonaProjectionStore {
    pub async fn upsert_ai_agent_persona(
        &self,
        agent_id: &str,
        display_name: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        let normalized_agent_id = normalize_ai_agent_id(agent_id)?;
        validate_display_name(display_name)?;
        let person_id = ai_agent_persona_id(&normalized_agent_id);
        let email_address = ai_agent_email_address(&normalized_agent_id);
        let mut transaction = self.pool().begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO personas (
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
                is_address_book,
                created_at,
                updated_at
            "#,
        )
        .bind(&person_id)
        .bind(&email_address)
        .bind(&email_address)
        .fetch_one(&mut *transaction)
        .await?;

        let person = row_to_persona(row)?;
        let graph_node_id = node_id(GraphNodeKind::Persona, &person.person_id);
        sqlx::query(
            r#"
            INSERT INTO graph_nodes (
                node_id,
                node_kind,
                stable_key,
                label,
                properties
            )
            VALUES (
                $1,
                'person',
                $2,
                $3,
                jsonb_build_object(
                    'email_address', $3,
                    'persona_type', 'ai_agent',
                    'agent_id', $4
                )
            )
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = graph_nodes.properties || EXCLUDED.properties,
                updated_at = now()
            "#,
        )
        .bind(&graph_node_id)
        .bind(&person.person_id)
        .bind(&email_address)
        .bind(&normalized_agent_id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO persona_identities (
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
