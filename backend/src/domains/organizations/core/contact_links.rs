use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
};

use super::{OrgCoreError, link_entity_in_transaction};

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
    ) -> Result<OrgContactLink, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO organization_contact_links (organization_id, person_id, role, department, source) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (organization_id, person_id, role) DO UPDATE SET department=EXCLUDED.department, source=EXCLUDED.source, updated_at=now() RETURNING id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at")
            .bind(org_id)
            .bind(person_id)
            .bind(role)
            .bind(dept)
            .bind(source.unwrap_or("manual"))
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

        if let Some(observation_id) = observation_id {
            link_entity_in_transaction(
                &mut transaction,
                observation_id,
                "contact_link",
                &link.id,
                json!({
                    "organization_id": org_id,
                    "person_id": link.person_id,
                    "role": link.role,
                    "department": link.department,
                }),
            )
            .await?;
        }
        materialize_member_of_relationship_in_transaction(
            &mut transaction,
            &link,
            RelationshipReviewState::UserConfirmed,
            &link.id,
            observation_id,
        )
        .await?;
        transaction.commit().await?;

        Ok(link)
    }

    pub async fn link_email_participant_with_observation(
        &self,
        org_id: &str,
        person_id: &str,
        message_id: &str,
        observation_id: &str,
    ) -> Result<bool, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO organization_contact_links (
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
        let inserted: bool = row.try_get("inserted")?;
        materialize_member_of_relationship_in_transaction(
            &mut transaction,
            &link,
            RelationshipReviewState::SystemAccepted,
            message_id,
            Some(observation_id),
        )
        .await?;
        transaction.commit().await?;

        Ok(inserted)
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

async fn materialize_member_of_relationship_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    link: &OrgContactLink,
    review_state: RelationshipReviewState,
    communication_source_id: &str,
    observation_id: Option<&str>,
) -> Result<(), OrgCoreError> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: link.person_id.clone(),
        target_entity_kind: RelationshipEntityKind::Organization,
        target_entity_id: link.organization_id.clone(),
        relationship_type: "member_of".to_owned(),
        trust_score: 0.5,
        strength_score: 0.5,
        confidence: link.confidence,
        review_state,
        valid_from: link.valid_from,
        valid_to: link.valid_to,
        metadata: json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "person_id": link.person_id,
            "role": link.role,
            "department": link.department,
            "source": link.source,
        }),
    };
    let evidence = NewRelationshipEvidence {
        source_kind: RelationshipEvidenceSourceKind::Communication,
        source_id: communication_source_id.to_owned(),
        observation_id: observation_id.map(str::to_owned),
        excerpt: Some(
            "Persona is linked to organization through compatibility organization contact data."
                .to_owned(),
        ),
        metadata: json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "person_id": link.person_id,
        }),
    };
    let _ = RelationshipStore::upsert_with_evidence_in_transaction(
        transaction,
        &relationship,
        &[evidence],
    )
    .await?;
    Ok(())
}
