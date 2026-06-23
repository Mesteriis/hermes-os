mod ai;
mod communication;
mod communications;
mod document_processing;
mod integrations;
mod knowledge;
mod persons;
mod platform;
mod review;
mod tasks;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use serde::Serialize;

use super::types::ApiError;

pub(super) type ErrorParts = (StatusCode, &'static str, String, bool);

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, message, authenticate) = parts(self);
        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        if authenticate {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::DatabaseNotConfigured
        | ApiError::SecretVaultNotConfigured
        | ApiError::HostVault(_)
        | ApiError::InvalidEnvelope(_)
        | ApiError::FailedPrecondition(_)
        | ApiError::Audit(_)
        | ApiError::Store(_)
        | ApiError::SettingNotFound
        | ApiError::Settings(_)
        | ApiError::SignalHub(_) => platform::parts(error),
        ApiError::Graph(_)
        | ApiError::InvalidGraphQuery(_)
        | ApiError::Projects(_)
        | ApiError::InvalidProjectQuery(_)
        | ApiError::InvalidProjectLinkReview(_)
        | ApiError::ProjectLinkTargetNotFound
        | ApiError::ProjectLinkReview(_)
        | ApiError::GraphNotFound
        | ApiError::ProjectNotFound
        | ApiError::NotFound => knowledge::parts(error),
        ApiError::InvalidTaskCandidateQuery(_)
        | ApiError::InvalidTaskCandidateReview(_)
        | ApiError::InvalidObligationQuery(_)
        | ApiError::InvalidObligationReview(_)
        | ApiError::InvalidDecisionQuery(_)
        | ApiError::InvalidDecisionReview(_)
        | ApiError::InvalidRelationshipQuery(_)
        | ApiError::InvalidRelationshipReview(_)
        | ApiError::InvalidContradictionQuery(_)
        | ApiError::InvalidContradictionReview(_)
        | ApiError::InvalidReviewQuery(_)
        | ApiError::InvalidReviewItem(_)
        | ApiError::TaskCandidateNotFound
        | ApiError::TaskCandidate(_)
        | ApiError::ObligationNotFound
        | ApiError::Obligation(_)
        | ApiError::DecisionNotFound
        | ApiError::Decision(_)
        | ApiError::RelationshipNotFound
        | ApiError::Relationship(_)
        | ApiError::ContradictionObservationNotFound
        | ApiError::Consistency(_)
        | ApiError::ReviewItemNotFound
        | ApiError::ReviewInbox(_)
        | ApiError::ReviewPromotion(_) => review::parts(error),
        ApiError::InvalidTaskQuery(_) => tasks::parts(error),
        ApiError::InvalidPersonaQuery(_)
        | ApiError::InvalidPersonIdentityReview(_)
        | ApiError::PersonIdentityNotFound
        | ApiError::PersonProjection(_)
        | ApiError::PersonIdentity(_) => persons::parts(error),
        ApiError::Messages(_)
        | ApiError::CommunicationIngestion(_)
        | ApiError::CommunicationStorage(_)
        | ApiError::InvalidCommunicationQuery(_)
        | ApiError::EmailAccountDeleteConflict
        | ApiError::ProviderWriteConfirmationRequired
        | ApiError::CommunicationMessageNotFound
        | ApiError::AccountSetup(_)
        | ApiError::AccountSetupState
        | ApiError::AccountSetupPendingGrantNotFound
        | ApiError::AccountSetupStateMismatch => communication::parts(error),
        ApiError::InvalidDocumentProcessingQuery(message) => {
            document_processing::invalid_query_parts(message)
        }
        ApiError::DocumentProcessing(error) => document_processing::parts(error),
        ApiError::AiRunNotFound => ai::ai_run_not_found_parts(),
        ApiError::Ai(error) => ai::ai_error_parts(error),
        ApiError::AiControlCenter(error) => ai::control_center_error_parts(error),
        ApiError::Telegram(error) => integrations::telegram_error_parts(error),
        ApiError::WhatsappWeb(error) => integrations::whatsapp_web_error_parts(error),
        ApiError::Automation(error) => integrations::automation_error_parts(error),
        ApiError::Call(error) => integrations::call_error_parts(error),
    }
}
