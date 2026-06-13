use super::*;

pub(crate) async fn post_v1_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let store = message_store(&state)?;
    let pinned = crate::domains::mail::flags::MessageFlags::toggle_pin(&store, &message_id).await?;
    Ok(Json(PinToggleResponse { message_id, pinned }))
}

pub(crate) async fn post_v1_message_important(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<ImportantToggleResponse>, ApiError> {
    let store = message_store(&state)?;
    let important =
        crate::domains::mail::flags::MessageFlags::toggle_important(&store, &message_id).await?;
    Ok(Json(ImportantToggleResponse {
        message_id,
        important,
    }))
}

#[derive(Deserialize)]
pub(crate) struct SnoozeRequest {
    pub(super) until: String,
}

pub(crate) async fn post_v1_message_snooze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<Value>, ApiError> {
    let until: DateTime<Utc> = req
        .until
        .parse()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid datetime"))?;
    let store = message_store(&state)?;
    crate::domains::mail::flags::MessageFlags::snooze(&store, &message_id, until).await?;
    Ok(Json(serde_json::json!({"snoozed": true})))
}

pub(crate) async fn post_v1_message_mute(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let store = message_store(&state)?;
    let muted = crate::domains::mail::flags::MessageFlags::toggle_mute(&store, &message_id).await?;
    Ok(Json(PinToggleResponse {
        message_id,
        pinned: muted,
    }))
}

#[derive(Deserialize)]
pub(crate) struct LabelRequest {
    pub(super) label: String,
}

pub(crate) async fn post_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    crate::domains::mail::flags::MessageFlags::add_label(&store, &message_id, &req.label).await?;
    Ok(Json(serde_json::json!({"labeled": true})))
}

pub(crate) async fn delete_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    crate::domains::mail::flags::MessageFlags::remove_label(&store, &message_id, &req.label)
        .await?;
    Ok(Json(serde_json::json!({"removed": true})))
}

#[derive(Deserialize)]
pub(crate) struct SubscriptionsQuery {
    pub(super) account_id: Option<String>,
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_subscriptions(
    State(state): State<AppState>,
    Query(query): Query<SubscriptionsQuery>,
) -> Result<Json<Vec<crate::domains::mail::subscriptions::SubscriptionSource>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::subscriptions::SubscriptionStore::new(pool);
    let subs = store
        .detect_subscriptions(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;
    Ok(Json(subs))
}

#[derive(Deserialize)]
pub(crate) struct DupQuery {
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_attachment_duplicates(
    State(state): State<AppState>,
    Query(query): Query<DupQuery>,
) -> Result<Json<Vec<crate::domains::mail::attachment_dedup::DuplicateGroup>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::attachment_dedup::AttachmentDedupStore::new(pool);
    let dups = store.find_duplicates(query.limit.unwrap_or(20)).await?;
    Ok(Json(dups))
}

#[derive(Deserialize)]
pub(crate) struct LegalDocQuery {
    pub(super) document_type: Option<String>,
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LegalDocListResponse {
    pub(super) items: Vec<crate::domains::mail::legal::LegalDocument>,
}
