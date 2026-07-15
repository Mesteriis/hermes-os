use url::form_urlencoded;

use crate::app::error::types::ApiError;
use crate::domains::tasks::candidates::models::TaskCandidateReviewState;

pub(crate) struct TaskCandidatesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_task_candidates_query(
    raw_query: Option<&str>,
) -> Result<TaskCandidatesQuery, ApiError> {
    let mut query = TaskCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidTaskCandidateQuery("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_task_candidate_review_state(
    value: &str,
) -> Result<TaskCandidateReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(TaskCandidateReviewState::Suggested),
        "user_confirmed" => Ok(TaskCandidateReviewState::UserConfirmed),
        "user_rejected" => Ok(TaskCandidateReviewState::UserRejected),
        _ => Err(ApiError::InvalidTaskCandidateReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_task_candidate_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidTaskCandidateReview(match field {
            "command_id" => "command_id must not be empty",
            "review_state" => "review_state must not be empty",
            "task_candidate_id" => "task_candidate_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}
