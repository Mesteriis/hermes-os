use serde::Deserialize;
use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectLinkTargetKind};

#[derive(Deserialize)]
pub(crate) struct ProjectLinkCandidatesQuery {
    pub(crate) limit: Option<usize>,
}

pub(crate) struct ProjectsQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_projects_query(raw_query: Option<&str>) -> Result<ProjectsQuery, ApiError> {
    let mut query = ProjectsQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| ApiError::InvalidProjectQuery("limit must be an integer"))?,
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_project_link_candidates_query(
    raw_query: Option<&str>,
) -> Result<ProjectLinkCandidatesQuery, ApiError> {
    let mut query = ProjectLinkCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| {
                            ApiError::InvalidProjectLinkReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_project_link_target_kind(
    value: &str,
) -> Result<ProjectLinkTargetKind, ApiError> {
    match value.trim() {
        "message" => Ok(ProjectLinkTargetKind::Message),
        "document" => Ok(ProjectLinkTargetKind::Document),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "target_kind must be message or document",
        )),
    }
}

pub(crate) fn parse_project_link_review_state(
    value: &str,
) -> Result<ProjectLinkReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(ProjectLinkReviewState::Suggested),
        "user_confirmed" => Ok(ProjectLinkReviewState::UserConfirmed),
        "user_rejected" => Ok(ProjectLinkReviewState::UserRejected),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_project_link_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidProjectLinkReview(field));
    }

    Ok(normalized.to_owned())
}
