use super::*;

pub(crate) async fn get_v1_invoices(
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<InvoiceListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::finance::EmailFinanceStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::mail::finance::InvoiceStatus::parse);
    let items = store.list(status).await?;
    Ok(Json(InvoiceListResponse { items }))
}

pub(crate) async fn post_v1_invoice(
    State(state): State<AppState>,
    Json(req): Json<NewInvoiceRequest>,
) -> Result<Json<crate::domains::mail::finance::InvoiceRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::finance::EmailFinanceStore::new(pool);
    let invoice = store
        .upsert_invoice(&crate::domains::mail::finance::NewInvoiceRecord {
            invoice_id: req.invoice_id,
            message_id: req.message_id,
            amount: req.amount,
            currency: req.currency,
            invoice_number: req.invoice_number,
            issue_date: req.issue_date,
            due_date: req.due_date,
            counterparty: req.counterparty,
            tax_id: req.tax_id,
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::mail::finance::InvoiceStatus::parse)
                .unwrap_or(crate::domains::mail::finance::InvoiceStatus::Received),
            linked_project_id: req.linked_project_id,
            linked_person_id: req.linked_person_id,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(invoice))
}

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

#[derive(Serialize)]
pub(crate) struct MessageExplainResponse {
    pub(super) reasons: Vec<String>,
}

pub(crate) async fn get_v1_message_explain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageExplainResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let ctx = crate::domains::mail::explain::explain_importance(&message);
    Ok(Json(MessageExplainResponse {
        reasons: ctx.reasons,
    }))
}

#[derive(Serialize)]
pub(crate) struct SmartCcResponse {
    pub(super) suggestions: Vec<String>,
}

pub(crate) async fn get_v1_message_smart_cc(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<SmartCcResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let suggestions = crate::domains::mail::explain::smart_cc_suggestions(&message);
    Ok(Json(SmartCcResponse { suggestions }))
}

#[derive(Serialize)]
pub(crate) struct PinToggleResponse {
    pub(super) message_id: String,
    pub(super) pinned: bool,
}

#[derive(Serialize)]
pub(crate) struct ImportantToggleResponse {
    pub(super) message_id: String,
    pub(super) important: bool,
}
