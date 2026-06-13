use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::errors::MessageProjectionError;
use super::states::{LocalMessageState, WorkflowState};
use super::validation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProjectedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub channel_kind: String,
    pub conversation_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
}

impl NewProjectedMessage {
    pub(crate) fn validate(&self) -> Result<(), MessageProjectionError> {
        self.validate_with_body_policy(false)
    }

    pub(crate) fn validate_with_body_policy(
        &self,
        allow_empty_body_text: bool,
    ) -> Result<(), MessageProjectionError> {
        validate_non_empty("message_id", &self.message_id)?;
        validate_non_empty("raw_record_id", &self.raw_record_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_record_id", &self.provider_record_id)?;
        validate_non_empty("subject", &self.subject)?;
        validate_non_empty("sender", &self.sender)?;
        if !allow_empty_body_text {
            validate_non_empty("body_text", &self.body_text)?;
        }
        validate_non_empty("channel_kind", &self.channel_kind)?;
        validate_non_empty("delivery_state", &self.delivery_state)?;
        if !self.message_metadata.is_object() {
            return Err(MessageProjectionError::InvalidMessageMetadata);
        }
        for recipient in &self.recipients {
            validate_non_empty("to", recipient)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub conversation_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
    pub workflow_state: WorkflowState,
    pub importance_score: Option<i16>,
    pub ai_category: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_summary_generated_at: Option<DateTime<Utc>>,
    pub local_state: LocalMessageState,
    pub local_state_changed_at: Option<DateTime<Utc>>,
    pub local_state_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessageSummary {
    pub message: ProjectedMessage,
    pub attachment_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkflowStateCount {
    pub state: WorkflowState,
    pub count: i64,
}
