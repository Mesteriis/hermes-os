use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, PgRow, Postgres};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
pub use crate::platform::communications::SmtpTransport;
use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

mod delivery;
mod delivery_status;
mod provider_send_store;
mod provider_sender;
mod smtp_sender;

pub use delivery::{
    EmailOutboxDeliveryWorker, OutboxDeliveryError, OutboxDeliveryReport, OutboxEmailSender,
    OutboxRetryPolicy, OutboxSendReceipt,
};
pub use delivery_status::{
    NewOutboxDeliveryStatus, OutboxDeliveryStatus, OutboxDeliveryStatusRecord,
};
pub use provider_send_store::{ProviderSendStore, ProviderSendStoreError};
pub use provider_sender::CommunicationOutboxEmailSender;
pub use smtp_sender::{
    SmtpOutboxEmailSender, outgoing_email_from_outbox_item, smtp_config_for_provider_account,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationOutboxStatus {
    Queued,
    Scheduled,
    Sending,
    Sent,
    Failed,
    Canceled,
}

impl CommunicationOutboxStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Scheduled => "scheduled",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "queued" => Some(Self::Queued),
            "scheduled" => Some(Self::Scheduled),
            "sending" => Some(Self::Sending),
            "sent" => Some(Self::Sent),
            "failed" => Some(Self::Failed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommunicationOutboxItem {
    pub outbox_id: String,
    pub account_id: String,
    pub draft_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: CommunicationOutboxStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_deadline_at: Option<DateTime<Utc>>,
    pub send_attempts: i32,
    pub claimed_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub provider_message_id: Option<String>,
    pub last_error: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailOutboxListPage {
    pub items: Vec<CommunicationOutboxItem>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct NewCommunicationOutboxItem {
    pub outbox_id: String,
    pub account_id: String,
    pub draft_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: CommunicationOutboxStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_deadline_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewCommunicationOutboxItem {
    fn validate(&self) -> Result<(), CommunicationOutboxError> {
        validate_non_empty("outbox_id", &self.outbox_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        if self
            .to_recipients
            .iter()
            .chain(self.cc_recipients.iter())
            .chain(self.bcc_recipients.iter())
            .all(|recipient| recipient.trim().is_empty())
        {
            return Err(CommunicationOutboxError::Invalid(
                "at least one recipient is required",
            ));
        }
        if !self.metadata.is_object() {
            return Err(CommunicationOutboxError::Invalid(
                "metadata must be a JSON object",
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct CommunicationOutboxStore {
    pool: PgPool,
}

impl CommunicationOutboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(
        &self,
        item: &NewCommunicationOutboxItem,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        self.enqueue_with_observation(item, None, "outbox_status_transition", None)
            .await
    }

    pub async fn enqueue_with_observation(
        &self,
        item: &NewCommunicationOutboxItem,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        item.validate()?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, Some(item.account_id.as_str()))
            .await?;
        let sql = outbox_returning_query(
            r#"
            INSERT INTO communication_outbox (
                outbox_id,
                account_id,
                draft_id,
                to_participants,
                cc_participants,
                bcc_participants,
                subject,
                body_text,
                body_html,
                status,
                scheduled_send_at,
                undo_deadline_at,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(item.outbox_id.trim())
            .bind(item.account_id.trim())
            .bind(item.draft_id.as_deref())
            .bind(serde_json::to_value(&item.to_recipients)?)
            .bind(serde_json::to_value(&item.cc_recipients)?)
            .bind(serde_json::to_value(&item.bcc_recipients)?)
            .bind(&item.subject)
            .bind(&item.body_text)
            .bind(item.body_html.as_deref())
            .bind(item.status.as_str())
            .bind(item.scheduled_send_at)
            .bind(item.undo_deadline_at)
            .bind(&item.metadata)
            .fetch_one(&mut *transaction)
            .await?;

        let outbox_item = row_to_outbox_item(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.trim().is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "status": outbox_item.status.as_str(),
                    "draft_id": outbox_item.draft_id,
                    "scheduled_send_at": outbox_item.scheduled_send_at,
                    "undo_deadline_at": outbox_item.undo_deadline_at,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "outbox_item",
                outbox_item.outbox_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(outbox_item)
    }

    pub async fn list(
        &self,
        account_id: Option<&str>,
        status: Option<CommunicationOutboxStatus>,
        limit: i64,
    ) -> Result<Vec<CommunicationOutboxItem>, CommunicationOutboxError> {
        Ok(self.list_page(account_id, status, None, limit).await?.items)
    }

    pub async fn list_page(
        &self,
        account_id: Option<&str>,
        status: Option<CommunicationOutboxStatus>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<EmailOutboxListPage, CommunicationOutboxError> {
        let limit = validate_limit(limit)?;
        let cursor = cursor.map(decode_outbox_list_cursor).transpose()?;
        let status = status.map(CommunicationOutboxStatus::as_str);
        let rows = sqlx::query(
            r#"
            SELECT
                outbox.outbox_id,
                outbox.account_id,
                outbox.draft_id,
                outbox.to_participants AS to_recipients,
                outbox.cc_participants AS cc_recipients,
                outbox.bcc_participants AS bcc_recipients,
                outbox.subject,
                outbox.body_text,
                outbox.body_html,
                outbox.status,
                outbox.scheduled_send_at,
                outbox.undo_deadline_at,
                outbox.send_attempts,
                outbox.claimed_at,
                outbox.sent_at,
                outbox.provider_message_id,
                outbox.last_error,
                CASE
                    WHEN latest_receipt.latest_read_receipt IS NULL THEN outbox.metadata
                    ELSE jsonb_set(
                        outbox.metadata,
                        '{latest_read_receipt}',
                        latest_receipt.latest_read_receipt,
                        true
                    )
                END AS metadata,
                outbox.created_at,
                outbox.updated_at
            FROM communication_outbox outbox
            LEFT JOIN LATERAL (
                SELECT jsonb_build_object(
                    'receipt_kind', receipt.receipt_kind,
                    'read_at', receipt.read_at,
                    'source_kind', receipt.source_kind
                ) AS latest_read_receipt
                FROM communication_read_receipts receipt
                WHERE receipt.outbox_id = outbox.outbox_id
                ORDER BY receipt.read_at DESC, receipt.receipt_id ASC
                LIMIT 1
            ) latest_receipt ON true
            WHERE ($1::text IS NULL OR outbox.account_id = $1)
              AND ($2::text IS NULL OR outbox.status = $2)
              AND (
                  $3::timestamptz IS NULL
                  OR outbox.created_at < $3
                  OR (outbox.created_at = $3 AND outbox.outbox_id > $4)
              )
            ORDER BY outbox.created_at DESC, outbox.outbox_id ASC
            LIMIT $5
            "#,
        )
        .bind(account_id)
        .bind(status)
        .bind(cursor.as_ref().map(|cursor| cursor.created_at))
        .bind(cursor.as_ref().map(|cursor| cursor.outbox_id.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

        let mut items = rows
            .into_iter()
            .map(row_to_outbox_item)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > limit as usize;
        if has_more {
            items.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            items.last().map(encode_outbox_list_cursor).transpose()?
        } else {
            None
        };

        Ok(EmailOutboxListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn undo(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        self.undo_with_observation(outbox_id, now, None, "outbox_status_transition", None)
            .await
    }

    pub async fn undo_with_observation(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'canceled',
                updated_at = $2
            WHERE outbox_id = $1
              AND status IN ('queued', 'scheduled')
              AND undo_deadline_at IS NOT NULL
              AND undo_deadline_at >= $2
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .fetch_optional(&mut *transaction)
            .await?;

        match row {
            Some(row) => {
                let item = row_to_outbox_item(row)?;
                if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
                    let link_metadata = merge_metadata(
                        json!({
                            "operation": "outbox_undo",
                            "status": item.status.as_str(),
                        }),
                        metadata,
                    );
                    link_mail_entity_in_transaction(
                        &mut transaction,
                        observation_id,
                        "outbox_item",
                        item.outbox_id.clone(),
                        relationship_kind,
                        link_metadata,
                        None,
                    )
                    .await?;
                }
                transaction.commit().await?;
                Ok(item)
            }
            None => {
                transaction.rollback().await?;
                Err(CommunicationOutboxError::UndoUnavailable)
            }
        }
    }

    pub async fn claim_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<CommunicationOutboxItem>, CommunicationOutboxError> {
        let limit = validate_limit(limit)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            WITH due AS (
                SELECT outbox_id
                FROM communication_outbox
                WHERE status IN ('queued', 'scheduled')
                  AND (scheduled_send_at IS NULL OR scheduled_send_at <= $1)
                  AND (undo_deadline_at IS NULL OR undo_deadline_at <= $1)
                ORDER BY COALESCE(scheduled_send_at, created_at) ASC, created_at ASC, outbox_id ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            UPDATE communication_outbox item
            SET status = 'sending',
                send_attempts = item.send_attempts + 1,
                claimed_at = $1,
                updated_at = $1
            FROM due
            WHERE item.outbox_id = due.outbox_id
            "#,
            "item",
        );
        let rows = sqlx::query(&sql)
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *transaction)
            .await?;
        let items = rows
            .into_iter()
            .map(row_to_outbox_item)
            .collect::<Result<Vec<_>, _>>()?;
        for item in &items {
            capture_outbox_transition_observation(
                &mut transaction,
                item,
                "outbox_claim_due",
                json!({
                    "status": item.status.as_str(),
                }),
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(items)
    }

    pub async fn mark_sent(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        receipt: &OutboxSendReceipt,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        validate_non_empty("provider_message_id", &receipt.provider_message_id)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'sent',
                sent_at = $2,
                provider_message_id = $3,
                last_error = NULL,
                updated_at = $2
            WHERE outbox_id = $1
              AND status = 'sending'
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .bind(receipt.provider_message_id.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(item) = row.map(row_to_outbox_item).transpose()? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        capture_outbox_transition_observation(
            &mut transaction,
            &item,
            "outbox_mark_sent",
            json!({
                "status": item.status.as_str(),
                "provider_message_id": item.provider_message_id,
            }),
        )
        .await?;
        let event = outbox_delivery_event("mail.outbox.sent", &item)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(item))
    }

    pub async fn mark_failed(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        error_message: &str,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        validate_non_empty("last_error", error_message)?;
        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'failed',
                sent_at = NULL,
                provider_message_id = NULL,
                last_error = $3,
                updated_at = $2
            WHERE outbox_id = $1
              AND status = 'sending'
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .bind(error_message.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(item) = row.map(row_to_outbox_item).transpose()? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        capture_outbox_transition_observation(
            &mut transaction,
            &item,
            "outbox_mark_failed",
            json!({
                "status": item.status.as_str(),
                "last_error": item.last_error,
            }),
        )
        .await?;
        let event = outbox_delivery_event("mail.outbox.failed", &item)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(item))
    }

    pub async fn mark_retry_scheduled(
        &self,
        outbox_id: &str,
        now: DateTime<Utc>,
        next_attempt_at: DateTime<Utc>,
        error_message: &str,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        validate_non_empty("outbox_id", outbox_id)?;
        validate_non_empty("last_error", error_message)?;
        if next_attempt_at <= now {
            return Err(CommunicationOutboxError::Invalid(
                "next_attempt_at must be after now",
            ));
        }

        let mut transaction = self.pool.begin().await?;
        let sql = outbox_returning_query(
            r#"
            UPDATE communication_outbox
            SET status = 'scheduled',
                scheduled_send_at = $3,
                claimed_at = NULL,
                sent_at = NULL,
                provider_message_id = NULL,
                last_error = $4,
                updated_at = $2
            WHERE outbox_id = $1
              AND status = 'sending'
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(outbox_id.trim())
            .bind(now)
            .bind(next_attempt_at)
            .bind(error_message.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(item) = row.map(row_to_outbox_item).transpose()? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        capture_outbox_transition_observation(
            &mut transaction,
            &item,
            "outbox_retry_scheduled",
            json!({
                "status": item.status.as_str(),
                "scheduled_send_at": item.scheduled_send_at,
                "last_error": item.last_error,
            }),
        )
        .await?;
        let event = outbox_delivery_event("mail.outbox.retry_scheduled", &item)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(Some(item))
    }
}

async fn capture_outbox_transition_observation(
    transaction: &mut Transaction<'_, Postgres>,
    item: &CommunicationOutboxItem,
    operation: &str,
    metadata: Value,
) -> Result<(), CommunicationOutboxError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "COMMUNICATION_OUTBOX",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "outbox_id": item.outbox_id,
                "account_id": item.account_id,
                "status": item.status.as_str(),
                "scheduled_send_at": item.scheduled_send_at,
                "undo_deadline_at": item.undo_deadline_at,
                "send_attempts": item.send_attempts,
                "provider_message_id": item.provider_message_id,
                "last_error": item.last_error,
                "operation": operation,
            }),
            format!("outbox://{}/{}", item.outbox_id, operation),
        )
        .provenance(json!({
            "captured_by": "mail.outbox",
            "operation": operation,
        })),
    )
    .await?;
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "outbox_item",
        item.outbox_id.clone(),
        "outbox_status_transition",
        metadata,
        None,
    )
    .await?;
    Ok(())
}

async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationOutboxError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn outbox_returning_query(prefix: &str, qualifier: &str) -> String {
    format!(
        r#"{prefix}
        RETURNING
            {qualifier}.outbox_id,
            {qualifier}.account_id,
            {qualifier}.draft_id,
            {qualifier}.to_participants AS to_recipients,
            {qualifier}.cc_participants AS cc_recipients,
            {qualifier}.bcc_participants AS bcc_recipients,
            {qualifier}.subject,
            {qualifier}.body_text,
            {qualifier}.body_html,
            {qualifier}.status,
            {qualifier}.scheduled_send_at,
            {qualifier}.undo_deadline_at,
            {qualifier}.send_attempts,
            {qualifier}.claimed_at,
            {qualifier}.sent_at,
            {qualifier}.provider_message_id,
            {qualifier}.last_error,
            {qualifier}.metadata,
            {qualifier}.created_at,
            {qualifier}.updated_at
        "#
    )
}

fn row_to_outbox_item(row: PgRow) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
    let status: String = row.try_get("status")?;
    Ok(CommunicationOutboxItem {
        outbox_id: row.try_get("outbox_id")?,
        account_id: row.try_get("account_id")?,
        draft_id: row.try_get("draft_id")?,
        to_recipients: string_array(row.try_get("to_recipients")?)?,
        cc_recipients: string_array(row.try_get("cc_recipients")?)?,
        bcc_recipients: string_array(row.try_get("bcc_recipients")?)?,
        subject: row.try_get("subject")?,
        body_text: row.try_get("body_text")?,
        body_html: row.try_get("body_html")?,
        status: CommunicationOutboxStatus::parse(&status)
            .unwrap_or(CommunicationOutboxStatus::Queued),
        scheduled_send_at: row.try_get("scheduled_send_at")?,
        undo_deadline_at: row.try_get("undo_deadline_at")?,
        send_attempts: row.try_get("send_attempts")?,
        claimed_at: row.try_get("claimed_at")?,
        sent_at: row.try_get("sent_at")?,
        provider_message_id: row.try_get("provider_message_id")?,
        last_error: row.try_get("last_error")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn string_array(value: Value) -> Result<Vec<String>, CommunicationOutboxError> {
    serde_json::from_value(value).map_err(CommunicationOutboxError::Serde)
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), CommunicationOutboxError> {
    if value.trim().is_empty() {
        return Err(CommunicationOutboxError::Invalid(field_name));
    }

    Ok(())
}

fn validate_limit(limit: i64) -> Result<i64, CommunicationOutboxError> {
    if !(1..=500).contains(&limit) {
        return Err(CommunicationOutboxError::Invalid(
            "limit must be between 1 and 500",
        ));
    }

    Ok(limit)
}

#[derive(Debug, Deserialize, Serialize)]
struct OutboxListCursor {
    created_at: DateTime<Utc>,
    outbox_id: String,
}

fn encode_outbox_list_cursor(
    item: &CommunicationOutboxItem,
) -> Result<String, CommunicationOutboxError> {
    let cursor = OutboxListCursor {
        created_at: item.created_at,
        outbox_id: item.outbox_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationOutboxError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_outbox_list_cursor(cursor: &str) -> Result<OutboxListCursor, CommunicationOutboxError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationOutboxError::InvalidCursor)?;
    let cursor: OutboxListCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationOutboxError::InvalidCursor)?;
    if cursor.outbox_id.trim().is_empty() {
        return Err(CommunicationOutboxError::InvalidCursor);
    }

    Ok(cursor)
}

fn outbox_delivery_event(
    event_type: &str,
    item: &CommunicationOutboxItem,
) -> Result<NewEventEnvelope, CommunicationOutboxError> {
    let recipient_count =
        item.to_recipients.len() + item.cc_recipients.len() + item.bcc_recipients.len();
    Ok(NewEventEnvelope::builder(
        generate_outbox_event_id(event_type, &item.outbox_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_outbox_worker" }),
        json!({
            "kind": "email_outbox",
            "id": item.outbox_id,
            "account_id": item.account_id,
            "status": item.status.as_str(),
        }),
    )
    .actor(json!({ "actor_id": "hermes-outbox-worker" }))
    .payload(json!({
        "outbox_id": item.outbox_id,
        "account_id": item.account_id,
        "status": item.status.as_str(),
        "provider_message_id": item.provider_message_id,
        "last_error": item.last_error,
        "send_attempts": item.send_attempts,
        "scheduled_send_at": item.scheduled_send_at,
        "undo_deadline_at": item.undo_deadline_at,
        "sent_at": item.sent_at,
        "recipient_count": recipient_count,
    }))
    .provenance(json!({
        "source_kind": "local_outbox",
        "source_id": item.outbox_id,
    }))
    .correlation_id(item.outbox_id.clone())
    .build()?)
}

fn generate_outbox_event_id(event_type: &str, outbox_id: &str) -> String {
    format!(
        "mail_outbox_event:{event_type}:{outbox_id}:{:x}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

#[derive(Debug, Error)]
pub enum CommunicationOutboxError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("invalid outbox item: {0}")]
    Invalid(&'static str),

    #[error("invalid outbox cursor")]
    InvalidCursor,

    #[error("outbox item cannot be undone")]
    UndoUnavailable,
}
