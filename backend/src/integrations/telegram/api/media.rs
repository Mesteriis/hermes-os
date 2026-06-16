use axum::Json;
use axum::extract::State;
use chrono::Utc;
use serde_json::json;

use super::helpers::{
    publish_telegram_event, telegram_message_snapshot_payload, telegram_secret_store,
};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    communication_ingestion_store, mail_storage_store, telegram_store,
};
use crate::integrations::telegram::runtime::{
    TelegramMediaDownloadContext, TelegramMediaDownloadRequest, TelegramMediaDownloadResponse,
};
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

fn build_event(account_id: &str, subject_id: &str, payload: serde_json::Value) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        telegram_event_types::MEDIA_DOWNLOADED.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_message"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

pub(crate) async fn post_telegram_media_download(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaDownloadRequest>,
) -> Result<Json<TelegramMediaDownloadResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    let communication_store = communication_ingestion_store(&state)?;
    let telegram_store = telegram_store(&state)?;
    let mail_store = mail_storage_store(&state)?;
    let response = state
        .telegram_runtime
        .download_media(
            TelegramMediaDownloadContext {
                communication_store: &communication_store,
                telegram_store: &telegram_store,
                mail_store: &mail_store,
                secret_store: &secret_store,
                secret_resolver: &state.vault,
                config: &state.config,
            },
            &request,
        )
        .await?;

    if response.is_downloading_completed {
        let attachment_anchor = telegram_store
            .attachment_anchor_for_message(
                &request.account_id,
                &request.provider_chat_id,
                &request.provider_message_id,
            )
            .await?;
        telegram_store
            .update_message_attachment_download_state(
                &attachment_anchor.message_id,
                &request.provider_attachment_id(),
                response.tdlib_file_id,
                &response.status,
                response.local_path.as_deref(),
                response.size_bytes,
                &request.content_type(),
                request.filename().as_deref(),
            )
            .await?;
        let event = build_event(
            &request.account_id,
            &attachment_anchor.message_id,
            telegram_message_snapshot_payload(
                &telegram_store,
                &attachment_anchor.message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "tdlib_file_id": response.tdlib_file_id,
                    "download_state": &response.status,
                    "local_path": response.local_path.clone(),
                    "attachment_id": response.attachment_id.clone(),
                    "blob_id": response.blob_id.clone(),
                    "scan_status": response.scan_status.clone(),
                }),
            )
            .await?,
        );
        publish_telegram_event(&state, event).await?;
    }

    Ok(Json(response))
}
