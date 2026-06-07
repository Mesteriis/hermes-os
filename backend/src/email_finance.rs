use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceRecord {
    pub invoice_id: String,
    pub message_id: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub invoice_number: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub counterparty: Option<String>,
    pub tax_id: Option<String>,
    pub status: InvoiceStatus,
    pub linked_project_id: Option<String>,
    pub linked_contact_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Received,
    Recognized,
    NeedsReview,
    Approved,
    Paid,
    Closed,
    Rejected,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoiceStatus::Received => "received",
            InvoiceStatus::Recognized => "recognized",
            InvoiceStatus::NeedsReview => "needs_review",
            InvoiceStatus::Approved => "approved",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Closed => "closed",
            InvoiceStatus::Rejected => "rejected",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "received" => Some(InvoiceStatus::Received),
            "recognized" => Some(InvoiceStatus::Recognized),
            "needs_review" => Some(InvoiceStatus::NeedsReview),
            "approved" => Some(InvoiceStatus::Approved),
            "paid" => Some(InvoiceStatus::Paid),
            "closed" => Some(InvoiceStatus::Closed),
            "rejected" => Some(InvoiceStatus::Rejected),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct EmailFinanceStore {
    pool: PgPool,
}

impl EmailFinanceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_invoice(
        &self,
        invoice: &NewInvoiceRecord,
    ) -> Result<InvoiceRecord, EmailFinanceError> {
        invoice.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO email_invoices (invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date, counterparty, tax_id, status, linked_project_id, linked_contact_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (invoice_id) DO UPDATE SET
                message_id = EXCLUDED.message_id, amount = EXCLUDED.amount, currency = EXCLUDED.currency,
                invoice_number = EXCLUDED.invoice_number, issue_date = EXCLUDED.issue_date,
                due_date = EXCLUDED.due_date, counterparty = EXCLUDED.counterparty,
                tax_id = EXCLUDED.tax_id, status = EXCLUDED.status,
                linked_project_id = EXCLUDED.linked_project_id, linked_contact_id = EXCLUDED.linked_contact_id,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date, counterparty, tax_id, status, linked_project_id, linked_contact_id, metadata, created_at, updated_at"#,
        )
        .bind(&invoice.invoice_id).bind(invoice.message_id.as_deref()).bind(invoice.amount)
        .bind(invoice.currency.as_deref()).bind(invoice.invoice_number.as_deref())
        .bind(invoice.issue_date).bind(invoice.due_date).bind(invoice.counterparty.as_deref())
        .bind(invoice.tax_id.as_deref()).bind(invoice.status.as_str())
        .bind(invoice.linked_project_id.as_deref()).bind(invoice.linked_contact_id.as_deref())
        .bind(&invoice.metadata).fetch_one(&self.pool).await?;
        row_to_invoice(row)
    }

    pub async fn list(
        &self,
        status: Option<InvoiceStatus>,
    ) -> Result<Vec<InvoiceRecord>, EmailFinanceError> {
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date, counterparty, tax_id, status, linked_project_id, linked_contact_id, metadata, created_at, updated_at
            FROM email_invoices WHERE ($1::text IS NULL OR status = $1) ORDER BY COALESCE(due_date, created_at) DESC"#,
        ).bind(status_str.as_deref()).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_invoice).collect()
    }
}

fn row_to_invoice(row: PgRow) -> Result<InvoiceRecord, EmailFinanceError> {
    let status_str: String = row.try_get("status")?;
    Ok(InvoiceRecord {
        invoice_id: row.try_get("invoice_id")?,
        message_id: row.try_get("message_id")?,
        amount: row.try_get("amount")?,
        currency: row.try_get("currency")?,
        invoice_number: row.try_get("invoice_number")?,
        issue_date: row.try_get("issue_date")?,
        due_date: row.try_get("due_date")?,
        counterparty: row.try_get("counterparty")?,
        tax_id: row.try_get("tax_id")?,
        status: InvoiceStatus::parse(&status_str).unwrap_or(InvoiceStatus::Received),
        linked_project_id: row.try_get("linked_project_id")?,
        linked_contact_id: row.try_get("linked_contact_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewInvoiceRecord {
    pub invoice_id: String,
    pub message_id: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub invoice_number: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub counterparty: Option<String>,
    pub tax_id: Option<String>,
    pub status: InvoiceStatus,
    pub linked_project_id: Option<String>,
    pub linked_contact_id: Option<String>,
    pub metadata: Value,
}

impl NewInvoiceRecord {
    fn validate(&self) -> Result<(), EmailFinanceError> {
        if self.invoice_id.trim().is_empty() {
            return Err(EmailFinanceError::Invalid("invoice_id empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum EmailFinanceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid invoice: {0}")]
    Invalid(&'static str),
}
