//! Explicit hooks for live workflow-persistence conformance on disposable PostgreSQL.

use sqlx::PgPool;

use crate::ReviewedNoteCandidatePromotionPersistenceV1;

pub struct ReviewedNoteCandidatePromotionPersistenceConformanceV1;

impl ReviewedNoteCandidatePromotionPersistenceConformanceV1 {
    #[must_use]
    pub fn from_disposable_pool(pool: PgPool) -> ReviewedNoteCandidatePromotionPersistenceV1 {
        ReviewedNoteCandidatePromotionPersistenceV1 { pool }
    }
}
