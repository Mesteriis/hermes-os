use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ProviderCommandValidationError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("max_retries must be greater than zero")]
    NonPositiveMaxRetries,

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewCommunicationProviderCommand {
    pub command_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_conversation_id: Option<String>,
    pub provider_message_id: Option<String>,
    pub target_ref: Value,
    pub payload: Value,
    pub capability_state: String,
    pub action_class: String,
    pub confirmation_decision: String,
    pub actor_id: String,
    pub max_retries: i32,
}

impl NewCommunicationProviderCommand {
    pub fn new(
        command_id: impl Into<String>,
        account_id: impl Into<String>,
        channel_kind: impl Into<String>,
        command_kind: impl Into<String>,
        idempotency_key: impl Into<String>,
        actor_id: impl Into<String>,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            account_id: account_id.into(),
            channel_kind: channel_kind.into(),
            command_kind: command_kind.into(),
            idempotency_key: idempotency_key.into(),
            provider_conversation_id: None,
            provider_message_id: None,
            target_ref: json!({}),
            payload: json!({}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "not_required".to_owned(),
            actor_id: actor_id.into(),
            max_retries: 3,
        }
    }

    pub fn provider_conversation_id(mut self, provider_conversation_id: impl Into<String>) -> Self {
        self.provider_conversation_id = Some(provider_conversation_id.into());
        self
    }

    pub fn provider_message_id(mut self, provider_message_id: impl Into<String>) -> Self {
        self.provider_message_id = Some(provider_message_id.into());
        self
    }

    pub fn target_ref(mut self, target_ref: Value) -> Self {
        self.target_ref = target_ref;
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn validate(&self) -> Result<(), ProviderCommandValidationError> {
        for (field, value) in [
            ("command_id", self.command_id.as_str()),
            ("account_id", self.account_id.as_str()),
            ("channel_kind", self.channel_kind.as_str()),
            ("command_kind", self.command_kind.as_str()),
            ("idempotency_key", self.idempotency_key.as_str()),
            ("capability_state", self.capability_state.as_str()),
            ("action_class", self.action_class.as_str()),
            ("confirmation_decision", self.confirmation_decision.as_str()),
            ("actor_id", self.actor_id.as_str()),
        ] {
            required_non_empty(field, value)?;
        }
        if self.max_retries <= 0 {
            return Err(ProviderCommandValidationError::NonPositiveMaxRetries);
        }
        required_object("target_ref", &self.target_ref)?;
        required_object("payload", &self.payload)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationProviderCommand {
    pub command_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_conversation_id: Option<String>,
    pub provider_message_id: Option<String>,
    pub target_ref: Value,
    pub payload: Value,
    pub capability_state: String,
    pub action_class: String,
    pub confirmation_decision: String,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub result_payload: Value,
    pub audit_metadata: Value,
    pub provider_state: Value,
    pub reconciliation_status: String,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub provider_observed_at: Option<DateTime<Utc>>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub dead_lettered_at: Option<DateTime<Utc>>,
    pub actor_id: String,
    pub happened_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommunicationProviderCommandDiagnostic {
    pub command_id: String,
    pub account_id: String,
    pub command_kind: String,
    pub message_id: Option<String>,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub reconciliation_status: String,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub dead_lettered_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommunicationProviderCommandStatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommunicationProviderCommandDiagnostics {
    pub items: Vec<CommunicationProviderCommandDiagnostic>,
    pub counts: Vec<CommunicationProviderCommandStatusCount>,
}

#[derive(Debug, Error)]
#[error("provider command queue port error: {0}")]
pub struct ProviderCommandQueuePortError(pub String);

impl ProviderCommandQueuePortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub type ProviderCommandQueuePortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderCommandQueuePortError>> + Send + 'a>>;

pub trait ProviderCommandQueuePort: Send + Sync {
    fn claim_due<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        limit: i64,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>>;

    fn mark_completed<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        result_payload: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>>;

    fn mark_failed<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        error: &'a str,
        result_payload: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>>;

    fn mark_terminal_failed<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        error: &'a str,
        result_payload: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>>;

    fn recover_stale_executing<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        execution_lease: chrono::Duration,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>>;

    fn mark_observed_by_provider_message<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        provider_message_id: &'a str,
        command_kinds: &'a [&'a str],
        observed_at: DateTime<Utc>,
        provider_state: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>>;
}

fn required_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), ProviderCommandValidationError> {
    if value.trim().is_empty() {
        return Err(ProviderCommandValidationError::EmptyField(field));
    }
    Ok(())
}

fn required_object(
    field: &'static str,
    value: &Value,
) -> Result<(), ProviderCommandValidationError> {
    if !value.is_object() {
        return Err(ProviderCommandValidationError::NonObjectJson(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{NewCommunicationProviderCommand, ProviderCommandValidationError};

    #[test]
    fn command_requires_stable_identity_and_object_payloads() {
        let invalid =
            NewCommunicationProviderCommand::new(" ", "account", "mail", "send", "key", "actor")
                .payload(json!("not-an-object"));
        assert_eq!(
            invalid.validate(),
            Err(ProviderCommandValidationError::EmptyField("command_id"))
        );
    }
}
