use super::super::*;
use crate::app::handlers::communications::workflow_state::{
    ThreadListQuery, ThreadListResponse, ThreadMessagesQuery, ThreadMessagesResponse,
};

pub(crate) async fn get_v1_threads(
    State(state): State<AppState>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<ThreadListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::threads::CommunicationThreadStore,
    >(pool);
    let page = store
        .list_threads_page(
            query.account_id.as_deref(),
            query.cursor.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(ThreadListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn get_v1_thread_messages(
    State(state): State<AppState>,
    Query(query): Query<ThreadMessagesQuery>,
) -> Result<Json<ThreadMessagesResponse>, ApiError> {
    let account_id = query
        .account_id
        .as_deref()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ))?;
    let subject = query
        .subject
        .as_deref()
        .ok_or(ApiError::InvalidCommunicationQuery("subject is required"))?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::communications::threads::CommunicationThreadStore,
    >(pool);
    let items = store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadMessagesResponse { items }))
}
