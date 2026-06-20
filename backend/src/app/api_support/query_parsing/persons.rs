use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::persons::identity::PersonIdentityReviewState;

pub(crate) struct PersonIdentityCandidatesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_person_identity_candidates_query(
    raw_query: Option<&str>,
) -> Result<PersonIdentityCandidatesQuery, ApiError> {
    let mut query = PersonIdentityCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidPersonIdentityReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_person_identity_review_state(
    value: &str,
) -> Result<PersonIdentityReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(PersonIdentityReviewState::Suggested),
        "user_confirmed" => Ok(PersonIdentityReviewState::UserConfirmed),
        "user_rejected" => Ok(PersonIdentityReviewState::UserRejected),
        _ => Err(ApiError::InvalidPersonIdentityReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_person_identity_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidPersonIdentityReview(match field {
            "command_id" => "command_id must not be empty",
            "identity_candidate_id" => "identity_candidate_id must not be empty",
            "person_id" => "person_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}
