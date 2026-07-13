use serde_json::Value;
use sqlx::postgres::PgPool;

use super::errors::MessageProjectionError;
use super::models::ProjectedMessage;
use super::states::WorkflowState;
use super::store::MessageProjectionStore;

/// Application-facing command/query boundary for message projections.
///
/// Workflows depend on this boundary rather than on SQL storage. PostgreSQL
/// ownership remains encapsulated by `MessageProjectionStore` until this port
/// is moved into the Communications API/PG crate pair.
#[derive(Clone)]
pub struct MessageProjectionPort {
    store: MessageProjectionStore,
}

impl MessageProjectionPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            store: MessageProjectionStore::new(pool),
        }
    }

    pub async fn message(
        &self,
        message_id: &str,
    ) -> Result<Option<ProjectedMessage>, MessageProjectionError> {
        self.store.message(message_id).await
    }

    pub async fn set_ai_analysis(
        &self,
        message_id: &str,
        category: Option<&str>,
        summary: Option<&str>,
        importance_score: Option<i16>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store
            .set_ai_analysis(message_id, category, summary, importance_score)
            .await
    }

    pub async fn set_message_metadata(
        &self,
        message_id: &str,
        metadata: &Value,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store.set_message_metadata(message_id, metadata).await
    }

    pub async fn transition_workflow_state(
        &self,
        message_id: &str,
        state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store
            .transition_workflow_state(message_id, state)
            .await
    }

    pub async fn upsert_email_participant(
        &self,
        message: &ProjectedMessage,
        persona_id: &str,
        email_address: &str,
        display_name: Option<&str>,
        role: &str,
    ) -> Result<bool, MessageProjectionError> {
        self.store
            .upsert_email_participant(message, persona_id, email_address, display_name, role)
            .await
    }
}
