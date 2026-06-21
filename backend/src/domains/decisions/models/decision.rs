use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::super::errors::DecisionStoreError;
use super::super::validation::{
    validate_json_array, validate_json_object, validate_non_empty, validate_score,
};
use super::entity_kind::DecisionEntityKind;
use super::states::{DecisionReviewState, DecisionStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecision {
    pub title: String,
    pub status: DecisionStatus,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: DecisionReviewState,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewDecision {
    pub fn new(
        title: impl Into<String>,
        rationale: impl Into<String>,
        confidence: f64,
        review_state: DecisionReviewState,
    ) -> Self {
        Self {
            title: title.into(),
            status: DecisionStatus::Active,
            rationale: rationale.into(),
            alternatives: json!([]),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
            decided_at: None,
            review_state,
            confidence,
            metadata: json!({}),
        }
    }

    pub fn status(mut self, status: DecisionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn alternatives(mut self, alternatives: Value) -> Self {
        self.alternatives = alternatives;
        self
    }

    pub fn decided_by(
        mut self,
        decided_by_entity_kind: DecisionEntityKind,
        decided_by_entity_id: impl Into<String>,
    ) -> Self {
        self.decided_by_entity_kind = Some(decided_by_entity_kind);
        self.decided_by_entity_id = Some(decided_by_entity_id.into());
        self
    }

    pub fn decided_at(mut self, decided_at: DateTime<Utc>) -> Self {
        self.decided_at = Some(decided_at);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(in crate::domains::decisions) fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("title", &self.title)?;
        validate_non_empty("rationale", &self.rationale)?;
        validate_score("confidence", self.confidence)?;
        validate_json_array("alternatives", &self.alternatives)?;
        validate_json_object("decision metadata", &self.metadata)?;

        match (
            self.decided_by_entity_kind,
            self.decided_by_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(decided_by_entity_id)) => {
                validate_non_empty("decided_by_entity_id", decided_by_entity_id)?;
            }
            _ => return Err(DecisionStoreError::PartialDecider),
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Decision {
    pub decision_id: String,
    pub title: String,
    pub status: DecisionStatus,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: DecisionReviewState,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
