use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::platform::events::bus::whatsapp_event_types;

use super::{
    WhatsAppCallsSyncRequest, WhatsAppCallsSyncResponse, capture_whatsapp_sync_runtime_signal,
    current_whatsapp_runtime_kind, ensure_whatsapp_sync_supported, list_whatsapp_sync_calls,
    publish_whatsapp_sync_event, required_string,
};

pub(crate) async fn post_whatsapp_sync_calls(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppCallsSyncRequest>,
) -> Result<Json<WhatsAppCallsSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_calls").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    let provider_chat_id = request
        .provider_chat_id
        .as_deref()
        .map(|value| required_string("provider_chat_id", value))
        .transpose()?;
    let subject_id = provider_chat_id
        .clone()
        .unwrap_or_else(|| account_id.clone());
    let started = json!({"scope": "calls", "provider_chat_id": provider_chat_id});
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "calls",
        "started",
        started.clone(),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &subject_id,
        started,
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items = match list_whatsapp_sync_calls(
        &state,
        &account_id,
        provider_chat_id.as_deref(),
        limit,
    )
    .await
    {
        Ok(items) => items,
        Err(error) => {
            let failed =
                json!({"scope": "calls", "provider_chat_id": provider_chat_id, "status": "failed"});
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &subject_id,
                "calls",
                "failed",
                failed.clone(),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &subject_id,
                failed,
            )
            .await?;
            return Err(error);
        }
    };
    let response = WhatsAppCallsSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    let progress = json!({"scope": "calls", "provider_chat_id": provider_chat_id, "status": response.status, "synced_count": response.synced_count, "has_more": response.has_more});
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "calls",
        "progress",
        progress.clone(),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &subject_id,
        progress.clone(),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "calls",
        "completed",
        progress.clone(),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &subject_id,
        progress,
    )
    .await?;
    Ok(Json(response))
}
