use super::support::*;

impl From<EnrichmentEngineError> for ApiError {
    fn from(error: EnrichmentEngineError) -> Self {
        tracing::error!(error = %error, "enrichment engine operation failed");
        ApiError::InvalidCommunicationQuery("enrichment engine operation failed")
    }
}

impl From<PersonaExpertiseError> for ApiError {
    fn from(error: PersonaExpertiseError) -> Self {
        tracing::error!(error = %error, "expertise operation failed");
        ApiError::InvalidCommunicationQuery("expertise operation failed")
    }
}

impl From<PersonaTrustError> for ApiError {
    fn from(error: PersonaTrustError) -> Self {
        tracing::error!(error = %error, "trust operation failed");
        ApiError::InvalidCommunicationQuery("trust operation failed")
    }
}

impl From<PersonaHealthError> for ApiError {
    fn from(error: PersonaHealthError) -> Self {
        tracing::error!(error = %error, "health operation failed");
        ApiError::InvalidCommunicationQuery("health operation failed")
    }
}

impl From<InvestigatorError> for ApiError {
    fn from(error: InvestigatorError) -> Self {
        match error {
            InvestigatorError::PersonaNotFound | InvestigatorError::DossierSnapshotNotFound => {
                ApiError::PersonaIdentityNotFound
            }
            InvestigatorError::InvalidDossierReviewState => ApiError::InvalidCommunicationQuery(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<AnalyticsError> for ApiError {
    fn from(error: AnalyticsError) -> Self {
        tracing::error!(error = %error, "analytics operation failed");
        ApiError::InvalidCommunicationQuery("analytics operation failed")
    }
}

impl From<ExportError> for ApiError {
    fn from(error: ExportError) -> Self {
        tracing::error!(error = %error, "export operation failed");
        ApiError::InvalidCommunicationQuery("export operation failed")
    }
}
