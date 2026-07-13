//! Hidden WebView WhatsApp status synchronization endpoint.

use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::app::api_support::stores::integration_stores::whatsapp_provider_runtime_service;
use crate::app::{ApiError, AppState};
use crate::platform::events::bus::whatsapp_event_types;

use super::{
    WhatsAppStatusSyncRequest, WhatsAppStatusSyncResponse, capture_whatsapp_sync_runtime_signal,
    current_whatsapp_runtime_kind, ensure_whatsapp_sync_supported, publish_whatsapp_sync_event,
    required_string,
};

pub(crate) async fn post_whatsapp_sync_statuses(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppStatusSyncRequest>,
) -> Result<Json<WhatsAppStatusSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_statuses").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    let provider_chat_id = "status-feed".to_owned();
    let started = json!({"scope": "statuses", "provider_chat_id": provider_chat_id});
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "statuses",
        "started",
        started.clone(),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &provider_chat_id,
        started,
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let mut items = match whatsapp_provider_runtime_service(&state)?
        .recent_messages(Some(&account_id), Some(&provider_chat_id), limit)
        .await
    {
        Ok(items) => items,
        Err(error) => {
            let failed = json!({"scope": "statuses", "provider_chat_id": provider_chat_id, "status": "failed"});
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &provider_chat_id,
                "statuses",
                "failed",
                failed.clone(),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &provider_chat_id,
                failed,
            )
            .await?;
            return Err(error.into());
        }
    };
    for item in &mut items {
        item.provider_chat_id = Some(provider_chat_id.clone());
    }
    let response = WhatsAppStatusSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    let progress = json!({"scope": "statuses", "provider_chat_id": provider_chat_id, "status": response.status, "synced_count": response.synced_count, "has_more": response.has_more});
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "statuses",
        "progress",
        progress.clone(),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &provider_chat_id,
        progress.clone(),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "statuses",
        "completed",
        progress.clone(),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &provider_chat_id,
        progress,
    )
    .await?;
    Ok(Json(response))
}
