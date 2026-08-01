//! Explicit hooks for live workflow-persistence conformance on disposable PostgreSQL.

use sqlx::PgPool;

use crate::ReviewedTaskCandidatePromotionPersistenceV1;

pub struct ReviewedTaskCandidatePromotionPersistenceConformanceV1;

impl ReviewedTaskCandidatePromotionPersistenceConformanceV1 {
    #[must_use]
    pub fn from_disposable_pool(pool: PgPool) -> ReviewedTaskCandidatePromotionPersistenceV1 {
        ReviewedTaskCandidatePromotionPersistenceV1 { pool }
    }
}
