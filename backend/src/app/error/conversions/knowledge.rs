use super::super::types::ApiError;
use crate::application::consistency_review::ContradictionReviewServiceError;
use crate::application::review_promotion::ReviewPromotionError;
use crate::domains::decisions::{DecisionCommandServiceError, DecisionStoreError};
use crate::domains::obligations::{ObligationCommandServiceError, ObligationStoreError};
use crate::domains::projects::core::ProjectStoreError;
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewError, ProjectLinkReviewServiceError,
};
use crate::domains::relationships::{RelationshipCommandServiceError, RelationshipStoreError};
use crate::domains::review::{ReviewInboxError, ReviewInboxServiceError};
use crate::domains::tasks::candidates::{TaskCandidateError, TaskCandidateReviewServiceError};
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

impl From<ProjectLinkReviewServiceError> for ApiError {
    fn from(error: ProjectLinkReviewServiceError) -> Self {
        match error {
            ProjectLinkReviewServiceError::ProjectLinkReview(error) => Self::from(error),
            ProjectLinkReviewServiceError::Observation(error) => {
                tracing::error!(error = %error, "project link review observation capture failed");
                Self::InvalidCommunicationQuery("project link review observation capture failed")
            }
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

impl From<TaskCandidateReviewServiceError> for ApiError {
    fn from(error: TaskCandidateReviewServiceError) -> Self {
        match error {
            TaskCandidateReviewServiceError::TaskCandidate(error) => Self::from(error),
            TaskCandidateReviewServiceError::Observation(error) => {
                tracing::error!(error = %error, "task candidate review observation capture failed");
                Self::InvalidTaskCandidateQuery("task candidate review observation capture failed")
            }
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
            ObligationStoreError::MissingEvidence => {
                Self::InvalidObligationQuery("obligation evidence is required")
            }
            ObligationStoreError::ObservationNotFound(_) => {
                Self::InvalidObligationQuery("obligation evidence observation was not found")
            }
            ObligationStoreError::InvalidObservationEvidenceSource => Self::InvalidObligationQuery(
                "obligation observation evidence must use the same source_id and observation_id",
            ),
            ObligationStoreError::UnknownEvidenceSourceKind(_) => {
                Self::InvalidObligationQuery("obligation evidence source kind is invalid")
            }
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
            DecisionStoreError::MissingEvidence => {
                Self::InvalidDecisionQuery("decision evidence is required")
            }
            DecisionStoreError::ObservationNotFound(_) => {
                Self::InvalidDecisionQuery("decision evidence observation was not found")
            }
            DecisionStoreError::InvalidObservationEvidenceSource => Self::InvalidDecisionQuery(
                "decision observation evidence must use the same source_id and observation_id",
            ),
            DecisionStoreError::UnknownEvidenceSourceKind(_) => {
                Self::InvalidDecisionQuery("decision evidence source kind is invalid")
            }
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
            RelationshipStoreError::MissingEvidence => {
                Self::InvalidRelationshipQuery("relationship evidence is required")
            }
            RelationshipStoreError::ObservationNotFound(_) => {
                Self::InvalidRelationshipQuery("relationship evidence observation was not found")
            }
            RelationshipStoreError::InvalidObservationEvidenceSource => {
                Self::InvalidRelationshipQuery(
                    "relationship observation evidence must use the same source_id and observation_id",
                )
            }
            RelationshipStoreError::UnknownEvidenceSourceKind(_) => {
                Self::InvalidRelationshipQuery("relationship evidence source kind is invalid")
            }
            RelationshipStoreError::UnknownReviewState(_) => Self::InvalidRelationshipReview(
                "review_state must be suggested, system_accepted, user_confirmed, or user_rejected",
            ),
            _ => Self::Relationship(error),
        }
    }
}

impl From<DecisionCommandServiceError> for ApiError {
    fn from(error: DecisionCommandServiceError) -> Self {
        match error {
            DecisionCommandServiceError::Decision(error) => Self::from(error),
            DecisionCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "decision review observation capture failed");
                Self::InvalidDecisionReview("decision review observation capture failed")
            }
        }
    }
}

impl From<ObligationCommandServiceError> for ApiError {
    fn from(error: ObligationCommandServiceError) -> Self {
        match error {
            ObligationCommandServiceError::Obligation(error) => Self::from(error),
            ObligationCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "obligation review observation capture failed");
                Self::InvalidObligationReview("obligation review observation capture failed")
            }
        }
    }
}

impl From<RelationshipCommandServiceError> for ApiError {
    fn from(error: RelationshipCommandServiceError) -> Self {
        match error {
            RelationshipCommandServiceError::Relationship(error) => Self::from(error),
            RelationshipCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "relationship review observation capture failed");
                Self::InvalidRelationshipReview("relationship review observation capture failed")
            }
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

impl From<ContradictionReviewServiceError> for ApiError {
    fn from(error: ContradictionReviewServiceError) -> Self {
        match error {
            ContradictionReviewServiceError::Consistency(error) => Self::from(error),
            ContradictionReviewServiceError::Observation(error) => {
                tracing::error!(error = %error, "contradiction review observation capture failed");
                Self::InvalidContradictionReview("contradiction review observation capture failed")
            }
            ContradictionReviewServiceError::ReviewWorkflow(error) => {
                tracing::error!(error = %error, "contradiction review inbox sync failed");
                Self::InvalidContradictionReview("contradiction review inbox sync failed")
            }
        }
    }
}

impl From<ReviewInboxError> for ApiError {
    fn from(error: ReviewInboxError) -> Self {
        match error {
            ReviewInboxError::ReviewItemNotFound(_) => Self::ReviewItemNotFound,
            ReviewInboxError::ObservationNotFound(_) => {
                Self::InvalidReviewQuery("review evidence observation was not found")
            }
            ReviewInboxError::UnknownItemKind(_) => Self::InvalidReviewItem(
                "item_kind must be new_person, new_organization, identity_candidate, project_link_candidate, contradiction_candidate, potential_task, potential_obligation, potential_decision, potential_relationship, potential_project, or knowledge_candidate",
            ),
            ReviewInboxError::UnknownStatus(_) => Self::InvalidReviewQuery(
                "status must be new, in_review, approved, promoted, dismissed, or archived",
            ),
            _ => Self::ReviewInbox(error),
        }
    }
}

impl From<ReviewPromotionError> for ApiError {
    fn from(error: ReviewPromotionError) -> Self {
        match error {
            ReviewPromotionError::ReviewInbox(inner) => Self::from(inner),
            ReviewPromotionError::Task(inner) => {
                Self::ReviewPromotion(ReviewPromotionError::Task(inner))
            }
            ReviewPromotionError::TaskCore(inner) => {
                Self::ReviewPromotion(ReviewPromotionError::TaskCore(inner))
            }
            ReviewPromotionError::DocumentImport(inner) => {
                Self::ReviewPromotion(ReviewPromotionError::DocumentImport(inner))
            }
            ReviewPromotionError::Decision(inner) => Self::from(inner),
            ReviewPromotionError::Obligation(inner) => Self::from(inner),
            ReviewPromotionError::Relationship(inner) => Self::from(inner),
            ReviewPromotionError::PersonIdentity(inner) => Self::from(inner),
            ReviewPromotionError::PersonProjection(_) => {
                Self::InvalidReviewQuery("review promotion person target is invalid")
            }
            ReviewPromotionError::ProjectLinkReview(inner) => Self::from(inner),
            ReviewPromotionError::ProjectCommandPort(_) => {
                Self::InvalidReviewQuery("review promotion project target is invalid")
            }
            ReviewPromotionError::Organization(_) => {
                Self::InvalidReviewQuery("review promotion organization target is invalid")
            }
            ReviewPromotionError::Observation(inner) => {
                Self::ReviewPromotion(ReviewPromotionError::Observation(inner))
            }
            ReviewPromotionError::InvalidTarget(_) => {
                Self::InvalidReviewQuery("review promotion target is invalid for this item kind")
            }
            ReviewPromotionError::Sqlx(inner) => {
                Self::ReviewPromotion(ReviewPromotionError::Sqlx(inner))
            }
        }
    }
}

impl From<ReviewInboxServiceError> for ApiError {
    fn from(error: ReviewInboxServiceError) -> Self {
        match error {
            ReviewInboxServiceError::StatusObservationCapture(inner) => {
                tracing::error!(error = %inner, "review item status observation capture failed");
                Self::InvalidReviewQuery("review item status observation capture failed")
            }
            ReviewInboxServiceError::PromotionObservationCapture(inner) => {
                tracing::error!(error = %inner, "review item promote observation capture failed");
                Self::InvalidReviewQuery("review item promote observation capture failed")
            }
            ReviewInboxServiceError::ReviewInbox(inner) => Self::from(inner),
        }
    }
}

impl From<crate::engines::search::SearchError> for ApiError {
    fn from(error: crate::engines::search::SearchError) -> Self {
        tracing::error!(error = %error, "search operation failed");
        ApiError::InvalidCommunicationQuery("search operation failed")
    }
}
