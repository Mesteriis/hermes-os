use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::communications::StoredRawCommunicationRecord;

#[derive(Clone)]
pub struct MessageProjectionStore {
    pool: PgPool,
}

impl MessageProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate()?;
        let canonical_message_id = message_id(&message.account_id, &message.provider_record_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at
            )
            SELECT
                $1,
                raw_record_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
              AND record_kind = 'email_message'
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at
            "#,
        )
        .bind(&canonical_message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }
}

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
}

impl NewProjectedMessage {
    fn validate(&self) -> Result<(), MessageProjectionError> {
        validate_non_empty("raw_record_id", &self.raw_record_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_record_id", &self.provider_record_id)?;
        validate_non_empty("subject", &self.subject)?;
        validate_non_empty("sender", &self.sender)?;
        validate_non_empty("body_text", &self.body_text)?;
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
}

pub async fn project_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let subject = required_payload_string(&raw.payload, "subject")?;
    let sender = required_payload_string(&raw.payload, "from")?;
    let recipients = required_payload_string_array(&raw.payload, "to")?;
    let body_text = required_payload_string(&raw.payload, "body_text")?;
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject,
        sender,
        recipients,
        body_text,
        occurred_at: raw.occurred_at,
    };

    store.upsert_message(&message).await
}

fn row_to_projected_message(row: PgRow) -> Result<ProjectedMessage, MessageProjectionError> {
    Ok(ProjectedMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        body_text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
    })
}

fn required_payload_string(
    payload: &Value,
    field_name: &'static str,
) -> Result<String, MessageProjectionError> {
    payload
        .get(field_name)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(MessageProjectionError::MissingPayloadField(field_name))
}

fn required_payload_string_array(
    payload: &Value,
    field_name: &'static str,
) -> Result<Vec<String>, MessageProjectionError> {
    let values = payload
        .get(field_name)
        .and_then(Value::as_array)
        .ok_or(MessageProjectionError::MissingPayloadField(field_name))?;

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(MessageProjectionError::MissingPayloadField(field_name))
        })
        .collect()
}

fn recipients_from_value(value: Value) -> Result<Vec<String>, MessageProjectionError> {
    let Some(values) = value.as_array() else {
        return Err(MessageProjectionError::InvalidStoredRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(MessageProjectionError::InvalidStoredRecipients)
        })
        .collect()
}

fn message_id(account_id: &str, provider_record_id: &str) -> String {
    let mut encoded = String::from("msg:v1:");
    append_message_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_message_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_message_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), MessageProjectionError> {
    if value.trim().is_empty() {
        return Err(MessageProjectionError::EmptyField(field_name));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum MessageProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("raw email payload missing required field or wrong type: {0}")]
    MissingPayloadField(&'static str),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error(
        "raw communication record does not match projected message tuple: raw_record_id={raw_record_id}, account_id={account_id}, provider_record_id={provider_record_id}"
    )]
    RawRecordTupleMismatch {
        raw_record_id: String,
        account_id: String,
        provider_record_id: String,
    },

    #[error("stored communication message recipients must be a JSON array of strings")]
    InvalidStoredRecipients,
}
