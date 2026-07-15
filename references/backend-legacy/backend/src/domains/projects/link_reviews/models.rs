use chrono::{DateTime, Utc};

use super::errors::ProjectLinkReviewError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkTargetKind {
    Message,
    Document,
}

impl ProjectLinkTargetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
        }
    }

    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, ProjectLinkReviewError> {
        match value.as_ref() {
            "message" => Ok(Self::Message),
            "document" => Ok(Self::Document),
            _ => Err(ProjectLinkReviewError::InvalidTargetKind(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ProjectLinkReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, ProjectLinkReviewError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ProjectLinkReviewError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReviewCommand {
    pub command_id: String,
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReviewCommandResult {
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReview {
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub event_id: String,
    pub actor_id: String,
    pub reviewed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectReviewedTarget {
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
}

pub(crate) struct ReviewEventApplication<'a> {
    pub(crate) target_kind: ProjectLinkTargetKind,
    pub(crate) project_id: &'a str,
    pub(crate) target_id: &'a str,
    pub(crate) review_state: ProjectLinkReviewState,
    pub(crate) event_id: &'a str,
    pub(crate) actor_id: &'a str,
    pub(crate) reviewed_at: DateTime<Utc>,
}
