use chrono::{DateTime, Utc};
use serde::Serialize;

use super::constants::PERSONA_IDENTITY_ID_PREFIX;
use super::errors::PersonaIdentityError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonaIdentityCandidateKind {
    MergePersonas,
    AttachEmailAddress,
    SplitPersona,
}

impl PersonaIdentityCandidateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MergePersonas => "merge_personas",
            Self::AttachEmailAddress => "attach_email_address",
            Self::SplitPersona => "split_persona",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonaIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl PersonaIdentityReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(super) fn parse(value: impl AsRef<str>) -> Result<Self, PersonaIdentityError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(PersonaIdentityError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonaIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: PersonaIdentityReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonaIdentityReviewCommandResult {
    pub identity_candidate_id: String,
    pub review_state: PersonaIdentityReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonaIdentityCandidate {
    pub identity_candidate_id: String,
    pub candidate_kind: String,
    pub left_persona_id: String,
    pub right_persona_id: Option<String>,
    pub email_address: Option<String>,
    pub evidence_summary: String,
    pub confidence: f64,
    pub review_state: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonaIdentityDetail {
    pub items: Vec<PersonaIdentityCandidate>,
}

#[derive(Debug)]
pub(crate) struct PersonaIdentityCandidatePayload {
    pub(crate) candidate_kind: PersonaIdentityCandidateKind,
    pub(crate) left_persona_id: String,
    pub(crate) right_persona_id: Option<String>,
    pub(crate) email_address: Option<String>,
    pub(crate) evidence_summary: String,
    pub(crate) confidence: f64,
}

impl PersonaIdentityCandidatePayload {
    pub(crate) fn identity_candidate_id(&self) -> String {
        let left = self.left_persona_id.clone();
        let right = self
            .right_persona_id
            .clone()
            .unwrap_or_else(|| String::from("single"));

        match self.candidate_kind {
            PersonaIdentityCandidateKind::MergePersonas => {
                format!("{PERSONA_IDENTITY_ID_PREFIX}merge_personas:{left}:{right}")
            }
            PersonaIdentityCandidateKind::AttachEmailAddress => {
                let email = self
                    .email_address
                    .clone()
                    .unwrap_or_else(|| String::from("missing"));
                format!(
                    "{PERSONA_IDENTITY_ID_PREFIX}attach_email_address:{left}:{}:{email}",
                    email.len()
                )
            }
            PersonaIdentityCandidateKind::SplitPersona => {
                format!("{PERSONA_IDENTITY_ID_PREFIX}split_persona:{left}:{right}")
            }
        }
    }
}
