use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgFinancialInfo {
    pub id: String,
    pub organization_id: String,
    pub bank_name: Option<String>,
    pub iban_masked: Option<String>,
    pub bic: Option<String>,
    pub payment_terms: Option<String>,
    pub currency: Option<String>,
    pub billing_email: Option<String>,
    pub billing_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgFinancialStore {
    pool: PgPool,
}
impl OrgFinancialStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn get(&self, org_id: &str) -> Result<Option<OrgFinancialInfo>, OrgFinanceError> {
        let row = sqlx::query("SELECT id::text, organization_id, bank_name, iban_masked, bic, payment_terms, currency, billing_email, billing_address, created_at, updated_at FROM organization_financial_info WHERE organization_id=$1")
            .bind(org_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(OrgFinancialInfo {
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                bank_name: r.try_get("bank_name")?,
                iban_masked: r.try_get("iban_masked")?,
                bic: r.try_get("bic")?,
                payment_terms: r.try_get("payment_terms")?,
                currency: r.try_get("currency")?,
                billing_email: r.try_get("billing_email")?,
                billing_address: r.try_get("billing_address")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })
        })
        .transpose()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        bank: Option<&str>,
        iban: Option<&str>,
        bic: Option<&str>,
    ) -> Result<OrgFinancialInfo, OrgFinanceError> {
        let row = sqlx::query("INSERT INTO organization_financial_info (organization_id, bank_name, iban_masked, bic) VALUES ($1,$2,$3,$4) ON CONFLICT (organization_id) DO UPDATE SET bank_name=EXCLUDED.bank_name, iban_masked=EXCLUDED.iban_masked, bic=EXCLUDED.bic, updated_at=now() RETURNING id::text, organization_id, bank_name, iban_masked, bic, payment_terms, currency, billing_email, billing_address, created_at, updated_at")
            .bind(org_id).bind(bank).bind(iban).bind(bic).fetch_one(&self.pool).await?;
        Ok(OrgFinancialInfo {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            bank_name: row.try_get("bank_name")?,
            iban_masked: row.try_get("iban_masked")?,
            bic: row.try_get("bic")?,
            payment_terms: row.try_get("payment_terms")?,
            currency: row.try_get("currency")?,
            billing_email: row.try_get("billing_email")?,
            billing_address: row.try_get("billing_address")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgContract {
    pub id: String,
    pub organization_id: String,
    pub contract_type: String,
    pub title: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgContractStore {
    pool: PgPool,
}
impl OrgContractStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgContract>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, contract_type, title, signed_at, expires_at, status, document_reference, notes, created_at, updated_at FROM organization_contracts WHERE organization_id=$1 ORDER BY signed_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgContract {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    contract_type: r.try_get("contract_type")?,
                    title: r.try_get("title")?,
                    signed_at: r.try_get("signed_at")?,
                    expires_at: r.try_get("expires_at")?,
                    status: r.try_get("status")?,
                    document_reference: r.try_get("document_reference")?,
                    notes: r.try_get("notes")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        contract_type: &str,
        title: &str,
    ) -> Result<OrgContract, OrgFinanceError> {
        let row = sqlx::query("INSERT INTO organization_contracts (organization_id, contract_type, title) VALUES ($1,$2,$3) RETURNING id::text, organization_id, contract_type, title, signed_at, expires_at, status, document_reference, notes, created_at, updated_at")
            .bind(org_id).bind(contract_type).bind(title).fetch_one(&self.pool).await?;
        Ok(OrgContract {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            contract_type: row.try_get("contract_type")?,
            title: row.try_get("title")?,
            signed_at: row.try_get("signed_at")?,
            expires_at: row.try_get("expires_at")?,
            status: row.try_get("status")?,
            document_reference: row.try_get("document_reference")?,
            notes: row.try_get("notes")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgCompliance {
    pub id: String,
    pub organization_id: String,
    pub compliance_type: String,
    pub status: String,
    pub document_reference: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgComplianceStore {
    pool: PgPool,
}
impl OrgComplianceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgCompliance>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, compliance_type, status, document_reference, expires_at, notes, created_at, updated_at FROM organization_compliance WHERE organization_id=$1 ORDER BY compliance_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgCompliance {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    compliance_type: r.try_get("compliance_type")?,
                    status: r.try_get("status")?,
                    document_reference: r.try_get("document_reference")?,
                    expires_at: r.try_get("expires_at")?,
                    notes: r.try_get("notes")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgService {
    pub id: String,
    pub organization_id: String,
    pub service_name: String,
    pub description: Option<String>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgServiceStore {
    pool: PgPool,
}
impl OrgServiceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgService>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, service_name, description, status, started_at, created_at, updated_at FROM organization_services WHERE organization_id=$1 ORDER BY service_name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgService {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    service_name: r.try_get("service_name")?,
                    description: r.try_get("description")?,
                    status: r.try_get("status")?,
                    started_at: r.try_get("started_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgProduct {
    pub id: String,
    pub organization_id: String,
    pub product_name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgProductStore {
    pool: PgPool,
}
impl OrgProductStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgProduct>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, product_name, description, status, created_at, updated_at FROM organization_products WHERE organization_id=$1 ORDER BY product_name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgProduct {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    product_name: r.try_get("product_name")?,
                    description: r.try_get("description")?,
                    status: r.try_get("status")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum OrgFinanceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
