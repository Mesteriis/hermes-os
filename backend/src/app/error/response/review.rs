use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::InvalidTaskCandidateQuery(message) => {
            bad_request("invalid_task_candidate_query", message)
        }
        ApiError::InvalidTaskCandidateReview(message) => {
            bad_request("invalid_task_candidate_review", message)
        }
        ApiError::InvalidObligationQuery(message) => {
            bad_request("invalid_obligation_query", message)
        }
        ApiError::InvalidObligationReview(message) => {
            bad_request("invalid_obligation_review", message)
        }
        ApiError::InvalidDecisionQuery(message) => bad_request("invalid_decision_query", message),
        ApiError::InvalidDecisionReview(message) => bad_request("invalid_decision_review", message),
        ApiError::InvalidRelationshipQuery(message) => {
            bad_request("invalid_relationship_query", message)
        }
        ApiError::InvalidRelationshipReview(message) => {
            bad_request("invalid_relationship_review", message)
        }
        ApiError::InvalidContradictionQuery(message) => {
            bad_request("invalid_contradiction_query", message)
        }
        ApiError::InvalidContradictionReview(message) => {
            bad_request("invalid_contradiction_review", message)
        }
        ApiError::TaskCandidateNotFound => {
            not_found("task_candidate_not_found", "task candidate was not found")
        }
        ApiError::TaskCandidate(error) => internal_store(
            error,
            "task candidate store operation failed",
            "task_candidate_store_error",
        ),
        ApiError::ObligationNotFound => {
            not_found("obligation_not_found", "obligation was not found")
        }
        ApiError::Obligation(error) => internal_store(
            error,
            "obligation store operation failed",
            "obligation_store_error",
        ),
        ApiError::DecisionNotFound => not_found("decision_not_found", "decision was not found"),
        ApiError::Decision(error) => internal_store(
            error,
            "decision store operation failed",
            "decision_store_error",
        ),
        ApiError::RelationshipNotFound => {
            not_found("relationship_not_found", "relationship was not found")
        }
        ApiError::Relationship(error) => internal_store(
            error,
            "relationship store operation failed",
            "relationship_store_error",
        ),
        ApiError::ContradictionObservationNotFound => not_found(
            "contradiction_observation_not_found",
            "contradiction observation was not found",
        ),
        ApiError::Consistency(error) => {
            tracing::error!(error = %error, "consistency engine operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "consistency_engine_error",
                "consistency engine operation failed".to_owned(),
                false,
            )
        }
        _ => unreachable!("review response mapper received non-review ApiError"),
    }
}

fn bad_request(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::BAD_REQUEST, error, message.to_owned(), false)
}

fn not_found(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::NOT_FOUND, error, message.to_owned(), false)
}

fn internal_store(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        log.to_owned(),
        false,
    )
}
