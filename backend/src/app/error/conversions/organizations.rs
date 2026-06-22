use super::super::types::ApiError;
use crate::application::OrganizationContactLinkApplicationError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::service::OrganizationCommandServiceError;

impl From<crate::domains::organizations::core::OrgCoreError> for ApiError {
    fn from(error: crate::domains::organizations::core::OrgCoreError) -> Self {
        tracing::error!(error = %error, "org core operation failed");
        ApiError::InvalidCommunicationQuery("org core operation failed")
    }
}

impl From<crate::domains::organizations::memory::OrgMemoryError> for ApiError {
    fn from(error: crate::domains::organizations::memory::OrgMemoryError) -> Self {
        tracing::error!(error = %error, "org memory operation failed");
        ApiError::InvalidCommunicationQuery("org memory operation failed")
    }
}

impl From<crate::domains::organizations::workflows::OrgWorkflowError> for ApiError {
    fn from(error: crate::domains::organizations::workflows::OrgWorkflowError) -> Self {
        tracing::error!(error = %error, "org workflow operation failed");
        ApiError::InvalidCommunicationQuery("org workflow operation failed")
    }
}

impl From<crate::domains::organizations::finance::OrgFinanceError> for ApiError {
    fn from(error: crate::domains::organizations::finance::OrgFinanceError) -> Self {
        tracing::error!(error = %error, "org finance operation failed");
        ApiError::InvalidCommunicationQuery("org finance operation failed")
    }
}

impl From<crate::domains::organizations::enrichment::OrgEnrichmentError> for ApiError {
    fn from(error: crate::domains::organizations::enrichment::OrgEnrichmentError) -> Self {
        tracing::error!(error = %error, "org enrichment operation failed");
        ApiError::InvalidCommunicationQuery("org enrichment operation failed")
    }
}

impl From<crate::domains::organizations::health::OrgHealthError> for ApiError {
    fn from(error: crate::domains::organizations::health::OrgHealthError) -> Self {
        tracing::error!(error = %error, "org health operation failed");
        ApiError::InvalidCommunicationQuery("org health operation failed")
    }
}

impl From<crate::domains::organizations::investigator::InvestigatorError> for ApiError {
    fn from(error: crate::domains::organizations::investigator::InvestigatorError) -> Self {
        match error {
            crate::domains::organizations::investigator::InvestigatorError::NotFound => {
                ApiError::NotFound
            }
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<OrganizationError> for ApiError {
    fn from(error: OrganizationError) -> Self {
        match error {
            OrganizationError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "organization operation failed");
                ApiError::InvalidCommunicationQuery("organization operation failed")
            }
        }
    }
}

impl From<OrganizationCommandServiceError> for ApiError {
    fn from(error: OrganizationCommandServiceError) -> Self {
        match error {
            OrganizationCommandServiceError::Observation(source) => {
                tracing::error!(error = %source, "organization command observation capture failed");
                ApiError::InvalidCommunicationQuery(
                    "organization command observation capture failed",
                )
            }
            OrganizationCommandServiceError::Organization(source) => ApiError::from(source),
            OrganizationCommandServiceError::Core(source) => ApiError::from(source),
            OrganizationCommandServiceError::Enrichment(source) => ApiError::from(source),
            OrganizationCommandServiceError::Health(source) => ApiError::from(source),
        }
    }
}

impl From<OrganizationContactLinkApplicationError> for ApiError {
    fn from(error: OrganizationContactLinkApplicationError) -> Self {
        match error {
            OrganizationContactLinkApplicationError::Organization(source) => Self::from(source),
            OrganizationContactLinkApplicationError::Relationship(source) => Self::from(source),
        }
    }
}
