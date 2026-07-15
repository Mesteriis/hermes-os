#[derive(Clone)]
pub struct PersonaProjectionPort(PersonaProjectionStore);

impl PersonaProjectionPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(PersonaProjectionStore::new(pool))
    }

    pub async fn upsert_review_person(
        &self,
        persona_id: &str,
        display_name: &str,
    ) -> Result<Persona, PersonaProjectionError> {
        self.0.upsert_review_person(persona_id, display_name).await
    }

    pub(crate) async fn upsert_email_persona_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
    ) -> Result<(Persona, String), PersonaProjectionError> {
        PersonaProjectionStore::upsert_email_persona_in_transaction(transaction, email_address)
            .await
    }

    pub(crate) async fn link_email_persona_projection_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation_id: &str,
        person: &Persona,
        identity_id: &str,
        identity_value: &str,
        relationship_kind: &str,
    ) -> Result<(), PersonaProjectionError> {
        PersonaProjectionStore::link_email_persona_projection_in_transaction(
            transaction,
            observation_id,
            person,
            identity_id,
            identity_value,
            relationship_kind,
        )
        .await
    }

    pub(crate) async fn upsert_persona_without_email_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        display_name: Option<&str>,
        fallback_persona_id: &str,
        is_address_book: bool,
    ) -> Result<Persona, PersonaProjectionError> {
        PersonaProjectionStore::upsert_persona_without_email_in_transaction(
            transaction,
            display_name,
            fallback_persona_id,
            is_address_book,
        )
        .await
    }

    pub(crate) async fn link_persona_projection_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation_id: &str,
        person: &Persona,
        relationship_kind: &str,
    ) -> Result<(), PersonaProjectionError> {
        PersonaProjectionStore::link_persona_projection_in_transaction(
            transaction,
            observation_id,
            person,
            relationship_kind,
        )
        .await
    }
}

impl std::ops::Deref for PersonaProjectionPort {
    type Target = PersonaProjectionStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use sqlx::{Postgres, Transaction};

use super::api::errors::PersonaProjectionError;
use super::api::models::Persona;
use super::api::store::PersonaProjectionStore;
use super::memory::errors::PersonaMemoryError;
use super::memory::relationship_events::RelationshipEventStore;

#[derive(Clone)]
pub struct RelationshipEventPort(RelationshipEventStore);

impl RelationshipEventPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(RelationshipEventStore::new(pool))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_email_message_event(
        &self,
        observation_id: &str,
        message_id: &str,
        occurred_at: chrono::DateTime<chrono::Utc>,
        persona_id: &str,
        event_type: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<bool, PersonaMemoryError> {
        self.0
            .upsert_email_message_event(
                observation_id,
                message_id,
                occurred_at,
                persona_id,
                event_type,
                title,
                description,
            )
            .await
    }
}
