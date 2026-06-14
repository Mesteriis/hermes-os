use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ObligationStoreError;
use super::validation::{validate_json_object, validate_non_empty, validate_score};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEntityKind {
    Persona,
    Organization,
    Project,
    Communication,
    Document,
    Task,
    Event,
    Decision,
    Obligation,
    Knowledge,
}

impl ObligationEntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Persona => "persona",
            Self::Organization => "organization",
            Self::Project => "project",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Task => "task",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Knowledge => "knowledge",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObligationStoreError> {
        let value = value.as_ref().trim();
        match value {
            "persona" => Ok(Self::Persona),
            "organization" => Ok(Self::Organization),
            "project" => Ok(Self::Project),
            "communication" => Ok(Self::Communication),
            "document" => Ok(Self::Document),
            "task" => Ok(Self::Task),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "obligation" => Ok(Self::Obligation),
            "knowledge" => Ok(Self::Knowledge),
            _ => Err(ObligationStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEvidenceSourceKind {
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Project,
    Organization,
    Persona,
    RawRecord,
}

impl ObligationEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Persona => "persona",
            Self::RawRecord => "raw_record",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Open,
    Fulfilled,
    Waived,
    Disputed,
    Canceled,
}

impl ObligationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Fulfilled => "fulfilled",
            Self::Waived => "waived",
            Self::Disputed => "disputed",
            Self::Canceled => "canceled",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ObligationReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObligationStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ObligationStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationRiskState {
    None,
    Watch,
    AtRisk,
    Breached,
}

impl ObligationRiskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Watch => "watch",
            Self::AtRisk => "at_risk",
            Self::Breached => "breached",
        }
    }
}

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

    pub(super) fn validate(&self) -> Result<(), ObligationStoreError> {
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

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligationEvidence {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligationEvidence {
    pub fn new(source_kind: ObligationEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(super) fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}

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
