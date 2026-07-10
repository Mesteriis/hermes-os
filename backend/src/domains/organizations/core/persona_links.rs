use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::{OrgCoreError, link_entity_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPersonaLink {
    pub id: String,
    pub organization_id: String,
    #[serde(rename = "persona_id", alias = "person_id")]
    pub person_id: String,
    pub role: Option<String>,
    pub department: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPersonaLinkStore {
    pool: PgPool,
}

impl OrgPersonaLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_org(&self, org_id: &str) -> Result<Vec<OrgPersonaLink>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at FROM organization_persona_links WHERE organization_id=$1 ORDER BY is_primary DESC, role")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgPersonaLink {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    person_id: row.try_get("person_id")?,
                    role: row.try_get("role")?,
                    department: row.try_get("department")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    valid_from: row.try_get("valid_from")?,
                    valid_to: row.try_get("valid_to")?,
                    is_primary: row.try_get("is_primary")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn link(
        &self,
        org_id: &str,
        person_id: &str,
        role: Option<&str>,
        dept: Option<&str>,
    ) -> Result<OrgPersonaLink, OrgCoreError> {
        self.link_with_observation(org_id, person_id, role, dept, None, None)
            .await
    }

    pub async fn link_with_observation(
        &self,
        org_id: &str,
        person_id: &str,
        role: Option<&str>,
        dept: Option<&str>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<OrgPersonaLink, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO organization_persona_links (organization_id, person_id, role, department, source) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (organization_id, person_id, role) DO UPDATE SET department=EXCLUDED.department, source=EXCLUDED.source, updated_at=now() RETURNING id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at")
            .bind(org_id)
            .bind(person_id)
            .bind(role)
            .bind(dept)
            .bind(source.unwrap_or("manual"))
            .fetch_one(&mut *transaction)
            .await?;
        let link = OrgPersonaLink {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            person_id: row.try_get("person_id")?,
            role: row.try_get("role")?,
            department: row.try_get("department")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            is_primary: row.try_get("is_primary")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };

        if let Some(observation_id) = observation_id {
            link_entity_in_transaction(
                &mut transaction,
                observation_id,
                "persona_link",
                &link.id,
                json!({
                    "organization_id": org_id,
                    "persona_id": link.person_id,
                    "role": link.role,
                    "department": link.department,
                }),
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(link)
    }

    pub async fn link_email_participant_with_observation(
        &self,
        org_id: &str,
        person_id: &str,
        message_id: &str,
        observation_id: &str,
    ) -> Result<(OrgPersonaLink, bool), OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO organization_persona_links (
                organization_id,
                person_id,
                role,
                source,
                confidence
            )
            VALUES ($1, $2, 'email_participant', 'email_sync', 1.0)
            ON CONFLICT (organization_id, person_id, role)
            DO UPDATE SET
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                updated_at = now()
            RETURNING
                id::text,
                organization_id,
                person_id,
                role,
                department,
                source,
                confidence::float8 AS confidence,
                valid_from,
                valid_to,
                is_primary,
                created_at,
                updated_at,
                (xmax = 0) AS inserted
            "#,
        )
        .bind(org_id)
        .bind(person_id)
        .fetch_one(&mut *transaction)
        .await?;
        let link = OrgPersonaLink {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            person_id: row.try_get("person_id")?,
            role: row.try_get("role")?,
            department: row.try_get("department")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            is_primary: row.try_get("is_primary")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        let inserted: bool = row.try_get("inserted")?;
        transaction.commit().await?;

        Ok((link, inserted))
    }

    pub async fn set_primary(&self, org_id: &str, person_id: &str) -> Result<(), OrgCoreError> {
        sqlx::query(
            "UPDATE organization_persona_links SET is_primary=false WHERE organization_id=$1",
        )
        .bind(org_id)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE organization_persona_links SET is_primary=true WHERE organization_id=$1 AND person_id=$2")
            .bind(org_id)
            .bind(person_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
