use serde::{Deserialize, Serialize};

use crate::domains::relationships::models::Relationship;

#[derive(Debug, Deserialize)]
pub(crate) struct RelationshipListQuery {
    pub(crate) entity_kind: Option<String>,
    pub(crate) entity_id: Option<String>,
    pub(crate) review_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RelationshipReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RelationshipListResponse {
    pub(crate) items: Vec<Relationship>,
}
