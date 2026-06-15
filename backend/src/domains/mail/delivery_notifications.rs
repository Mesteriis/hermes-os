use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::outbox::{
    EmailOutboxStore, NewOutboxDeliveryStatus, OutboxDeliveryStatus, OutboxDeliveryStatusRecord,
};
use super::read_receipts::{MailReadReceipt, MailReadReceiptStore, NewMailReadReceipt};

const MAX_NOTIFICATION_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize)]
pub struct NewMailDeliveryNotification {
    pub account_id: String,
    pub raw_message: String,
    pub received_at: Option<DateTime<Utc>>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderDeliveryEventKind {
    Delivered,
    Delayed,
    Failed,
    Read,
}

impl ProviderDeliveryEventKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Delayed => "delayed",
            Self::Failed => "failed",
            Self::Read => "read",
        }
    }

    fn delivery_status(self) -> Option<OutboxDeliveryStatus> {
        match self {
            Self::Delivered => Some(OutboxDeliveryStatus::Delivered),
            Self::Delayed => Some(OutboxDeliveryStatus::Delayed),
            Self::Failed => Some(OutboxDeliveryStatus::Failed),
            Self::Read => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewProviderDeliveryEvent {
    pub account_id: String,
    pub provider_message_id: String,
    pub event_kind: ProviderDeliveryEventKind,
    pub recipient: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub source_kind: Option<String>,
    pub smtp_status: Option<String>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailDeliveryNotificationKind {
    DeliveryStatus,
    ReadReceipt,
}

#[derive(Clone, Debug, Serialize)]
pub struct MailDeliveryNotificationRecord {
    pub notification_kind: MailDeliveryNotificationKind,
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub delivery_status: Option<OutboxDeliveryStatus>,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub read_receipt: Option<MailReadReceipt>,
}

#[derive(Clone)]
pub struct MailDeliveryNotificationStore {
    pool: PgPool,
}

impl MailDeliveryNotificationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        notification: NewMailDeliveryNotification,
    ) -> Result<MailDeliveryNotificationRecord, MailDeliveryNotificationError> {
        let account_id = normalize_required("account_id", &notification.account_id)?;
        let received_at = notification.received_at.unwrap_or_else(Utc::now);
        let provider_record_id = normalize_optional(notification.provider_record_id);
        let raw_record_id = normalize_optional(notification.raw_record_id);
        let parsed = parse_delivery_notification(&notification.raw_message)?;

        match parsed {
            ParsedDeliveryNotification::DeliveryStatus {
                provider_message_id,
                delivery_status,
                smtp_status,
            } => {
                let record = EmailOutboxStore::new(self.pool.clone())
                    .record_delivery_status(&NewOutboxDeliveryStatus {
                        account_id,
                        provider_message_id,
                        delivery_status,
                        smtp_status,
                        source_kind: "dsn".to_owned(),
                        provider_record_id,
                        raw_record_id,
                        recorded_at: received_at,
                    })
                    .await?;
                Ok(delivery_status_response(record))
            }
            ParsedDeliveryNotification::ReadReceipt {
                provider_message_id,
                recipient,
                reporting_ua,
            } => {
                let receipt = MailReadReceiptStore::new(self.pool.clone())
                    .record(NewMailReadReceipt {
                        receipt_id: None,
                        account_id,
                        provider_message_id,
                        recipient,
                        read_at: received_at,
                        source_kind: Some("mdn".to_owned()),
                        provider_record_id,
                        raw_record_id,
                        metadata: Some(json!({ "reporting_ua": reporting_ua })),
                    })
                    .await?;
                Ok(read_receipt_response(receipt))
            }
        }
    }

    pub async fn record_provider_event(
        &self,
        event: NewProviderDeliveryEvent,
    ) -> Result<MailDeliveryNotificationRecord, MailDeliveryNotificationError> {
        let account_id = normalize_required("account_id", &event.account_id)?;
        let provider_message_id =
            normalize_required("provider_message_id", &event.provider_message_id)?;
        let occurred_at = event.occurred_at.unwrap_or_else(Utc::now);
        let source_kind =
            normalize_optional(event.source_kind).unwrap_or_else(|| "provider_event".to_owned());
        let provider_record_id = normalize_optional(event.provider_record_id);
        let raw_record_id = normalize_optional(event.raw_record_id);

        if let Some(delivery_status) = event.event_kind.delivery_status() {
            let record = EmailOutboxStore::new(self.pool.clone())
                .record_delivery_status(&NewOutboxDeliveryStatus {
                    account_id,
                    provider_message_id,
                    delivery_status,
                    smtp_status: normalize_optional(event.smtp_status),
                    source_kind,
                    provider_record_id,
                    raw_record_id,
                    recorded_at: occurred_at,
                })
                .await?;
            return Ok(delivery_status_response(record));
        }

        let recipient = normalize_optional(event.recipient)
            .ok_or(MailDeliveryNotificationError::Invalid("recipient"))?;
        let receipt = MailReadReceiptStore::new(self.pool.clone())
            .record(NewMailReadReceipt {
                receipt_id: None,
                account_id,
                provider_message_id,
                recipient,
                read_at: occurred_at,
                source_kind: Some(source_kind),
                provider_record_id,
                raw_record_id,
                metadata: Some(json!({ "provider_event_kind": event.event_kind.as_str() })),
            })
            .await?;

        Ok(read_receipt_response(receipt))
    }
}

fn delivery_status_response(record: OutboxDeliveryStatusRecord) -> MailDeliveryNotificationRecord {
    MailDeliveryNotificationRecord {
        notification_kind: MailDeliveryNotificationKind::DeliveryStatus,
        account_id: record.account_id,
        outbox_id: record.outbox_id,
        provider_message_id: record.provider_message_id,
        delivery_status: Some(record.delivery_status),
        smtp_status: record.smtp_status,
        source_kind: record.source_kind,
        read_receipt: None,
    }
}

fn read_receipt_response(receipt: MailReadReceipt) -> MailDeliveryNotificationRecord {
    MailDeliveryNotificationRecord {
        notification_kind: MailDeliveryNotificationKind::ReadReceipt,
        account_id: receipt.account_id.clone(),
        outbox_id: receipt.outbox_id.clone(),
        provider_message_id: receipt.provider_message_id.clone(),
        delivery_status: None,
        smtp_status: None,
        source_kind: receipt.source_kind.clone(),
        read_receipt: Some(receipt),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ParsedDeliveryNotification {
    DeliveryStatus {
        provider_message_id: String,
        delivery_status: OutboxDeliveryStatus,
        smtp_status: Option<String>,
    },
    ReadReceipt {
        provider_message_id: String,
        recipient: String,
        reporting_ua: Option<String>,
    },
}

fn parse_delivery_notification(
    raw_message: &str,
) -> Result<ParsedDeliveryNotification, MailDeliveryNotificationError> {
    if raw_message.trim().is_empty() {
        return Err(MailDeliveryNotificationError::Invalid("raw_message"));
    }
    if raw_message.len() > MAX_NOTIFICATION_BYTES {
        return Err(MailDeliveryNotificationError::Invalid("raw_message"));
    }

    let fields = unfolded_fields(raw_message);
    if let Some(disposition) = first_field(&fields, "disposition")
        && disposition.to_ascii_lowercase().contains("displayed")
    {
        return Ok(ParsedDeliveryNotification::ReadReceipt {
            provider_message_id: original_message_id(&fields)?,
            recipient: recipient_from_fields(&fields)?,
            reporting_ua: first_field(&fields, "reporting-ua"),
        });
    }

    let action =
        first_field(&fields, "action").ok_or(MailDeliveryNotificationError::Invalid("action"))?;
    Ok(ParsedDeliveryNotification::DeliveryStatus {
        provider_message_id: original_message_id(&fields)?,
        delivery_status: delivery_status_from_action(&action)?,
        smtp_status: first_field(&fields, "status")
            .and_then(|value| normalize_optional(Some(value))),
    })
}

fn unfolded_fields(raw_message: &str) -> Vec<(String, String)> {
    let normalized = raw_message.replace("\r\n", "\n");
    let mut fields: Vec<(String, String)> = Vec::new();

    for line in normalized.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, value)) = fields.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if !name.is_empty() {
            fields.push((name, value.trim().to_owned()));
        }
    }

    fields
}

fn original_message_id(
    fields: &[(String, String)],
) -> Result<String, MailDeliveryNotificationError> {
    first_field(fields, "original-message-id")
        .or_else(|| first_field(fields, "message-id"))
        .and_then(|value| normalize_message_id(&value))
        .ok_or(MailDeliveryNotificationError::Invalid(
            "original-message-id",
        ))
}

fn recipient_from_fields(
    fields: &[(String, String)],
) -> Result<String, MailDeliveryNotificationError> {
    first_field(fields, "final-recipient")
        .or_else(|| first_field(fields, "original-recipient"))
        .and_then(|value| recipient_from_report_field(&value))
        .ok_or(MailDeliveryNotificationError::Invalid("final-recipient"))
}

fn delivery_status_from_action(
    action: &str,
) -> Result<OutboxDeliveryStatus, MailDeliveryNotificationError> {
    match action.trim().to_ascii_lowercase().as_str() {
        "delivered" | "relayed" | "expanded" => Ok(OutboxDeliveryStatus::Delivered),
        "delayed" => Ok(OutboxDeliveryStatus::Delayed),
        "failed" => Ok(OutboxDeliveryStatus::Failed),
        _ => Err(MailDeliveryNotificationError::Invalid("action")),
    }
}

fn first_field(fields: &[(String, String)], name: &str) -> Option<String> {
    fields
        .iter()
        .find(|(field_name, _)| field_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_message_id(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .strip_prefix('<')
        .and_then(|stripped| stripped.strip_suffix('>'))
        .unwrap_or(value)
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn recipient_from_report_field(value: &str) -> Option<String> {
    let recipient = value
        .split_once(';')
        .map(|(_, recipient)| recipient)
        .unwrap_or(value)
        .trim();
    normalize_optional(Some(recipient))
}

fn normalize_required(
    field_name: &'static str,
    value: &str,
) -> Result<String, MailDeliveryNotificationError> {
    normalize_optional(Some(value)).ok_or(MailDeliveryNotificationError::Invalid(field_name))
}

fn normalize_optional(value: Option<impl AsRef<str>>) -> Option<String> {
    value
        .map(|value| value.as_ref().trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Error)]
pub enum MailDeliveryNotificationError {
    #[error(transparent)]
    Outbox(#[from] super::outbox::EmailOutboxError),
    #[error(transparent)]
    ReadReceipt(#[from] super::read_receipts::MailReadReceiptError),
    #[error("invalid mail delivery notification field: {0}")]
    Invalid(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dsn_delivery_status() {
        let parsed = parse_delivery_notification(
            "Original-Message-ID: <provider-1>\r\nFinal-Recipient: rfc822; user@example.com\r\nAction: failed\r\nStatus: 5.1.1\r\n",
        )
        .expect("parse dsn");

        assert_eq!(
            parsed,
            ParsedDeliveryNotification::DeliveryStatus {
                provider_message_id: "provider-1".to_owned(),
                delivery_status: OutboxDeliveryStatus::Failed,
                smtp_status: Some("5.1.1".to_owned())
            }
        );
    }

    #[test]
    fn parse_mdn_read_receipt() {
        let parsed = parse_delivery_notification(
            "Original-Message-ID: <provider-2>\r\nFinal-Recipient: rfc822; reader@example.com\r\nDisposition: automatic-action/MDN-sent-automatically; displayed\r\nReporting-UA: fixture\r\n",
        )
        .expect("parse mdn");

        assert_eq!(
            parsed,
            ParsedDeliveryNotification::ReadReceipt {
                provider_message_id: "provider-2".to_owned(),
                recipient: "reader@example.com".to_owned(),
                reporting_ua: Some("fixture".to_owned())
            }
        );
    }
}
