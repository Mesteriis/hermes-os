use super::super::*;
use crate::domains::mail::delivery_notifications::{
    MailDeliveryNotificationRecord, MailDeliveryNotificationStore, NewMailDeliveryNotification,
    NewProviderDeliveryEvent,
};
use crate::domains::mail::read_receipts::{
    MailReadReceipt, MailReadReceiptStore, NewMailReadReceipt,
};

pub(crate) async fn post_v1_read_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewMailReadReceipt>,
) -> Result<Json<MailReadReceipt>, ApiError> {
    Ok(Json(read_receipt_store(&state)?.record(request).await?))
}

pub(crate) async fn post_v1_delivery_notification(
    State(state): State<AppState>,
    Json(request): Json<NewMailDeliveryNotification>,
) -> Result<Json<MailDeliveryNotificationRecord>, ApiError> {
    Ok(Json(
        delivery_notification_store(&state)?.record(request).await?,
    ))
}

pub(crate) async fn post_v1_provider_delivery_event(
    State(state): State<AppState>,
    Json(request): Json<NewProviderDeliveryEvent>,
) -> Result<Json<MailDeliveryNotificationRecord>, ApiError> {
    Ok(Json(
        delivery_notification_store(&state)?
            .record_provider_event(request)
            .await?,
    ))
}

fn read_receipt_store(state: &AppState) -> Result<MailReadReceiptStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(MailReadReceiptStore::new(pool))
}

fn delivery_notification_store(
    state: &AppState,
) -> Result<MailDeliveryNotificationStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(MailDeliveryNotificationStore::new(pool))
}
