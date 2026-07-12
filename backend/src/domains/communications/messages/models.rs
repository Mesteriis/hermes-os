use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::domains::communications::ai_state::CommunicationAiState;

use super::errors::MessageProjectionError;
use super::search::MessageSearchExpression;
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
    pub observation_id: String,
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
    pub ai_state: Option<CommunicationAiState>,
    pub local_state: LocalMessageState,
    pub local_state_changed_at: Option<DateTime<Utc>>,
    pub local_state_reason: Option<String>,
    pub is_read: bool,
    pub read_changed_at: Option<DateTime<Utc>>,
    pub read_origin: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessageSummary {
    pub message: ProjectedMessage,
    pub attachment_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessagePage {
    pub items: Vec<ProjectedMessageSummary>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MessageSearchMatchMode {
    #[default]
    All,
    Any,
}

impl MessageSearchMatchMode {
    pub const fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    pub const fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MessageSearchQuery {
    pub plain_terms: Vec<String>,
    pub subject_contains: Vec<String>,
    pub subject_equals: Vec<String>,
    pub body_contains: Vec<String>,
    pub body_equals: Vec<String>,
    pub sender_contains: Vec<String>,
    pub sender_equals: Vec<String>,
    pub all_contains: Vec<String>,
    pub all_equals: Vec<String>,
    pub match_mode: MessageSearchMatchMode,
    pub expression: Option<MessageSearchExpression>,
}

impl MessageSearchQuery {
    pub fn is_empty(&self) -> bool {
        self.expression.is_none()
            && self.plain_terms.is_empty()
            && self.subject_contains.is_empty()
            && self.subject_equals.is_empty()
            && self.body_contains.is_empty()
            && self.body_equals.is_empty()
            && self.sender_contains.is_empty()
            && self.sender_equals.is_empty()
            && self.all_contains.is_empty()
            && self.all_equals.is_empty()
    }

    pub fn term_count(&self) -> usize {
        self.expression
            .as_ref()
            .map(MessageSearchExpression::term_count)
            .unwrap_or(0)
            + self.plain_terms.len()
            + self.subject_contains.len()
            + self.subject_equals.len()
            + self.body_contains.len()
            + self.body_equals.len()
            + self.sender_contains.len()
            + self.sender_equals.len()
            + self.all_contains.len()
            + self.all_equals.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessagePageQuery<'a> {
    pub account_id: Option<&'a str>,
    pub workflow_state: Option<WorkflowState>,
    pub is_read: Option<bool>,
    pub channel_kind: Option<&'a str>,
    pub conversation_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub match_mode: MessageSearchMatchMode,
    pub search: MessageSearchQuery,
    pub local_state: LocalMessageState,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkflowStateCount {
    pub state: WorkflowState,
    pub count: i64,
}
