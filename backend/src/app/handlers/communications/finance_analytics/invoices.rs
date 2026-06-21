use super::super::*;

#[derive(Deserialize)]
pub(crate) struct InvoiceListQuery {
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct InvoiceListResponse {
    pub(super) items: Vec<crate::domains::communications::finance::InvoiceRecord>,
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

pub(crate) async fn get_v1_invoices(
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<InvoiceListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::communications::finance::CommunicationFinanceStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::communications::finance::InvoiceStatus::parse);
    let items = store.list(status).await?;
    Ok(Json(InvoiceListResponse { items }))
}

pub(crate) async fn post_v1_invoice(
    State(state): State<AppState>,
    Json(req): Json<NewInvoiceRequest>,
) -> Result<Json<crate::domains::communications::finance::InvoiceRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::communications::finance::CommunicationFinanceStore::new(pool);
    let invoice = store
        .upsert_invoice(&crate::domains::communications::finance::NewInvoiceRecord {
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
                .and_then(crate::domains::communications::finance::InvoiceStatus::parse)
                .unwrap_or(crate::domains::communications::finance::InvoiceStatus::Received),
            linked_project_id: req.linked_project_id,
            linked_person_id: req.linked_person_id,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(invoice))
}
