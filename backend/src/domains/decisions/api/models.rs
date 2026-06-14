use serde::{Deserialize, Serialize};

use super::super::Decision;

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionListQuery {
    pub(crate) entity_kind: Option<String>,
    pub(crate) entity_id: Option<String>,
    pub(crate) review_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DecisionListResponse {
    pub(crate) items: Vec<Decision>,
}
