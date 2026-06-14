use super::super::*;

#[derive(Deserialize)]
pub(crate) struct AnalyticsQuery {
    pub(super) account_id: Option<String>,
}

pub(crate) async fn get_v1_analytics_health(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<crate::domains::mail::analytics::MailboxHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::analytics::EmailAnalyticsStore::new(pool);
    let health = store.mailbox_health(query.account_id.as_deref()).await?;
    Ok(Json(health))
}

#[derive(Deserialize)]
pub(crate) struct SendersQuery {
    pub(super) account_id: Option<String>,
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_analytics_senders(
    State(state): State<AppState>,
    Query(query): Query<SendersQuery>,
) -> Result<Json<Vec<crate::domains::mail::analytics::SenderStats>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::analytics::EmailAnalyticsStore::new(pool);
    let senders = store
        .top_senders(query.account_id.as_deref(), query.limit.unwrap_or(20))
        .await?;
    Ok(Json(senders))
}
