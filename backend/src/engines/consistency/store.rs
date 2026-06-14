mod observations;
mod refresh;
mod review;
mod sources;

use sqlx::postgres::PgPool;

use super::errors::ConsistencyError;
use super::models::{
    ContradictionObservation, ContradictionReviewState, NewContradictionObservation,
};

pub struct ContradictionObservationStore {
    pool: PgPool,
}

impl ContradictionObservationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_observations(
        &self,
        limit: i64,
    ) -> Result<usize, ConsistencyError> {
        refresh::refresh_deterministic_observations(&self.pool, limit).await
    }

    pub async fn upsert(
        &self,
        observation: &NewContradictionObservation,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        observations::upsert(&self.pool, observation).await
    }

    pub async fn list_open(
        &self,
        limit: i64,
    ) -> Result<Vec<ContradictionObservation>, ConsistencyError> {
        observations::list_open(&self.pool, limit).await
    }

    pub async fn set_review_state(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        reviewed_by: &str,
        resolution: Option<&str>,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        review::set_review_state(
            &self.pool,
            observation_id,
            review_state,
            reviewed_by,
            resolution,
        )
        .await
    }
}
