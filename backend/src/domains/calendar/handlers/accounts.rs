use super::*;

#[derive(Serialize)]
pub(crate) struct CalendarAccountsResponse {
    items: Vec<crate::domains::calendar::events::CalendarAccount>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarAccountQuery {
    provider: Option<String>,
}

pub(crate) async fn get_calendar_accounts(
    State(state): State<AppState>,
    Query(query): Query<CalendarAccountQuery>,
) -> Result<Json<CalendarAccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarAccountStore::new(pool)
        .list(query.provider.as_deref())
        .await?;
    Ok(Json(CalendarAccountsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewCalendarAccountRequest {
    provider: String,
    account_name: String,
    email: Option<String>,
}

pub(crate) async fn post_calendar_account(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarAccountRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let acct = CalendarAccountStore::new(pool)
        .create(&req.provider, &req.account_name, req.email.as_deref())
        .await?;
    Ok(Json(acct))
}

pub(crate) async fn get_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool)
        .get(&account_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(update): Json<CalendarAccountUpdate>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let acct = CalendarAccountStore::new(pool)
        .update(&account_id, &update)
        .await?;
    Ok(Json(acct))
}

pub(crate) async fn delete_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool).delete(&account_id).await?;
    Ok(Json(json!({"deleted": true})))
}

// ── Calendar Sources ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarSourcesResponse {
    items: Vec<crate::domains::calendar::events::CalendarSource>,
}

pub(crate) async fn get_calendar_sources(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<CalendarSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarSourceStore::new(pool)
        .list_by_account(&account_id)
        .await?;
    Ok(Json(CalendarSourcesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewCalendarSourceRequest {
    name: String,
    provider_calendar_id: Option<String>,
    color: Option<String>,
    timezone: Option<String>,
}

pub(crate) async fn post_calendar_source(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(req): Json<NewCalendarSourceRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarSource>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let src = CalendarSourceStore::new(pool)
        .create(
            &account_id,
            &req.name,
            req.provider_calendar_id.as_deref(),
            req.color.as_deref(),
            req.timezone.as_deref(),
        )
        .await?;
    Ok(Json(src))
}
