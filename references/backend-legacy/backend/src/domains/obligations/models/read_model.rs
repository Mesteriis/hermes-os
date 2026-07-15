use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::entity_kind::ObligationEntityKind;
use super::states::{ObligationReviewState, ObligationRiskState, ObligationStatus};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Obligation {
    pub obligation_id: String,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: ObligationStatus,
    pub review_state: ObligationReviewState,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: ObligationRiskState,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
