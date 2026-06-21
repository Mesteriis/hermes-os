use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{Relationship, RelationshipReviewState, RelationshipStore, RelationshipStoreError};

#[derive(Clone)]
pub struct RelationshipCommandService {
    pool: PgPool,
}

impl RelationshipCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipCommandServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "relationship_id": relationship_id,
                        "review_state": review_state.as_str(),
                        "operation": "relationship_review",
                        "actor_id": "hermes-frontend",
                    }),
                    format!("relationship://{relationship_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "relationships_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(RelationshipStore::new(self.pool.clone())
            .set_review_state_with_observation(
                relationship_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "relationships_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum RelationshipCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
}
