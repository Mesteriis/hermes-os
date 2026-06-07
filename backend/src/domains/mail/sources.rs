use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FixtureEmailMessage {
    pub provider_record_id: String,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub body_text: String,
    pub source_fingerprint: String,
}

#[derive(Debug, Deserialize)]
struct RawFixtureEmailMessage {
    provider_record_id: String,
    subject: String,
    from: String,
    to: Vec<String>,
    sent_at: Option<DateTime<Utc>>,
    body_text: String,
    source_fingerprint: String,
}

pub fn parse_fixture_email_messages(
    input: &str,
) -> Result<Vec<FixtureEmailMessage>, FixtureEmailSourceError> {
    let raw_messages: Vec<RawFixtureEmailMessage> = serde_json::from_str(input)?;
    raw_messages
        .into_iter()
        .map(validate_fixture_message)
        .collect()
}

fn validate_fixture_message(
    message: RawFixtureEmailMessage,
) -> Result<FixtureEmailMessage, FixtureEmailSourceError> {
    validate_non_empty("provider_record_id", &message.provider_record_id)?;
    validate_non_empty("subject", &message.subject)?;
    validate_non_empty("from", &message.from)?;
    validate_non_empty("body_text", &message.body_text)?;
    validate_non_empty("source_fingerprint", &message.source_fingerprint)?;
    if message.to.is_empty() {
        return Err(FixtureEmailSourceError::EmptyRecipients);
    }
    for recipient in &message.to {
        validate_non_empty("to", recipient)?;
    }

    Ok(FixtureEmailMessage {
        provider_record_id: message.provider_record_id,
        subject: message.subject,
        from: message.from,
        to: message.to,
        sent_at: message.sent_at,
        body_text: message.body_text,
        source_fingerprint: message.source_fingerprint,
    })
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), FixtureEmailSourceError> {
    if value.trim().is_empty() {
        return Err(FixtureEmailSourceError::EmptyField(field_name));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum FixtureEmailSourceError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("to must contain at least one recipient")]
    EmptyRecipients,
}
