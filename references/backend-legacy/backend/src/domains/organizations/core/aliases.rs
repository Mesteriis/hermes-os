use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::errors::OrgCoreError;
use super::evidence::link_entity_in_transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationAlias {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub alias_type: String,
    pub source: String,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgAliasStore {
    pool: PgPool,
}

impl OrgAliasStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationAlias>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, alias_type, source, confidence::float8 AS confidence, valid_from, valid_to, created_at FROM organization_aliases WHERE organization_id=$1 ORDER BY name")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrganizationAlias {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    alias_type: row.try_get("alias_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    valid_from: row.try_get("valid_from")?,
                    valid_to: row.try_get("valid_to")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let alias =
            Self::add_in_transaction(&mut transaction, org_id, name, alias_type, source).await?;
        transaction.commit().await?;
        Ok(alias)
    }

    pub async fn add_with_observation(
        &self,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let alias =
            Self::add_in_transaction(&mut transaction, org_id, name, alias_type, source).await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "alias",
            &alias.id,
            json!({
                "organization_id": org_id,
                "alias_type": alias.alias_type,
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(alias)
    }

    pub(crate) async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let alias_type = normalize_alias_type(alias_type);
        let row = sqlx::query("INSERT INTO organization_aliases (organization_id, name, alias_type, source) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, name, alias_type, source, confidence::float8 AS confidence, valid_from, valid_to, created_at")
            .bind(org_id)
            .bind(name)
            .bind(alias_type)
            .bind(source)
            .fetch_one(&mut **transaction)
            .await?;

        Ok(OrganizationAlias {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            alias_type: row.try_get("alias_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn normalize_alias_type(alias_type: &str) -> &str {
    match alias_type {
        "former_name" => "former",
        other => other,
    }
}
