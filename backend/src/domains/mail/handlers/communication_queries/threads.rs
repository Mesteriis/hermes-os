use super::super::*;

pub(crate) async fn get_v1_threads(
    State(state): State<AppState>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<ThreadListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::threads::EmailThreadStore::new(pool);
    let items = store
        .list_threads(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadListResponse { items }))
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
    let store = crate::domains::mail::threads::EmailThreadStore::new(pool);
    let items = store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadMessagesResponse { items }))
}
