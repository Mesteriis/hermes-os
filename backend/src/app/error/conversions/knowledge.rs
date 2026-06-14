use super::super::types::ApiError;
use crate::domains::decisions::DecisionStoreError;
use crate::domains::obligations::ObligationStoreError;
use crate::domains::projects::core::ProjectStoreError;
use crate::domains::projects::link_reviews::ProjectLinkReviewError;
use crate::domains::relationships::RelationshipStoreError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::engines::consistency::ConsistencyError;

impl From<crate::domains::graph::core::GraphStoreError> for ApiError {
    fn from(error: crate::domains::graph::core::GraphStoreError) -> Self {
        Self::Graph(error)
    }
}

impl From<ProjectLinkReviewError> for ApiError {
    fn from(error: ProjectLinkReviewError) -> Self {
        match error {
            ProjectLinkReviewError::ProjectNotFound | ProjectLinkReviewError::TargetNotFound => {
                Self::ProjectLinkTargetNotFound
            }
            _ => Self::ProjectLinkReview(error),
        }
    }
}

impl From<ProjectStoreError> for ApiError {
    fn from(error: ProjectStoreError) -> Self {
        Self::Projects(error)
    }
}

impl From<TaskCandidateError> for ApiError {
    fn from(error: TaskCandidateError) -> Self {
        match error {
            TaskCandidateError::TaskCandidateNotFound => Self::TaskCandidateNotFound,
            _ => Self::TaskCandidate(error),
        }
    }
}

impl From<ObligationStoreError> for ApiError {
    fn from(error: ObligationStoreError) -> Self {
        match error {
            ObligationStoreError::ObligationNotFound => Self::ObligationNotFound,
            ObligationStoreError::UnknownEntityKind(_) => Self::InvalidObligationQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            ObligationStoreError::UnknownReviewState(_) => Self::InvalidObligationReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Obligation(error),
        }
    }
}

impl From<DecisionStoreError> for ApiError {
    fn from(error: DecisionStoreError) -> Self {
        match error {
            DecisionStoreError::DecisionNotFound => Self::DecisionNotFound,
            DecisionStoreError::UnknownEntityKind(_) => Self::InvalidDecisionQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            DecisionStoreError::UnknownReviewState(_) => Self::InvalidDecisionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Decision(error),
        }
    }
}

impl From<RelationshipStoreError> for ApiError {
    fn from(error: RelationshipStoreError) -> Self {
        match error {
            RelationshipStoreError::RelationshipNotFound => Self::RelationshipNotFound,
            RelationshipStoreError::UnknownEntityKind(_) => Self::InvalidRelationshipQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            RelationshipStoreError::UnknownReviewState(_) => Self::InvalidRelationshipReview(
                "review_state must be suggested, system_accepted, user_confirmed, or user_rejected",
            ),
            _ => Self::Relationship(error),
        }
    }
}

impl From<ConsistencyError> for ApiError {
    fn from(error: ConsistencyError) -> Self {
        match error {
            ConsistencyError::ObservationNotFound(_) => Self::ContradictionObservationNotFound,
            ConsistencyError::UnknownReviewState(_) => Self::InvalidContradictionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Consistency(error),
        }
    }
}

impl From<crate::engines::search::SearchError> for ApiError {
    fn from(error: crate::engines::search::SearchError) -> Self {
        tracing::error!(error = %error, "search operation failed");
        ApiError::InvalidCommunicationQuery("search operation failed")
    }
}
