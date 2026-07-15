use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::super::errors::ObligationStoreError;
use super::super::validation::{validate_json_object, validate_non_empty, validate_score};
use super::entity_kind::ObligationEntityKind;
use super::states::{ObligationReviewState, ObligationRiskState, ObligationStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligation {
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
}

impl NewObligation {
    pub fn new(
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
        statement: impl Into<String>,
        confidence: f64,
        review_state: ObligationReviewState,
    ) -> Self {
        Self {
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
            statement: statement.into(),
            status: ObligationStatus::Open,
            review_state,
            due_at: None,
            condition: None,
            risk_state: ObligationRiskState::None,
            confidence,
            metadata: json!({}),
        }
    }

    pub fn beneficiary(
        mut self,
        beneficiary_entity_kind: ObligationEntityKind,
        beneficiary_entity_id: impl Into<String>,
    ) -> Self {
        self.beneficiary_entity_kind = Some(beneficiary_entity_kind);
        self.beneficiary_entity_id = Some(beneficiary_entity_id.into());
        self
    }

    pub fn status(mut self, status: ObligationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn due_at(mut self, due_at: DateTime<Utc>) -> Self {
        self.due_at = Some(due_at);
        self
    }

    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    pub fn risk_state(mut self, risk_state: ObligationRiskState) -> Self {
        self.risk_state = risk_state;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(in crate::domains::obligations) fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("obligated_entity_id", &self.obligated_entity_id)?;
        validate_non_empty("statement", &self.statement)?;
        validate_score("confidence", self.confidence)?;
        validate_json_object("obligation metadata", &self.metadata)?;

        match (
            self.beneficiary_entity_kind,
            self.beneficiary_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(beneficiary_entity_id)) => {
                validate_non_empty("beneficiary_entity_id", beneficiary_entity_id)?;
            }
            _ => return Err(ObligationStoreError::PartialBeneficiary),
        }

        if let Some(condition) = &self.condition {
            validate_non_empty("condition", condition)?;
        }

        Ok(())
    }
}
