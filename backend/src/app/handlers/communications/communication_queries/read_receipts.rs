use super::super::*;
use crate::domains::communications::delivery_notifications::{
    CommunicationDeliveryNotificationError, CommunicationDeliveryNotificationRecord,
    NewCommunicationDeliveryNotification, NewProviderDeliveryEvent,
    project_accepted_mail_delivery_signal_if_runtime_allows,
    provider_event_from_delivery_notification,
};
use crate::domains::communications::read_receipts::{
    CommunicationReadReceipt, CommunicationReadReceiptStore, NewCommunicationReadReceipt,
};
use crate::domains::signal_hub::mail::{
    MailDeliverySignalRequest, dispatch_mail_delivery_event_signal,
};

pub(crate) async fn post_v1_read_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewCommunicationReadReceipt>,
) -> Result<Json<CommunicationReadReceipt>, ApiError> {
    Ok(Json(read_receipt_store(&state)?.record(request).await?))
}

pub(crate) async fn post_v1_delivery_notification(
    State(state): State<AppState>,
    Json(request): Json<NewCommunicationDeliveryNotification>,
) -> Result<Json<CommunicationDeliveryNotificationRecord>, ApiError> {
    let provider_event = provider_event_from_delivery_notification(&request)?;
    Ok(Json(
        dispatch_provider_delivery_event(state, provider_event)
            .await?
            .ok_or_else(|| {
                CommunicationDeliveryNotificationError::SignalControlBlocked(
                    "mail delivery notification was accepted by Signal Hub but deferred by runtime control"
                        .to_owned(),
                )
            })?,
    ))
}

pub(crate) async fn post_v1_provider_delivery_event(
    State(state): State<AppState>,
    Json(request): Json<NewProviderDeliveryEvent>,
) -> Result<Json<CommunicationDeliveryNotificationRecord>, ApiError> {
    Ok(Json(
        dispatch_provider_delivery_event(state, request)
            .await?
            .ok_or_else(|| {
                CommunicationDeliveryNotificationError::SignalControlBlocked(
                    "mail provider delivery event was accepted by Signal Hub but deferred by runtime control"
                        .to_owned(),
                )
            })?,
    ))
}

fn read_receipt_store(state: &AppState) -> Result<CommunicationReadReceiptStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationReadReceiptStore,
    >(pool))
}

async fn dispatch_provider_delivery_event(
    state: AppState,
    request: NewProviderDeliveryEvent,
) -> Result<Option<CommunicationDeliveryNotificationRecord>, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let occurred_at = request.occurred_at.unwrap_or_else(chrono::Utc::now);
    let account_id = request.account_id.clone();
    let provider_message_id = request.provider_message_id.clone();
    let source_kind = request
        .source_kind
        .clone()
        .unwrap_or_else(|| "provider_event".to_owned());
    let provider_record_id = request.provider_record_id.clone();
    let raw_record_id = request.raw_record_id.clone();
    let payload = serde_json::json!({
        "account_id": request.account_id,
        "provider_message_id": request.provider_message_id,
        "event_kind": request.event_kind.as_str(),
        "occurred_at": occurred_at,
        "recipient": request.recipient,
        "source_kind": request.source_kind,
        "smtp_status": request.smtp_status,
        "provider_record_id": provider_record_id,
        "raw_record_id": raw_record_id,
        "reporting_ua": request.metadata.as_ref().and_then(|m| m.get("reporting_ua")).cloned(),
    });
    let event_kind = match request.event_kind {
        crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Read => {
            "read_receipt"
        }
        crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Delivered
        | crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Delayed
        | crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Failed => {
            "delivery_status"
        }
    };
    let accepted = dispatch_mail_delivery_event_signal(
        pool.clone(),
        MailDeliverySignalRequest {
            occurred_at,
            account_id: &account_id,
            provider_message_id: &provider_message_id,
            event_kind,
            payload,
            source_kind: &source_kind,
            provider_record_id: provider_record_id.as_deref(),
            raw_record_id: raw_record_id.as_deref(),
            correlation_id: provider_record_id.as_deref().or(raw_record_id.as_deref()),
        },
    )
    .await?;

    let Some(accepted_event) = accepted else {
        return Ok(None);
    };

    project_accepted_mail_delivery_signal_if_runtime_allows(pool, &accepted_event)
        .await
        .map_err(ApiError::from)
}
