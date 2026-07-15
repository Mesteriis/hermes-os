use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegalDocument {
    pub document_id: String,
    pub message_id: Option<String>,
    pub document_type: LegalDocType,
    pub title: String,
    pub parties: Vec<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub status: LegalDocStatus,
    pub linked_project_id: Option<String>,
    pub risks: Vec<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalDocType {
    Contract,
    Nda,
    Msa,
    Dpa,
    Agreement,
    LegalNotice,
    Claim,
    CourtDocument,
    TaxNotice,
    GovernmentDoc,
    Other,
}

impl LegalDocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LegalDocType::Contract => "contract",
            LegalDocType::Nda => "nda",
            LegalDocType::Msa => "msa",
            LegalDocType::Dpa => "dpa",
            LegalDocType::Agreement => "agreement",
            LegalDocType::LegalNotice => "legal_notice",
            LegalDocType::Claim => "claim",
            LegalDocType::CourtDocument => "court_document",
            LegalDocType::TaxNotice => "tax_notice",
            LegalDocType::GovernmentDoc => "government_doc",
            LegalDocType::Other => "other",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "contract" => Some(LegalDocType::Contract),
            "nda" => Some(LegalDocType::Nda),
            "msa" => Some(LegalDocType::Msa),
            "dpa" => Some(LegalDocType::Dpa),
            "agreement" => Some(LegalDocType::Agreement),
            "legal_notice" => Some(LegalDocType::LegalNotice),
            "claim" => Some(LegalDocType::Claim),
            "court_document" => Some(LegalDocType::CourtDocument),
            "tax_notice" => Some(LegalDocType::TaxNotice),
            "government_doc" => Some(LegalDocType::GovernmentDoc),
            "other" => Some(LegalDocType::Other),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalDocStatus {
    Active,
    Expired,
    PendingReview,
    Signed,
    Terminated,
    Draft,
}

impl LegalDocStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LegalDocStatus::Active => "active",
            LegalDocStatus::Expired => "expired",
            LegalDocStatus::PendingReview => "pending_review",
            LegalDocStatus::Signed => "signed",
            LegalDocStatus::Terminated => "terminated",
            LegalDocStatus::Draft => "draft",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "active" => Some(LegalDocStatus::Active),
            "expired" => Some(LegalDocStatus::Expired),
            "pending_review" => Some(LegalDocStatus::PendingReview),
            "signed" => Some(LegalDocStatus::Signed),
            "terminated" => Some(LegalDocStatus::Terminated),
            "draft" => Some(LegalDocStatus::Draft),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct LegalDocumentStore {
    pool: PgPool,
}

impl LegalDocumentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        doc: &NewLegalDocument,
    ) -> Result<LegalDocument, LegalDocumentError> {
        doc.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO communication_legal_documents (document_id, message_id, document_type, title, parties, effective_date, expiry_date, amount, currency, status, linked_project_id, risks, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (document_id) DO UPDATE SET
                message_id = EXCLUDED.message_id, document_type = EXCLUDED.document_type, title = EXCLUDED.title,
                parties = EXCLUDED.parties, effective_date = EXCLUDED.effective_date, expiry_date = EXCLUDED.expiry_date,
                amount = EXCLUDED.amount, currency = EXCLUDED.currency, status = EXCLUDED.status,
                linked_project_id = EXCLUDED.linked_project_id, risks = EXCLUDED.risks,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING document_id, message_id, document_type, title, parties, effective_date, expiry_date, amount, currency, status, linked_project_id, risks, metadata, created_at, updated_at"#,
        )
        .bind(&doc.document_id).bind(doc.message_id.as_deref()).bind(doc.document_type.as_str())
        .bind(&doc.title).bind(serde_json::to_value(&doc.parties).unwrap_or_default())
        .bind(doc.effective_date).bind(doc.expiry_date).bind(doc.amount).bind(doc.currency.as_deref())
        .bind(doc.status.as_str()).bind(doc.linked_project_id.as_deref())
        .bind(serde_json::to_value(&doc.risks).unwrap_or_default()).bind(&doc.metadata)
        .fetch_one(&self.pool).await?;
        row_to_legal_doc(row)
    }

    pub async fn list(
        &self,
        doc_type: Option<LegalDocType>,
        status: Option<LegalDocStatus>,
    ) -> Result<Vec<LegalDocument>, LegalDocumentError> {
        let dt = doc_type.map(|t| t.as_str().to_owned());
        let st = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT document_id, message_id, document_type, title, parties, effective_date, expiry_date, amount, currency, status, linked_project_id, risks, metadata, created_at, updated_at
            FROM communication_legal_documents WHERE ($1::text IS NULL OR document_type = $1) AND ($2::text IS NULL OR status = $2) ORDER BY COALESCE(effective_date, created_at) DESC"#,
        ).bind(dt.as_deref()).bind(st.as_deref()).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_legal_doc).collect()
    }
}

fn row_to_legal_doc(row: PgRow) -> Result<LegalDocument, LegalDocumentError> {
    let dt_str: String = row.try_get("document_type")?;
    let st_str: String = row.try_get("status")?;
    Ok(LegalDocument {
        document_id: row.try_get("document_id")?,
        message_id: row.try_get("message_id")?,
        document_type: LegalDocType::parse(&dt_str).unwrap_or(LegalDocType::Other),
        title: row.try_get("title")?,
        parties: serde_json::from_value(row.try_get("parties")?).unwrap_or_default(),
        effective_date: row.try_get("effective_date")?,
        expiry_date: row.try_get("expiry_date")?,
        amount: row.try_get("amount")?,
        currency: row.try_get("currency")?,
        status: LegalDocStatus::parse(&st_str).unwrap_or(LegalDocStatus::Draft),
        linked_project_id: row.try_get("linked_project_id")?,
        risks: serde_json::from_value(row.try_get("risks")?).unwrap_or_default(),
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewLegalDocument {
    pub document_id: String,
    pub message_id: Option<String>,
    pub document_type: LegalDocType,
    pub title: String,
    pub parties: Vec<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub status: LegalDocStatus,
    pub linked_project_id: Option<String>,
    pub risks: Vec<String>,
    pub metadata: Value,
}

impl NewLegalDocument {
    fn validate(&self) -> Result<(), LegalDocumentError> {
        if self.document_id.trim().is_empty() {
            return Err(LegalDocumentError::Invalid("document_id empty"));
        }
        if self.title.trim().is_empty() {
            return Err(LegalDocumentError::Invalid("title empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum LegalDocumentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid document: {0}")]
    Invalid(&'static str),
}
