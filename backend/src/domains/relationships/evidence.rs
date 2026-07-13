use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use hermes_observations_postgres::errors::ObservationStoreError;
use hermes_observations_postgres::review_links::link_domain_entity_in_transaction;

pub(crate) async fn link_relationship_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "relationships",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        confidence,
        metadata,
    )
    .await
}
