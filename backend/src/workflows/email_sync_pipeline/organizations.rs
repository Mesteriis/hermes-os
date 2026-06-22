use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::organizations::api::OrganizationCommandPort;
use crate::domains::organizations::core::OrganizationContactLinkPort;

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
    let inserted = OrganizationContactLinkPort::new(pool.clone())
        .link_email_participant_with_observation(
            organization_id,
            person_id,
            &message.message_id,
            &message.observation_id,
        )
        .await?;
    Ok(inserted)
}
