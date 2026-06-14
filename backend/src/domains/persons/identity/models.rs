use chrono::{DateTime, Utc};
use serde::Serialize;

use super::constants::PERSON_IDENTITY_ID_PREFIX;
use super::errors::PersonIdentityError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonIdentityCandidateKind {
    MergePersons,
    AttachEmailAddress,
    SplitPerson,
}

impl PersonIdentityCandidateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MergePersons => "merge_persons",
            Self::AttachEmailAddress => "attach_email_address",
            Self::SplitPerson => "split_person",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl PersonIdentityReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(super) fn parse(value: impl AsRef<str>) -> Result<Self, PersonIdentityError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(PersonIdentityError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: PersonIdentityReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonIdentityReviewCommandResult {
    pub identity_candidate_id: String,
    pub review_state: PersonIdentityReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonIdentityCandidate {
    pub identity_candidate_id: String,
    pub candidate_kind: String,
    pub left_person_id: String,
    pub right_person_id: Option<String>,
    pub email_address: Option<String>,
    pub evidence_summary: String,
    pub confidence: f64,
    pub review_state: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonIdentityDetail {
    pub items: Vec<PersonIdentityCandidate>,
}

#[derive(Debug)]
pub(super) struct PersonIdentityCandidatePayload {
    pub(super) candidate_kind: PersonIdentityCandidateKind,
    pub(super) left_person_id: String,
    pub(super) right_person_id: Option<String>,
    pub(super) email_address: Option<String>,
    pub(super) evidence_summary: String,
    pub(super) confidence: f64,
}

impl PersonIdentityCandidatePayload {
    pub(super) fn identity_candidate_id(&self) -> String {
        let left = self.left_person_id.clone();
        let right = self
            .right_person_id
            .clone()
            .unwrap_or_else(|| String::from("single"));

        match self.candidate_kind {
            PersonIdentityCandidateKind::MergePersons => {
                format!("{PERSON_IDENTITY_ID_PREFIX}merge_persons:{left}:{right}")
            }
            PersonIdentityCandidateKind::AttachEmailAddress => {
                format!("{PERSON_IDENTITY_ID_PREFIX}attach_email_address:{left}:{right}")
            }
            PersonIdentityCandidateKind::SplitPerson => {
                format!("{PERSON_IDENTITY_ID_PREFIX}split_person:{left}:{right}")
            }
        }
    }
}
