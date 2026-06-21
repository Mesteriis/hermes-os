use super::super::*;
use crate::domains::communications::service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct OutboxListQuery {
    account_id: Option<String>,
    status: Option<String>,
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct OutboxListResponse {
    items: Vec<crate::domains::communications::outbox::CommunicationOutboxItem>,
    next_cursor: Option<String>,
    has_more: bool,
}

pub(crate) async fn get_v1_outbox(
    State(state): State<AppState>,
    Query(query): Query<OutboxListQuery>,
) -> Result<Json<OutboxListResponse>, ApiError> {
    let status = match query.status.as_deref() {
        Some(value) => Some(
            crate::domains::communications::outbox::CommunicationOutboxStatus::parse(value).ok_or(
                ApiError::InvalidCommunicationQuery("invalid outbox status value"),
            )?,
        ),
        None => None,
    };
    let page = outbox_store(&state)?
        .list_page(
            query.account_id.as_deref(),
            status,
            query.cursor.as_deref(),
            query.limit.unwrap_or(100),
        )
        .await?;

    Ok(Json(OutboxListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn post_v1_outbox_undo(
    State(state): State<AppState>,
    Path(outbox_id): Path<String>,
) -> Result<Json<crate::domains::communications::outbox::CommunicationOutboxItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = CommunicationCommandService::new(pool)
        .undo_outbox(&outbox_id)
        .await?;

    Ok(Json(item))
}

pub(super) fn outbox_store(
    state: &AppState,
) -> Result<crate::domains::communications::outbox::CommunicationOutboxStore, ApiError> {
    Ok(crate::app::api_support::app_store::<
        crate::domains::communications::outbox::CommunicationOutboxStore,
    >(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    ))
}
