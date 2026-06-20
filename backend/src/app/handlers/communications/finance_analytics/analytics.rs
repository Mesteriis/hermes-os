use super::super::*;

#[derive(Deserialize)]
pub(crate) struct AnalyticsQuery {
    pub(super) account_id: Option<String>,
}

pub(crate) async fn get_v1_analytics_health(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<crate::domains::communications::analytics::MailboxHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::communications::analytics::EmailAnalyticsStore::new(pool);
    let health = store.mailbox_health(query.account_id.as_deref()).await?;
    Ok(Json(health))
}

#[derive(Deserialize)]
pub(crate) struct SendersQuery {
    pub(super) account_id: Option<String>,
    pub(super) limit: Option<i64>,
    pub(super) cursor: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SendersResponse {
    pub(super) items: Vec<crate::domains::communications::analytics::SenderStats>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

pub(crate) async fn get_v1_analytics_senders(
    State(state): State<AppState>,
    Query(query): Query<SendersQuery>,
) -> Result<Json<SendersResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::communications::analytics::EmailAnalyticsStore::new(pool);
    let page = store
        .top_senders_page(
            query.account_id.as_deref(),
            query.limit.unwrap_or(20),
            query.cursor.as_deref(),
        )
        .await?;
    Ok(Json(SendersResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}
