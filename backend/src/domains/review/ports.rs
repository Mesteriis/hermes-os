use serde_json::Value;
use sqlx::{Postgres, Transaction};

use super::errors::ReviewInboxError;
use super::models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItem, ReviewItemKind, ReviewItemStatus,
    ReviewPromotionTarget,
};
use super::store::ReviewInboxStore;

#[derive(Clone)]
pub struct ReviewInboxPort(ReviewInboxStore);

impl ReviewInboxPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(ReviewInboxStore::new(pool))
    }

    pub async fn get(&self, id: &str) -> Result<ReviewItem, ReviewInboxError> {
        self.0.get(id).await
    }

    pub async fn create_with_evidence(
        &self,
        item: &NewReviewItem,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        self.0.create_with_evidence(item, evidence).await
    }

    pub async fn list_evidence(
        &self,
        id: &str,
    ) -> Result<Vec<super::store::ReviewItemEvidenceRecord>, ReviewInboxError> {
        self.0.list_evidence(id).await
    }

    pub async fn promote_with_observation(
        &self,
        id: &str,
        target: ReviewPromotionTarget,
        observation_id: Option<&str>,
        metadata: Option<Value>,
        causation_id: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        self.0
            .promote_with_observation(
                id,
                target,
                observation_id,
                metadata,
                causation_id,
                correlation_id,
            )
            .await
    }

    pub(crate) async fn create_with_evidence_in_transaction(
        tx: &mut Transaction<'_, Postgres>,
        item: &NewReviewItem,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        ReviewInboxStore::create_with_evidence_in_transaction(tx, item, evidence).await
    }

    pub(crate) async fn attach_evidence_in_transaction(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        ReviewInboxStore::attach_evidence_in_transaction(tx, id, evidence).await
    }

    pub(crate) async fn transition_status_in_transaction(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        ReviewInboxStore::transition_status_in_transaction(tx, id, status).await
    }

    pub(crate) async fn promote_in_transaction(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewInboxError> {
        ReviewInboxStore::promote_in_transaction(tx, id, target).await
    }

    pub(crate) async fn find_latest_by_kind_and_metadata_in_transaction(
        tx: &mut Transaction<'_, Postgres>,
        kind: ReviewItemKind,
        metadata: &Value,
    ) -> Result<Option<ReviewItem>, ReviewInboxError> {
        ReviewInboxStore::find_latest_by_kind_and_metadata_in_transaction(tx, kind, metadata).await
    }
}
