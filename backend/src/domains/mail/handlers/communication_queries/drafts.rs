use super::super::*;

#[derive(Deserialize)]
pub(crate) struct DraftListQuery {
    pub(super) account_id: Option<String>,
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct DraftListResponse {
    pub(super) items: Vec<crate::domains::mail::drafts::EmailDraft>,
}

#[derive(Deserialize)]
pub(crate) struct NewDraftRequest {
    pub(super) draft_id: String,
    pub(super) account_id: String,
    pub(super) persona_id: Option<String>,
    pub(super) to_recipients: Vec<String>,
    pub(super) cc_recipients: Option<Vec<String>>,
    pub(super) bcc_recipients: Option<Vec<String>>,
    pub(super) subject: String,
    pub(super) body_text: String,
    pub(super) body_html: Option<String>,
    pub(super) in_reply_to: Option<String>,
    pub(super) references: Option<Vec<String>>,
    pub(super) status: Option<String>,
    pub(super) scheduled_send_at: Option<DateTime<Utc>>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn get_v1_drafts(
    State(state): State<AppState>,
    Query(query): Query<DraftListQuery>,
) -> Result<Json<DraftListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::mail::drafts::DraftStatus::parse);
    let items = store.list(query.account_id.as_deref(), status).await?;
    Ok(Json(DraftListResponse { items }))
}

pub(crate) async fn post_v1_draft(
    State(state): State<AppState>,
    Json(req): Json<NewDraftRequest>,
) -> Result<Json<crate::domains::mail::drafts::EmailDraft>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    let draft = store
        .upsert(&crate::domains::mail::drafts::NewEmailDraft {
            draft_id: req.draft_id,
            account_id: req.account_id,
            persona_id: req.persona_id,
            to_recipients: req.to_recipients,
            cc_recipients: req.cc_recipients.unwrap_or_default(),
            bcc_recipients: req.bcc_recipients.unwrap_or_default(),
            subject: req.subject,
            body_text: req.body_text,
            body_html: req.body_html,
            in_reply_to: req.in_reply_to,
            references: req.references.unwrap_or_default(),
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::mail::drafts::DraftStatus::parse)
                .unwrap_or(crate::domains::mail::drafts::DraftStatus::Draft),
            scheduled_send_at: req.scheduled_send_at,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(draft))
}

pub(crate) async fn get_v1_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Result<Json<crate::domains::mail::drafts::EmailDraft>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    store
        .get(&draft_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn delete_v1_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    let deleted = store.delete(&draft_id).await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}
