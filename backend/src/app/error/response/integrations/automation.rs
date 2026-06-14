use axum::http::StatusCode;

use crate::engines::automation::AutomationError;

use super::super::ErrorParts;

pub(super) fn automation_error_parts(error: AutomationError) -> ErrorParts {
    match error {
        AutomationError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_automation_request",
            message,
            false,
        ),
        AutomationError::PolicyNotFound => (
            StatusCode::NOT_FOUND,
            "automation_policy_not_found",
            "automation policy was not found".to_owned(),
            false,
        ),
        AutomationError::PolicyDisabled
        | AutomationError::ChatNotAllowed
        | AutomationError::MissingTemplateVariable(_)
        | AutomationError::UndeclaredTemplateVariable(_) => (
            StatusCode::FORBIDDEN,
            "automation_policy_denied",
            error.to_string(),
            false,
        ),
        AutomationError::EventEnvelope(error) => (
            StatusCode::BAD_REQUEST,
            "invalid_automation_event",
            error.to_string(),
            false,
        ),
        AutomationError::EventStore(error) => {
            internal(error, "automation event store operation failed")
        }
        AutomationError::Sqlx(error) => internal(error, "automation database operation failed"),
    }
}

fn internal(error: impl std::fmt::Display, log: &'static str) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "automation_store_error",
        "automation operation failed".to_owned(),
        false,
    )
}
