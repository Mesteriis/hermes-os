use super::super::types::ApiError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::core::PersonCoreError;
use crate::domains::persons::identity::PersonIdentityError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::persons::service::PersonCommandServiceError;

impl From<PersonIdentityError> for ApiError {
    fn from(error: PersonIdentityError) -> Self {
        match error {
            PersonIdentityError::IdentityCandidateNotFound => Self::PersonIdentityNotFound,
            PersonIdentityError::InvalidLimit | PersonIdentityError::InvalidReviewState(_) => {
                Self::InvalidPersonIdentityReview(
                    "review_state or limit must be valid for person identity candidates",
                )
            }
            PersonIdentityError::InvalidPayload(_)
            | PersonIdentityError::MissingPayloadField(_)
            | PersonIdentityError::MissingActorId => {
                Self::InvalidPersonIdentityReview("invalid identity candidate review payload")
            }
            PersonIdentityError::Observation(_) => {
                Self::InvalidPersonIdentityReview("identity candidate evidence observation failed")
            }
            PersonIdentityError::ReviewMirror(_) => {
                Self::InvalidPersonIdentityReview("identity candidate review inbox sync failed")
            }
            _ => Self::PersonIdentity(error),
        }
    }
}

impl From<PersonProjectionError> for ApiError {
    fn from(error: PersonProjectionError) -> Self {
        Self::PersonProjection(error)
    }
}

impl From<crate::domains::persons::enrichment::PersonEnrichmentError> for ApiError {
    fn from(error: crate::domains::persons::enrichment::PersonEnrichmentError) -> Self {
        match error {
            crate::domains::persons::enrichment::PersonEnrichmentError::NotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person enrichment failed");
                ApiError::InvalidCommunicationQuery("person enrichment failed")
            }
        }
    }
}

impl From<PersonMemoryError> for ApiError {
    fn from(error: PersonMemoryError) -> Self {
        match error {
            PersonMemoryError::NotFound => ApiError::PersonIdentityNotFound,
            _ => {
                tracing::error!(error = %error, "person memory operation failed");
                ApiError::InvalidCommunicationQuery("person memory operation failed")
            }
        }
    }
}

impl From<PersonCoreError> for ApiError {
    fn from(error: PersonCoreError) -> Self {
        match error {
            PersonCoreError::IdentityNotFound | PersonCoreError::PersonaNotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person core operation failed");
                ApiError::InvalidCommunicationQuery("person core operation failed")
            }
        }
    }
}

impl From<PersonCommandServiceError> for ApiError {
    fn from(error: PersonCommandServiceError) -> Self {
        match error {
            PersonCommandServiceError::Projection(error) => Self::from(error),
            PersonCommandServiceError::Core(error) => Self::from(error),
            PersonCommandServiceError::Enrichment(error) => Self::from(error),
            PersonCommandServiceError::EnrichmentEngine(error) => Self::from(error),
            PersonCommandServiceError::Memory(error) => Self::from(error),
            PersonCommandServiceError::Health(error) => Self::from(error),
            PersonCommandServiceError::Identity(error) => Self::from(error),
            PersonCommandServiceError::Investigator(error) => Self::from(error),
            PersonCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "person manual observation capture failed");
                ApiError::InvalidCommunicationQuery("person manual observation capture failed")
            }
        }
    }
}
