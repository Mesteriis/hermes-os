use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
};

use super::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgContactLink {
    pub id: String,
    pub organization_id: String,
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
pub struct OrgContactLinkStore {
    pool: PgPool,
}

impl OrgContactLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_org(&self, org_id: &str) -> Result<Vec<OrgContactLink>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at FROM organization_contact_links WHERE organization_id=$1 ORDER BY is_primary DESC, role")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgContactLink {
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
    ) -> Result<OrgContactLink, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO organization_contact_links (organization_id, person_id, role, department) VALUES ($1,$2,$3,$4) ON CONFLICT (organization_id, person_id, role) DO UPDATE SET department=EXCLUDED.department, updated_at=now() RETURNING id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at")
            .bind(org_id)
            .bind(person_id)
            .bind(role)
            .bind(dept)
            .fetch_one(&mut *transaction)
            .await?;
        let link = OrgContactLink {
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

        Self::materialize_relationship_in_transaction(&mut transaction, &link).await?;
        transaction.commit().await?;

        Ok(link)
    }

    async fn materialize_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        link: &OrgContactLink,
    ) -> Result<(), OrgCoreError> {
        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: link.person_id.clone(),
            target_entity_kind: RelationshipEntityKind::Organization,
            target_entity_id: link.organization_id.clone(),
            relationship_type: "member_of".to_owned(),
            trust_score: 0.5,
            strength_score: if link.is_primary { 0.85 } else { 0.65 },
            confidence: link.confidence,
            review_state: RelationshipReviewState::UserConfirmed,
            valid_from: link.valid_from,
            valid_to: link.valid_to,
            metadata: json!({
                "compatibility_table": "organization_contact_links",
                "compatibility_record_id": link.id,
                "role": link.role,
                "department": link.department,
                "source": link.source,
                "is_primary": link.is_primary
            }),
        };
        let evidence = NewRelationshipEvidence::new(
            RelationshipEvidenceSourceKind::RawRecord,
            link.id.clone(),
        )
        .excerpt(
            "Persona is linked to organization through compatibility organization contact data.",
        )
        .metadata(json!({
            "compatibility_table": "organization_contact_links",
            "organization_id": link.organization_id,
            "person_id": link.person_id,
            "role": link.role,
            "department": link.department,
            "source": link.source
        }));

        RelationshipStore::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;

        Ok(())
    }

    pub async fn set_primary(&self, org_id: &str, person_id: &str) -> Result<(), OrgCoreError> {
        sqlx::query(
            "UPDATE organization_contact_links SET is_primary=false WHERE organization_id=$1",
        )
        .bind(org_id)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE organization_contact_links SET is_primary=true WHERE organization_id=$1 AND person_id=$2")
            .bind(org_id)
            .bind(person_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
