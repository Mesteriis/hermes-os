use serde_json::{Value, json};

use super::super::errors::DecisionStoreError;
use super::super::validation::{validate_json_object, validate_non_empty};
use super::entity_kind::DecisionEntityKind;

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecisionImpactedEntity {
    pub entity_kind: DecisionEntityKind,
    pub entity_id: String,
    pub impact_type: String,
    pub metadata: Value,
}

impl NewDecisionImpactedEntity {
    pub fn new(entity_kind: DecisionEntityKind, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind,
            entity_id: entity_id.into(),
            impact_type: "related".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn impact_type(mut self, impact_type: impl Into<String>) -> Self {
        self.impact_type = impact_type.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(in crate::domains::decisions) fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("entity_id", &self.entity_id)?;
        validate_non_empty("impact_type", &self.impact_type)?;
        validate_json_object("impact metadata", &self.metadata)
    }
}
