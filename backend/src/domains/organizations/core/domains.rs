use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationDomain {
    pub id: String,
    pub organization_id: String,
    pub domain: String,
    pub domain_type: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgDomainStore {
    pool: PgPool,
}

impl OrgDomainStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationDomain>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, domain, domain_type, source, confidence, last_verified_at, created_at FROM organization_domains WHERE organization_id=$1 ORDER BY domain_type, domain")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrganizationDomain {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    domain: row.try_get("domain")?,
                    domain_type: row.try_get("domain_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    last_verified_at: row.try_get("last_verified_at")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        domain: &str,
        domain_type: &str,
        source: &str,
    ) -> Result<OrganizationDomain, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_domains (organization_id, domain, domain_type, source) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, domain, domain_type, source, confidence, last_verified_at, created_at")
            .bind(org_id)
            .bind(domain)
            .bind(domain_type)
            .bind(source)
            .fetch_one(&self.pool)
            .await?;

        Ok(OrganizationDomain {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            domain: row.try_get("domain")?,
            domain_type: row.try_get("domain_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn upsert_email_domain(
        &self,
        org_id: &str,
        domain: &str,
    ) -> Result<bool, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let (_, inserted) =
            Self::upsert_email_domain_in_transaction(&mut transaction, org_id, domain).await?;
        transaction.commit().await?;
        Ok(inserted)
    }

    pub(crate) async fn upsert_email_domain_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        domain: &str,
    ) -> Result<(OrganizationDomain, bool), OrgCoreError> {
        let result = sqlx::query(
            r#"
            INSERT INTO organization_domains (organization_id, domain, domain_type, source)
            SELECT $1, $2, 'email', 'email_sync'
            WHERE NOT EXISTS (
                SELECT 1
                FROM organization_domains
                WHERE organization_id = $1
                  AND domain = $2
                  AND domain_type != 'former'
            )
            "#,
        )
        .bind(org_id)
        .bind(domain)
        .execute(&mut **transaction)
        .await?;
        let inserted = result.rows_affected() > 0;
        let row = sqlx::query(
            r#"
            SELECT id::text, organization_id, domain, domain_type, source, confidence, last_verified_at, created_at
            FROM organization_domains
            WHERE organization_id = $1
              AND domain = $2
              AND domain_type != 'former'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(domain)
        .fetch_one(&mut **transaction)
        .await?;
        Ok((
            OrganizationDomain {
                id: row.try_get("id")?,
                organization_id: row.try_get("organization_id")?,
                domain: row.try_get("domain")?,
                domain_type: row.try_get("domain_type")?,
                source: row.try_get("source")?,
                confidence: row.try_get("confidence")?,
                last_verified_at: row.try_get("last_verified_at")?,
                created_at: row.try_get("created_at")?,
            },
            inserted,
        ))
    }
}
