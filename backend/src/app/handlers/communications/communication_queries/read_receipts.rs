use super::super::*;
use crate::domains::communications::delivery_notifications::{
    CommunicationDeliveryNotificationRecord, CommunicationDeliveryNotificationStore,
    NewCommunicationDeliveryNotification, NewProviderDeliveryEvent,
};
use crate::domains::communications::read_receipts::{
    CommunicationReadReceipt, CommunicationReadReceiptStore, NewCommunicationReadReceipt,
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
    Ok(Json(
        delivery_notification_store(&state)?.record(request).await?,
    ))
}

pub(crate) async fn post_v1_provider_delivery_event(
    State(state): State<AppState>,
    Json(request): Json<NewProviderDeliveryEvent>,
) -> Result<Json<CommunicationDeliveryNotificationRecord>, ApiError> {
    Ok(Json(
        delivery_notification_store(&state)?
            .record_provider_event(request)
            .await?,
    ))
}

fn read_receipt_store(state: &AppState) -> Result<CommunicationReadReceiptStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<
        CommunicationReadReceiptStore,
    >(pool))
}

fn delivery_notification_store(
    state: &AppState,
) -> Result<CommunicationDeliveryNotificationStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<
        CommunicationDeliveryNotificationStore,
    >(pool))
}
