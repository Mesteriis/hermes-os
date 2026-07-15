use super::api::{Organization, OrganizationError, OrganizationStore};
use super::core::errors::OrgCoreError;
use super::core::persona_links::{OrgPersonaLink, OrgPersonaLinkStore};

#[derive(Clone)]
pub struct OrganizationCommandPort(OrganizationStore);

impl OrganizationCommandPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(OrganizationStore::new(pool))
    }

    pub async fn upsert_review_organization(
        &self,
        organization_id: &str,
        display_name: &str,
        description: Option<&str>,
    ) -> Result<Organization, OrganizationError> {
        self.0
            .upsert_review_organization(organization_id, display_name, description)
            .await
    }

    pub async fn upsert_email_domain_organization_with_observation(
        &self,
        domain: &str,
        observation_id: &str,
    ) -> Result<(Organization, bool), OrganizationError> {
        self.0
            .upsert_email_domain_organization_with_observation(domain, observation_id)
            .await
    }
}

#[derive(Clone)]
pub struct OrganizationPersonaLinkPort(OrgPersonaLinkStore);

impl OrganizationPersonaLinkPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(OrgPersonaLinkStore::new(pool))
    }

    pub async fn link_email_participant_with_observation(
        &self,
        organization_id: &str,
        persona_id: &str,
        message_id: &str,
        observation_id: &str,
    ) -> Result<(OrgPersonaLink, bool), OrgCoreError> {
        self.0
            .link_email_participant_with_observation(
                organization_id,
                persona_id,
                message_id,
                observation_id,
            )
            .await
    }
}
