use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::communications::StoredRawCommunicationRecord;
use crate::email_rfc822::{EmailRfc822ParseError, ParsedEmailMessage, parse_rfc822_message};
use crate::mail_storage::{LocalMailBlobStore, MailStorageError};

#[derive(Clone)]
pub struct MessageProjectionStore {
    pool: PgPool,
}

impl MessageProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn recent_messages(
        &self,
        limit: i64,
    ) -> Result<Vec<ProjectedMessageSummary>, MessageProjectionError> {
        let limit = validate_limit(limit)?;
        let rows = sqlx::query(
            r#"
            SELECT
                m.message_id,
                m.raw_record_id,
                m.account_id,
                m.provider_record_id,
                m.subject,
                m.sender,
                m.recipients,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            GROUP BY
                m.message_id,
                m.raw_record_id,
                m.account_id,
                m.provider_record_id,
                m.subject,
                m.sender,
                m.recipients,
                m.body_text,
                m.occurred_at,
                m.projected_at
            ORDER BY
                COALESCE(m.occurred_at, m.projected_at) DESC,
                m.projected_at DESC,
                m.message_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_projected_message_summary)
            .collect()
    }

    pub async fn message(
        &self,
        message_id: &str,
    ) -> Result<Option<ProjectedMessage>, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;

        let row = sqlx::query(
            r#"
            SELECT
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
            FROM communication_messages
            WHERE message_id = $1
            "#,
        )
        .bind(message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_projected_message).transpose()
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessageSummary {
    pub message: ProjectedMessage,
    pub attachment_count: i64,
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

pub async fn project_raw_email_message_from_blob(
    store: &MessageProjectionStore,
    blob_store: &LocalMailBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let parsed = parse_raw_email_message_from_blob(blob_store, raw).await?;
    project_parsed_raw_email_message(store, raw, &parsed).await
}

pub async fn parse_raw_email_message_from_blob(
    blob_store: &LocalMailBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ParsedEmailMessage, MessageProjectionError> {
    let storage_kind = required_payload_string(&raw.payload, "raw_blob_storage_kind")?;
    if storage_kind != "local_fs" {
        return Err(MessageProjectionError::UnsupportedRawBlobStorageKind(
            storage_kind,
        ));
    }
    let storage_path = required_payload_string(&raw.payload, "raw_blob_storage_path")?;
    let bytes = blob_store.read_blob(&storage_path).await?;
    Ok(parse_rfc822_message(&bytes)?)
}

pub async fn project_parsed_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
    parsed: &ParsedEmailMessage,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject: parsed.subject.clone(),
        sender: parsed.from.clone(),
        recipients: parsed.to.clone(),
        body_text: parsed.body_text.clone(),
        occurred_at: raw.occurred_at,
    };

    store.upsert_message(&message).await
}

fn row_to_projected_message_summary(
    row: PgRow,
) -> Result<ProjectedMessageSummary, MessageProjectionError> {
    let attachment_count = row.try_get("attachment_count")?;
    Ok(ProjectedMessageSummary {
        message: row_to_projected_message(row)?,
        attachment_count,
    })
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

fn validate_limit(limit: i64) -> Result<i64, MessageProjectionError> {
    if !(1..=100).contains(&limit) {
        return Err(MessageProjectionError::InvalidLimit(limit));
    }

    Ok(limit)
}

#[derive(Debug, Error)]
pub enum MessageProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error(transparent)]
    Rfc822(#[from] EmailRfc822ParseError),

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

    #[error("unsupported raw blob storage kind: {0}")]
    UnsupportedRawBlobStorageKind(String),

    #[error("message query limit must be between 1 and 100: {0}")]
    InvalidLimit(i64),
}
