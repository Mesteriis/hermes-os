use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;

use crate::platform::events::{EventStore, NewEventEnvelope};

use super::{EmailOutboxError, EmailOutboxStore, generate_outbox_event_id, validate_non_empty};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxDeliveryStatus {
    Delivered,
    Delayed,
    Failed,
}

impl OutboxDeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Delayed => "delayed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewOutboxDeliveryStatus {
    pub account_id: String,
    pub provider_message_id: String,
    pub delivery_status: OutboxDeliveryStatus,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutboxDeliveryStatusRecord {
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub delivery_status: OutboxDeliveryStatus,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

impl EmailOutboxStore {
    pub async fn record_delivery_status(
        &self,
        delivery_status: &NewOutboxDeliveryStatus,
    ) -> Result<OutboxDeliveryStatusRecord, EmailOutboxError> {
        let account_id = normalize_non_empty("account_id", &delivery_status.account_id)?;
        let provider_message_id =
            normalize_non_empty("provider_message_id", &delivery_status.provider_message_id)?;
        let source_kind = normalize_non_empty("source_kind", &delivery_status.source_kind)?;
        let smtp_status = normalize_optional(delivery_status.smtp_status.as_deref());
        let provider_record_id = normalize_optional(delivery_status.provider_record_id.as_deref());
        let raw_record_id = normalize_optional(delivery_status.raw_record_id.as_deref());
        let metadata = json!({
            "delivery_status": delivery_status.delivery_status.as_str(),
            "smtp_status": smtp_status,
            "source_kind": source_kind,
            "provider_record_id": provider_record_id,
            "recorded_at": delivery_status.recorded_at,
        });
        let terminal_error = match (delivery_status.delivery_status, smtp_status.as_deref()) {
            (OutboxDeliveryStatus::Failed, Some(status)) => {
                Some(format!("delivery failed with SMTP status {status}"))
            }
            (OutboxDeliveryStatus::Failed, None) => Some("delivery failed".to_owned()),
            _ => None,
        };

        let mut transaction = self.pool.begin().await?;
        let outbox_id = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE email_outbox_tracking
            SET metadata = jsonb_set(metadata, '{delivery_status}', $3::jsonb, true),
                last_error = CASE
                    WHEN $4::text IS NULL THEN last_error
                    ELSE $4
                END,
                updated_at = $5
            WHERE account_id = $1
              AND provider_message_id = $2
            RETURNING outbox_id
            "#,
        )
        .bind(&account_id)
        .bind(&provider_message_id)
        .bind(&metadata)
        .bind(terminal_error.as_deref())
        .bind(delivery_status.recorded_at)
        .fetch_optional(&mut *transaction)
        .await?;
        let record = OutboxDeliveryStatusRecord {
            account_id,
            outbox_id,
            provider_message_id,
            delivery_status: delivery_status.delivery_status,
            smtp_status,
            source_kind,
            provider_record_id,
            raw_record_id,
            recorded_at: delivery_status.recorded_at,
        };
        let event = outbox_delivery_status_event(&record)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(record)
    }
}

fn normalize_non_empty(field_name: &'static str, value: &str) -> Result<String, EmailOutboxError> {
    validate_non_empty(field_name, value)?;
    Ok(value.trim().to_owned())
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn outbox_delivery_status_event(
    record: &OutboxDeliveryStatusRecord,
) -> Result<NewEventEnvelope, EmailOutboxError> {
    let subject_id = record
        .outbox_id
        .as_deref()
        .unwrap_or(record.provider_message_id.as_str());
    Ok(NewEventEnvelope::builder(
        generate_outbox_event_id("mail.outbox.delivery_status_changed", subject_id),
        "mail.outbox.delivery_status_changed",
        Utc::now(),
        json!({ "kind": "mail_delivery_notification" }),
        json!({
            "kind": "email_outbox_delivery_status",
            "id": subject_id,
            "account_id": record.account_id,
            "outbox_id": record.outbox_id,
        }),
    )
    .actor(json!({ "actor_id": "hermes-delivery-notification" }))
    .payload(json!({
        "account_id": record.account_id,
        "outbox_id": record.outbox_id,
        "provider_message_id": record.provider_message_id,
        "delivery_status": record.delivery_status.as_str(),
        "smtp_status": record.smtp_status,
        "source_kind": record.source_kind,
        "recorded_at": record.recorded_at,
    }))
    .provenance(json!({
        "source_kind": record.source_kind,
        "source_id": record.provider_record_id,
    }))
    .correlation_id(subject_id.to_owned())
    .build()?)
}
