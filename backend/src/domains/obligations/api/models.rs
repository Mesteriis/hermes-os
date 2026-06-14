use serde::{Deserialize, Serialize};

use super::super::Obligation;

#[derive(Debug, Deserialize)]
pub(crate) struct ObligationListQuery {
    pub(crate) entity_kind: Option<String>,
    pub(crate) entity_id: Option<String>,
    pub(crate) review_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ObligationReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObligationListResponse {
    pub(crate) items: Vec<Obligation>,
}
