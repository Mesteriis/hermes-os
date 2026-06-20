use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::Value;

use crate::app::api_support::{communication_ingestion_store, telegram_store};
use crate::app::{ApiError, AppState};
use crate::domains::communications::core::StoredRawCommunicationRecord;

#[derive(Serialize)]
pub(crate) struct TelegramRawMessageResponse {
    pub(crate) raw_record: TelegramRawMessageRecord,
}

#[derive(Serialize)]
pub(crate) struct TelegramRawMessageRecord {
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) record_kind: String,
    pub(crate) provider_record_id: String,
    pub(crate) source_fingerprint: String,
    pub(crate) import_batch_id: String,
    pub(crate) occurred_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) captured_at: chrono::DateTime<chrono::Utc>,
    pub(crate) payload: Value,
    pub(crate) provenance: Value,
}

impl From<StoredRawCommunicationRecord> for TelegramRawMessageRecord {
    fn from(record: StoredRawCommunicationRecord) -> Self {
        Self {
            raw_record_id: record.raw_record_id,
            account_id: record.account_id,
            record_kind: record.record_kind,
            provider_record_id: record.provider_record_id,
            source_fingerprint: record.source_fingerprint,
            import_batch_id: record.import_batch_id,
            occurred_at: record.occurred_at,
            captured_at: record.captured_at,
            payload: redact_secret_material(record.payload),
            provenance: redact_secret_material(record.provenance),
        }
    }
}

/// GET /api/v1/communications/telegram/messages/{message_id}/raw
pub(crate) async fn get_telegram_message_raw(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramRawMessageResponse>, ApiError> {
    let Some(message) = telegram_store(&state)?.message_by_id(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let Some(raw_record) = communication_ingestion_store(&state)?
        .raw_record(&message.raw_record_id)
        .await?
    else {
        return Err(ApiError::CommunicationMessageNotFound);
    };

    Ok(Json(TelegramRawMessageResponse {
        raw_record: raw_record.into(),
    }))
}

fn redact_secret_material(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    if is_secret_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_material(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_material).collect())
        }
        other => other,
    }
}

fn is_secret_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "access_token"
            | "api_hash"
            | "bot_token"
            | "client_secret"
            | "password"
            | "proxy_password"
            | "secret"
            | "session_encryption_key"
            | "session_key"
            | "token"
    )
}
