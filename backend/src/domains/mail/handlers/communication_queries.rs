use super::*;

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

#[derive(Deserialize)]
pub(crate) struct EmailSearchQuery {
    pub(super) q: String,
    pub(super) limit: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct EmailSearchResponse {
    pub(super) results: Vec<SearchResultResponse>,
}

#[derive(Serialize)]
pub(crate) struct SearchResultResponse {
    pub(super) object_id: String,
    pub(super) object_kind: String,
    pub(super) title: String,
}

pub(crate) async fn get_v1_email_search(
    State(state): State<AppState>,
    Query(query): Query<EmailSearchQuery>,
) -> Result<Json<EmailSearchResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "search query is required",
        ));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = MessageProjectionStore::new(pool.clone());

    // Index recent messages into Tantivy for search
    let search_path: Option<String> = std::env::var("HERMES_SEARCH_INDEX_PATH").ok();
    if let Some(path) = search_path {
        let index =
            crate::engines::search::SearchIndex::open_or_create(std::path::Path::new(&path))?;
        let _ = crate::domains::mail::search::index_messages(&index, &store, 100).await;
        let results = crate::domains::mail::search::search_emails(
            &index,
            &query.q,
            query.limit.unwrap_or(20),
        )?;
        let items: Vec<SearchResultResponse> = results
            .into_iter()
            .map(|r| SearchResultResponse {
                object_id: r.object_id,
                object_kind: r.object_kind,
                title: r.title,
            })
            .collect();
        return Ok(Json(EmailSearchResponse { results: items }));
    }

    Ok(Json(EmailSearchResponse { results: vec![] }))
}

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    pub(super) items: Vec<crate::domains::mail::personas::EmailPersona>,
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaRequest {
    pub(super) persona_id: String,
    pub(super) name: String,
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) signature: Option<String>,
    pub(super) default_language: Option<String>,
    pub(super) default_tone: Option<String>,
    pub(super) is_default: Option<bool>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn get_v1_personas(
    State(state): State<AppState>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::personas::EmailPersonaStore::new(pool);
    let items = store.list().await?;
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn post_v1_persona(
    State(state): State<AppState>,
    Json(request): Json<NewPersonaRequest>,
) -> Result<Json<crate::domains::mail::personas::EmailPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::personas::EmailPersonaStore::new(pool);
    let persona = store
        .upsert(&crate::domains::mail::personas::NewEmailPersona {
            persona_id: request.persona_id,
            name: request.name,
            account_id: request.account_id,
            display_name: request.display_name,
            signature: request.signature.unwrap_or_default(),
            default_language: request.default_language,
            default_tone: request.default_tone,
            is_default: request.is_default.unwrap_or(false),
            metadata: request.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(persona))
}

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

#[derive(Deserialize)]
pub(crate) struct InvoiceListQuery {
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct InvoiceListResponse {
    pub(super) items: Vec<crate::domains::mail::finance::InvoiceRecord>,
}

#[derive(Deserialize)]
pub(crate) struct NewInvoiceRequest {
    pub(super) invoice_id: String,
    pub(super) message_id: Option<String>,
    pub(super) amount: Option<f64>,
    pub(super) currency: Option<String>,
    pub(super) invoice_number: Option<String>,
    pub(super) issue_date: Option<DateTime<Utc>>,
    pub(super) due_date: Option<DateTime<Utc>>,
    pub(super) counterparty: Option<String>,
    pub(super) tax_id: Option<String>,
    pub(super) status: Option<String>,
    pub(super) linked_project_id: Option<String>,
    pub(super) linked_person_id: Option<String>,
    pub(super) metadata: Option<Value>,
}
