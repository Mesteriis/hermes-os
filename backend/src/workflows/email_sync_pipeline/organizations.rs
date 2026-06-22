use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::organizations::api::OrganizationCommandPort;
use crate::domains::organizations::core::{OrgContactLink, OrganizationContactLinkPort};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewPort, RelationshipReviewState,
};

use super::errors::EmailSyncPipelineError;
use super::participants::EmailParticipant;

#[derive(Default)]
pub(crate) struct OrganizationProjectionReport {
    pub(crate) upserted_organizations: usize,
    pub(crate) upserted_organization_contact_links: usize,
}

pub(crate) async fn project_email_participant_organization(
    pool: &PgPool,
    person_id: &str,
    message: &ProjectedMessage,
    participant: &EmailParticipant,
) -> Result<OrganizationProjectionReport, EmailSyncPipelineError> {
    let Some(domain) = organization_domain_for_email(&participant.email_address) else {
        return Ok(OrganizationProjectionReport::default());
    };

    let organization_id =
        upsert_email_domain_organization(pool, &domain, &message.observation_id).await?;
    let organization_inserted = organization_id.is_some();
    let organization_id = organization_id.unwrap_or_else(|| organization_id_for_domain(&domain));
    let contact_link_inserted =
        upsert_organization_contact_link(pool, &organization_id, person_id, message, participant)
            .await?;

    Ok(OrganizationProjectionReport {
        upserted_organizations: usize::from(organization_inserted),
        upserted_organization_contact_links: usize::from(contact_link_inserted),
    })
}

fn organization_domain_for_email(email_address: &str) -> Option<String> {
    let domain = email_address.split('@').nth(1)?.trim().to_ascii_lowercase();
    if domain.is_empty() || is_public_mail_domain(&domain) {
        None
    } else {
        Some(domain)
    }
}

fn is_public_mail_domain(domain: &str) -> bool {
    matches!(
        domain,
        "gmail.com"
            | "googlemail.com"
            | "icloud.com"
            | "me.com"
            | "mac.com"
            | "outlook.com"
            | "hotmail.com"
            | "live.com"
            | "yahoo.com"
            | "proton.me"
            | "protonmail.com"
            | "mail.ru"
            | "yandex.ru"
    )
}

fn organization_id_for_domain(domain: &str) -> String {
    format!("org:v1:email-domain:{}:{domain}", domain.len())
}

async fn upsert_email_domain_organization(
    pool: &PgPool,
    domain: &str,
    observation_id: &str,
) -> Result<Option<String>, EmailSyncPipelineError> {
    let (_, inserted) = OrganizationCommandPort::new(pool.clone())
        .upsert_email_domain_organization_with_observation(domain, observation_id)
        .await?;
    Ok(inserted.then(|| organization_id_for_domain(domain)))
}

async fn upsert_organization_contact_link(
    pool: &PgPool,
    organization_id: &str,
    person_id: &str,
    message: &ProjectedMessage,
    _participant: &EmailParticipant,
) -> Result<bool, EmailSyncPipelineError> {
    let (link, inserted) = OrganizationContactLinkPort::new(pool.clone())
        .link_email_participant_with_observation(
            organization_id,
            person_id,
            &message.message_id,
            &message.observation_id,
        )
        .await?;
    materialize_email_participant_member_relationship(
        pool,
        &link,
        &message.message_id,
        &message.observation_id,
    )
    .await?;
    Ok(inserted)
}

async fn materialize_email_participant_member_relationship(
    pool: &PgPool,
    link: &OrgContactLink,
    message_id: &str,
    observation_id: &str,
) -> Result<(), EmailSyncPipelineError> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: link.person_id.clone(),
        target_entity_kind: RelationshipEntityKind::Organization,
        target_entity_id: link.organization_id.clone(),
        relationship_type: "member_of".to_owned(),
        trust_score: 0.5,
        strength_score: 0.5,
        confidence: link.confidence,
        review_state: RelationshipReviewState::SystemAccepted,
        valid_from: link.valid_from,
        valid_to: link.valid_to,
        metadata: serde_json::json!({
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
        source_id: message_id.to_owned(),
        observation_id: Some(observation_id.to_owned()),
        excerpt: Some(
            "Persona is linked to organization through compatibility organization contact data."
                .to_owned(),
        ),
        metadata: serde_json::json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "person_id": link.person_id,
        }),
    };
    let _ = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}
