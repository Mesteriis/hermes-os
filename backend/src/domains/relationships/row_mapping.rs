use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::RelationshipStoreError;
use super::models::{Relationship, RelationshipEntityKind, RelationshipReviewState};

pub(super) fn row_to_relationship(row: PgRow) -> Result<Relationship, RelationshipStoreError> {
    Ok(Relationship {
        relationship_id: row.try_get("relationship_id")?,
        source_entity_kind: parse_entity_kind(row.try_get("source_entity_kind")?)?,
        source_entity_id: row.try_get("source_entity_id")?,
        target_entity_kind: parse_entity_kind(row.try_get("target_entity_kind")?)?,
        target_entity_id: row.try_get("target_entity_id")?,
        relationship_type: row.try_get("relationship_type")?,
        trust_score: row.try_get("trust_score")?,
        strength_score: row.try_get("strength_score")?,
        confidence: row.try_get("confidence")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<RelationshipEntityKind, RelationshipStoreError> {
    RelationshipEntityKind::parse(value)
}

fn parse_review_state(value: String) -> Result<RelationshipReviewState, RelationshipStoreError> {
    RelationshipReviewState::parse(value)
}
