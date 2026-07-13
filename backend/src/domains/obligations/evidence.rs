use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use hermes_observations_postgres::errors::ObservationStoreError;
use hermes_observations_postgres::review_links::{
    link_domain_entity_in_transaction, materialize_review_transition_link_in_transaction,
};

use super::models::ObligationReviewState;

pub(crate) async fn link_obligation_support_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    obligation_id: impl Into<String>,
    confidence: f64,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "obligations",
        "obligation",
        obligation_id.into(),
        Some("supports"),
        Some(confidence),
        Some(metadata),
    )
    .await
}

pub(crate) async fn link_obligation_review_transition_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    obligation_id: &str,
    review_state: ObligationReviewState,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    materialize_review_transition_link_in_transaction(
        transaction,
        observation_id,
        "obligations",
        "obligation",
        obligation_id,
        "review_state",
        review_state.as_str(),
        metadata,
    )
    .await
}
