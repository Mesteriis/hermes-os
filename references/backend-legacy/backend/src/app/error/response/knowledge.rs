use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::Graph(error) => {
            tracing::error!(error = %error, "graph store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "graph_store_error",
                "graph store operation failed".to_owned(),
                false,
            )
        }
        ApiError::InvalidGraphQuery(message) => bad_request("invalid_graph_query", message),
        ApiError::Projects(error) => {
            tracing::error!(error = %error, "project API store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "project_store_error",
                "project store operation failed".to_owned(),
                false,
            )
        }
        ApiError::InvalidProjectQuery(message) => bad_request("invalid_project_query", message),
        ApiError::InvalidProjectLinkReview(message) => {
            bad_request("invalid_project_link_review", message)
        }
        ApiError::ProjectLinkTargetNotFound => (
            StatusCode::NOT_FOUND,
            "project_link_target_not_found",
            "project link target was not found".to_owned(),
            false,
        ),
        ApiError::ProjectLinkReview(error) => {
            tracing::error!(error = %error, "project link review store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "project_link_review_store_error",
                "project link review store operation failed".to_owned(),
                false,
            )
        }
        ApiError::GraphNotFound => (
            StatusCode::NOT_FOUND,
            "graph_node_not_found",
            "graph node was not found".to_owned(),
            false,
        ),
        ApiError::ProjectNotFound => (
            StatusCode::NOT_FOUND,
            "project_not_found",
            "project was not found".to_owned(),
            false,
        ),
        ApiError::NotFound => (
            StatusCode::NOT_FOUND,
            "event_not_found",
            "event was not found".to_owned(),
            false,
        ),
        _ => unreachable!("knowledge response mapper received non-knowledge ApiError"),
    }
}

fn bad_request(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::BAD_REQUEST, error, message.to_owned(), false)
}
