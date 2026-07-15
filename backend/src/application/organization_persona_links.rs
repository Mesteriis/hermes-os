use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::organizations::core::persona_links::OrgPersonaLink;
use crate::domains::organizations::service::{
    OrganizationCommandService, OrganizationCommandServiceError,
};
use crate::domains::relationships::models::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState,
};

use super::relationship_graph::{RelationshipGraphCoordinator, RelationshipGraphCoordinatorError};

#[derive(Clone)]
pub struct OrganizationPersonaLinkApplicationService {
    pool: PgPool,
}

impl OrganizationPersonaLinkApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn link_persona_manual(
        &self,
        organization_id: &str,
        persona_id: &str,
        role: Option<&str>,
        department: Option<&str>,
        requested_source: &str,
    ) -> Result<OrgPersonaLink, OrganizationPersonaLinkApplicationError> {
        let link = OrganizationCommandService::new(self.pool.clone())
            .link_persona_manual(
                organization_id,
                persona_id,
                role,
                department,
                requested_source,
            )
            .await?;

        materialize_member_of_relationship(
            &self.pool,
            &link,
            RelationshipReviewState::UserConfirmed,
            manual_persona_link_evidence(&link),
        )
        .await?;

        Ok(link)
    }
}

fn manual_persona_link_evidence(link: &OrgPersonaLink) -> NewRelationshipEvidence {
    if let Some(observation_id) = link.source.strip_prefix("observation:") {
        return NewRelationshipEvidence::observation(observation_id.to_owned())
            .excerpt(relationship_excerpt())
            .metadata(relationship_evidence_metadata(link));
    }

    NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Organization,
        link.organization_id.clone(),
    )
    .excerpt(relationship_excerpt())
    .metadata(relationship_evidence_metadata(link))
}

async fn materialize_member_of_relationship(
    pool: &PgPool,
    link: &OrgPersonaLink,
    review_state: RelationshipReviewState,
    evidence: NewRelationshipEvidence,
) -> Result<(), RelationshipGraphCoordinatorError> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: link.persona_id.clone(),
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
            "compatibility_table": "organization_persona_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "persona_id": link.persona_id,
            "role": link.role,
            "department": link.department,
            "source": link.source,
        }),
    };
    let _ = RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}

fn relationship_excerpt() -> String {
    "Persona is linked to organization through organization-persona compatibility data.".to_owned()
}

fn relationship_evidence_metadata(link: &OrgPersonaLink) -> serde_json::Value {
    json!({
        "compatibility_table": "organization_persona_links",
        "compatibility_record_id": link.id,
        "organization_id": link.organization_id,
        "persona_id": link.persona_id,
    })
}

#[derive(Debug, Error)]
pub enum OrganizationPersonaLinkApplicationError {
    #[error(transparent)]
    Organization(#[from] OrganizationCommandServiceError),

    #[error(transparent)]
    RelationshipGraph(#[from] RelationshipGraphCoordinatorError),
}
