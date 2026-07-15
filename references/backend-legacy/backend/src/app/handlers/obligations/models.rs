use serde::{Deserialize, Serialize};

use hermes_obligations_api::ObligationRead;

#[derive(Debug, Deserialize)]
pub(crate) struct ObligationReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObligationListResponse {
    pub(crate) items: Vec<ObligationRead>,
}
