use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::platform::events::{EventStore, NewEventEnvelope};

const EVENT_TYPE_RECORDED: &str = "mail.read_receipt.recorded";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailReadReceiptKind {
    Read,
}

impl MailReadReceiptKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
        }
    }
}

impl TryFrom<&str> for MailReadReceiptKind {
    type Error = MailReadReceiptError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "read" => Ok(Self::Read),
            _ => Err(MailReadReceiptError::Invalid("receipt_kind")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MailReadReceipt {
    pub receipt_id: String,
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub recipient: String,
    pub receipt_kind: MailReadReceiptKind,
    pub read_at: DateTime<Utc>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewMailReadReceipt {
    pub receipt_id: Option<String>,
    pub account_id: String,
    pub provider_message_id: String,
    pub recipient: String,
    pub read_at: DateTime<Utc>,
    pub source_kind: Option<String>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Clone)]
pub struct MailReadReceiptStore {
    pool: PgPool,
}

impl MailReadReceiptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        receipt: NewMailReadReceipt,
    ) -> Result<MailReadReceipt, MailReadReceiptError> {
        let normalized = NormalizedMailReadReceipt::from_new(receipt)?;
        let mut transaction = self.pool.begin().await?;
        let outbox_id = correlate_outbox(
            &mut transaction,
            &normalized.account_id,
            &normalized.provider_message_id,
        )
        .await?;
        let receipt = insert_receipt(&mut transaction, &normalized, outbox_id.as_deref()).await?;
        let event = read_receipt_event(&receipt)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(receipt)
    }
}

#[derive(Debug)]
struct NormalizedMailReadReceipt {
    receipt_id: String,
    account_id: String,
    provider_message_id: String,
    recipient: String,
    read_at: DateTime<Utc>,
    source_kind: String,
    provider_record_id: Option<String>,
    raw_record_id: Option<String>,
    metadata: Value,
}

impl NormalizedMailReadReceipt {
    fn from_new(receipt: NewMailReadReceipt) -> Result<Self, MailReadReceiptError> {
        let account_id = normalize_required("account_id", &receipt.account_id)?;
        let provider_message_id =
            normalize_required("provider_message_id", &receipt.provider_message_id)?;
        let provider_record_id = normalize_optional(receipt.provider_record_id)?;
        let receipt_id = match receipt.receipt_id {
            Some(value) => normalize_required("receipt_id", &value)?,
            None => generate_receipt_id(&account_id, provider_record_id.as_deref()),
        };
        let metadata = receipt.metadata.unwrap_or_else(|| json!({}));
        if !metadata.is_object() {
            return Err(MailReadReceiptError::Invalid("metadata"));
        }

        Ok(Self {
            receipt_id,
            account_id,
            provider_message_id,
            recipient: normalize_required("recipient", &receipt.recipient)?,
            read_at: receipt.read_at,
            source_kind: normalize_optional(receipt.source_kind)?
                .unwrap_or_else(|| "mdn".to_owned()),
            provider_record_id,
            raw_record_id: normalize_optional(receipt.raw_record_id)?,
            metadata,
        })
    }
}

async fn correlate_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
) -> Result<Option<String>, MailReadReceiptError> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"
        SELECT outbox_id
        FROM email_outbox_tracking
        WHERE account_id = $1
          AND provider_message_id = $2
        ORDER BY sent_at DESC NULLS LAST, created_at DESC, outbox_id ASC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .bind(provider_message_id)
    .fetch_optional(&mut **transaction)
    .await?)
}

async fn insert_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    receipt: &NormalizedMailReadReceipt,
    outbox_id: Option<&str>,
) -> Result<MailReadReceipt, MailReadReceiptError> {
    let row = sqlx::query(
        r#"
        INSERT INTO mail_read_receipts (
            receipt_id,
            account_id,
            outbox_id,
            provider_message_id,
            recipient,
            receipt_kind,
            read_at,
            source_kind,
            provider_record_id,
            raw_record_id,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, 'read', $6, $7, $8, $9, $10)
        RETURNING
            receipt_id,
            account_id,
            outbox_id,
            provider_message_id,
            recipient,
            receipt_kind,
            read_at,
            source_kind,
            provider_record_id,
            raw_record_id,
            metadata,
            created_at
        "#,
    )
    .bind(&receipt.receipt_id)
    .bind(&receipt.account_id)
    .bind(outbox_id)
    .bind(&receipt.provider_message_id)
    .bind(&receipt.recipient)
    .bind(receipt.read_at)
    .bind(&receipt.source_kind)
    .bind(receipt.provider_record_id.as_deref())
    .bind(receipt.raw_record_id.as_deref())
    .bind(&receipt.metadata)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_receipt(row)
}

fn row_to_receipt(row: PgRow) -> Result<MailReadReceipt, MailReadReceiptError> {
    let receipt_kind: String = row.try_get("receipt_kind")?;
    Ok(MailReadReceipt {
        receipt_id: row.try_get("receipt_id")?,
        account_id: row.try_get("account_id")?,
        outbox_id: row.try_get("outbox_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        recipient: row.try_get("recipient")?,
        receipt_kind: MailReadReceiptKind::try_from(receipt_kind.as_str())?,
        read_at: row.try_get("read_at")?,
        source_kind: row.try_get("source_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn read_receipt_event(receipt: &MailReadReceipt) -> Result<NewEventEnvelope, MailReadReceiptError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_read_receipt_event:{}:{}",
            receipt.receipt_id,
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        EVENT_TYPE_RECORDED,
        Utc::now(),
        json!({ "kind": "mail_read_receipt_api" }),
        json!({
            "kind": "mail_read_receipt",
            "id": receipt.receipt_id,
            "account_id": receipt.account_id,
            "outbox_id": receipt.outbox_id,
        }),
    )
    .actor(json!({ "actor_id": "hermes-frontend" }))
    .payload(json!({
        "receipt_id": receipt.receipt_id,
        "account_id": receipt.account_id,
        "outbox_id": receipt.outbox_id,
        "provider_message_id": receipt.provider_message_id,
        "receipt_kind": receipt.receipt_kind.as_str(),
        "read_at": receipt.read_at,
        "source_kind": receipt.source_kind,
    }))
    .provenance(json!({
        "source_kind": receipt.source_kind,
        "source_id": receipt.provider_record_id,
    }))
    .correlation_id(
        receipt
            .outbox_id
            .clone()
            .unwrap_or_else(|| receipt.receipt_id.clone()),
    )
    .build()?)
}

fn normalize_required(field: &'static str, value: &str) -> Result<String, MailReadReceiptError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(MailReadReceiptError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(value: Option<String>) -> Result<Option<String>, MailReadReceiptError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn generate_receipt_id(account_id: &str, provider_record_id: Option<&str>) -> String {
    match provider_record_id {
        Some(provider_record_id) => format!("mail_read_receipt:{account_id}:{provider_record_id}"),
        None => format!(
            "mail_read_receipt:{account_id}:{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
    }
}

#[derive(Debug, Error)]
pub enum MailReadReceiptError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),
    #[error("invalid mail read receipt field: {0}")]
    Invalid(&'static str),
}
