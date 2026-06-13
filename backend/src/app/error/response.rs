mod ai;
mod document_processing;
mod integrations;
mod mail;
mod persons;

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
        let (status, error, message, authenticate) = match self {
            Self::DatabaseNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_not_configured",
                "DATABASE_URL is not configured".to_owned(),
                false,
            ),
            Self::SecretVaultNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "secret_vault_not_configured",
                "host vault must be initialized and unlocked for account setup".to_owned(),
                false,
            ),
            Self::HostVault(error) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "host_vault_error",
                error.to_string(),
                false,
            ),
            Self::InvalidEnvelope(error) => (
                StatusCode::BAD_REQUEST,
                "invalid_event_envelope",
                error.to_string(),
                false,
            ),
            Self::Audit(error) => {
                tracing::error!(error = %error, "event API audit operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "api_audit_error",
                    "API audit operation failed".to_owned(),
                    false,
                )
            }
            Self::Store(error) if error.is_unique_violation() => (
                StatusCode::CONFLICT,
                "event_conflict",
                "event already exists or violates idempotency constraints".to_owned(),
                false,
            ),
            Self::Store(error) => {
                tracing::error!(error = %error, "event API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "event_store_error",
                    "event store operation failed".to_owned(),
                    false,
                )
            }
            Self::Graph(error) => {
                tracing::error!(error = %error, "graph store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "graph_store_error",
                    "graph store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidGraphQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_graph_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidPersonaQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_persona_query",
                message.to_owned(),
                false,
            ),
            Self::Projects(error) => {
                tracing::error!(error = %error, "project API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "project_store_error",
                    "project store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidProjectQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_project_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidProjectLinkReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_project_link_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidTaskCandidateQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_task_candidate_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidTaskCandidateReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_task_candidate_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidObligationQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_obligation_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidObligationReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_obligation_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidDecisionQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_decision_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidDecisionReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_decision_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidRelationshipQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_relationship_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidRelationshipReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_relationship_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidContradictionQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_contradiction_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidContradictionReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_contradiction_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidPersonIdentityReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_person_identity_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidDocumentProcessingQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_document_processing_query",
                message.to_owned(),
                false,
            ),
            Self::SettingNotFound => (
                StatusCode::NOT_FOUND,
                "setting_not_found",
                "application setting was not found".to_owned(),
                false,
            ),
            Self::Settings(error) if error.is_invalid_request() => (
                StatusCode::BAD_REQUEST,
                "invalid_application_setting",
                error.to_string(),
                false,
            ),
            Self::Settings(error) => {
                tracing::error!(error = %error, "application settings operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "application_settings_error",
                    "application settings operation failed".to_owned(),
                    false,
                )
            }
            Self::DocumentProcessing(error) => document_processing::parts(error),
            Self::TaskCandidateNotFound => (
                StatusCode::NOT_FOUND,
                "task_candidate_not_found",
                "task candidate was not found".to_owned(),
                false,
            ),
            Self::TaskCandidate(error) => {
                tracing::error!(error = %error, "task candidate store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "task_candidate_store_error",
                    "task candidate store operation failed".to_owned(),
                    false,
                )
            }
            Self::ObligationNotFound => (
                StatusCode::NOT_FOUND,
                "obligation_not_found",
                "obligation was not found".to_owned(),
                false,
            ),
            Self::Obligation(error) => {
                tracing::error!(error = %error, "obligation store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "obligation_store_error",
                    "obligation store operation failed".to_owned(),
                    false,
                )
            }
            Self::DecisionNotFound => (
                StatusCode::NOT_FOUND,
                "decision_not_found",
                "decision was not found".to_owned(),
                false,
            ),
            Self::Decision(error) => {
                tracing::error!(error = %error, "decision store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "decision_store_error",
                    "decision store operation failed".to_owned(),
                    false,
                )
            }
            Self::RelationshipNotFound => (
                StatusCode::NOT_FOUND,
                "relationship_not_found",
                "relationship was not found".to_owned(),
                false,
            ),
            Self::Relationship(error) => {
                tracing::error!(error = %error, "relationship store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "relationship_store_error",
                    "relationship store operation failed".to_owned(),
                    false,
                )
            }
            Self::ContradictionObservationNotFound => (
                StatusCode::NOT_FOUND,
                "contradiction_observation_not_found",
                "contradiction observation was not found".to_owned(),
                false,
            ),
            Self::Consistency(error) => {
                tracing::error!(error = %error, "consistency engine operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "consistency_engine_error",
                    "consistency engine operation failed".to_owned(),
                    false,
                )
            }
            Self::AiRunNotFound => (
                StatusCode::NOT_FOUND,
                "ai_run_not_found",
                "AI run was not found".to_owned(),
                false,
            ),
            Self::Ai(error) => ai::ai_error_parts(error),
            Self::AiControlCenter(error) => ai::control_center_error_parts(error),
            Self::Telegram(error) => integrations::telegram_error_parts(error),
            Self::WhatsappWeb(error) => integrations::whatsapp_web_error_parts(error),
            Self::Automation(error) => integrations::automation_error_parts(error),
            Self::Call(error) => integrations::call_error_parts(error),
            Self::ProjectLinkTargetNotFound => (
                StatusCode::NOT_FOUND,
                "project_link_target_not_found",
                "project link target was not found".to_owned(),
                false,
            ),
            Self::ProjectLinkReview(error) => {
                tracing::error!(error = %error, "project link review store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "project_link_review_store_error",
                    "project link review store operation failed".to_owned(),
                    false,
                )
            }
            Self::PersonIdentityNotFound => (
                StatusCode::NOT_FOUND,
                "person_identity_candidate_not_found",
                "person identity candidate was not found".to_owned(),
                false,
            ),
            Self::PersonProjection(error) => persons::projection_error_parts(error),
            Self::PersonIdentity(error) => {
                tracing::error!(
                    error = %error,
                    "person identity store operation failed"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "person_identity_store_error",
                    "person identity store operation failed".to_owned(),
                    false,
                )
            }
            Self::Messages(error) => {
                tracing::error!(error = %error, "communication message API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "communication_message_store_error",
                    "communication message store operation failed".to_owned(),
                    false,
                )
            }
            Self::CommunicationIngestion(error) => {
                tracing::error!(error = %error, "communication account API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "communication_account_store_error",
                    "communication account store operation failed".to_owned(),
                    false,
                )
            }
            Self::MailStorage(error) => {
                tracing::error!(error = %error, "communication attachment API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "communication_attachment_store_error",
                    "communication attachment store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidCommunicationQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_communication_query",
                message.to_owned(),
                false,
            ),
            Self::EmailAccountDeleteConflict => (
                StatusCode::CONFLICT,
                "email_account_delete_conflict",
                "email account has retained communication evidence and cannot be deleted"
                    .to_owned(),
                false,
            ),
            Self::ProviderWriteConfirmationRequired => (
                StatusCode::BAD_REQUEST,
                "provider_write_confirmation_required",
                "explicit provider write confirmation is required".to_owned(),
                false,
            ),
            Self::CommunicationMessageNotFound => (
                StatusCode::NOT_FOUND,
                "communication_message_not_found",
                "communication message was not found".to_owned(),
                false,
            ),
            Self::AccountSetup(error) => mail::account_setup_error_parts(error),
            Self::AccountSetupState => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "account_setup_state_error",
                "account setup state is unavailable".to_owned(),
                false,
            ),
            Self::AccountSetupPendingGrantNotFound => (
                StatusCode::NOT_FOUND,
                "account_setup_pending_grant_not_found",
                "pending Gmail OAuth setup was not found".to_owned(),
                false,
            ),
            Self::AccountSetupStateMismatch => (
                StatusCode::BAD_REQUEST,
                "account_setup_state_mismatch",
                "Gmail OAuth state does not match the pending setup".to_owned(),
                false,
            ),
            Self::GraphNotFound => (
                StatusCode::NOT_FOUND,
                "graph_node_not_found",
                "graph node was not found".to_owned(),
                false,
            ),
            Self::ProjectNotFound => (
                StatusCode::NOT_FOUND,
                "project_not_found",
                "project was not found".to_owned(),
                false,
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "event_not_found",
                "event was not found".to_owned(),
                false,
            ),
        };

        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        if authenticate {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}
