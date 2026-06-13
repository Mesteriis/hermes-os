use super::*;

pub(crate) async fn get_v1_legal_docs(
    State(state): State<AppState>,
    Query(query): Query<LegalDocQuery>,
) -> Result<Json<LegalDocListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::legal::LegalDocumentStore::new(pool);
    let dt = query
        .document_type
        .as_deref()
        .and_then(crate::domains::mail::legal::LegalDocType::parse);
    let st = query
        .status
        .as_deref()
        .and_then(crate::domains::mail::legal::LegalDocStatus::parse);
    let items = store.list(dt, st).await?;
    Ok(Json(LegalDocListResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewLegalDocRequest {
    pub(super) document_id: String,
    pub(super) message_id: Option<String>,
    pub(super) document_type: String,
    pub(super) title: String,
    pub(super) parties: Option<Vec<String>>,
    pub(super) effective_date: Option<DateTime<Utc>>,
    pub(super) expiry_date: Option<DateTime<Utc>>,
    pub(super) amount: Option<f64>,
    pub(super) currency: Option<String>,
    pub(super) status: Option<String>,
    pub(super) linked_project_id: Option<String>,
    pub(super) risks: Option<Vec<String>>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn post_v1_legal_doc(
    State(state): State<AppState>,
    Json(req): Json<NewLegalDocRequest>,
) -> Result<Json<crate::domains::mail::legal::LegalDocument>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::legal::LegalDocumentStore::new(pool);
    let doc = store
        .upsert(&crate::domains::mail::legal::NewLegalDocument {
            document_id: req.document_id,
            message_id: req.message_id,
            document_type: crate::domains::mail::legal::LegalDocType::parse(&req.document_type)
                .unwrap_or(crate::domains::mail::legal::LegalDocType::Other),
            title: req.title,
            parties: req.parties.unwrap_or_default(),
            effective_date: req.effective_date,
            expiry_date: req.expiry_date,
            amount: req.amount,
            currency: req.currency,
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::mail::legal::LegalDocStatus::parse)
                .unwrap_or(crate::domains::mail::legal::LegalDocStatus::Draft),
            linked_project_id: req.linked_project_id,
            risks: req.risks.unwrap_or_default(),
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(doc))
}

#[derive(Serialize)]
pub(crate) struct ExportResponse {
    pub(super) content_type: String,
    pub(super) content: String,
    pub(super) filename: String,
}

#[derive(Deserialize)]
pub(crate) struct MessageExportQuery {
    pub(super) format: Option<String>,
}

pub(crate) async fn get_v1_message_export(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<MessageExportQuery>,
) -> Result<Json<ExportResponse>, ApiError> {
    let msg_store = message_store(&state)?;
    let att_store = mail_storage_store(&state)?;
    let format = match query.format.as_deref().unwrap_or("markdown") {
        "eml" => crate::domains::mail::export::ExportFormat::Eml,
        "json" => crate::domains::mail::export::ExportFormat::Json,
        _ => crate::domains::mail::export::ExportFormat::Markdown,
    };
    let export =
        crate::domains::mail::export::export_message(&msg_store, &att_store, &message_id, format)
            .await?;
    Ok(Json(ExportResponse {
        content_type: export.format.content_type().to_owned(),
        content: export.content,
        filename: format!(
            "message_{}.{}",
            &message_id[..8.min(message_id.len())],
            export.format.extension()
        ),
    }))
}

#[derive(Deserialize)]
pub(crate) struct SendRequest {
    pub(super) account_id: String,
    pub(super) to: Vec<String>,
    pub(super) cc: Option<Vec<String>>,
    pub(super) bcc: Option<Vec<String>>,
    pub(super) subject: String,
    pub(super) body_text: String,
    pub(super) body_html: Option<String>,
    pub(super) in_reply_to: Option<String>,
    pub(super) references: Option<Vec<String>>,
    pub(super) confirmed_provider_write: Option<bool>,
}

#[derive(Serialize)]
pub(crate) struct SendResponse {
    pub(super) message_id: String,
    pub(super) accepted: Vec<String>,
    pub(super) accepted_recipients: Vec<String>,
    pub(super) transport: String,
    pub(super) status: String,
    pub(super) failure_reason: Option<String>,
}
