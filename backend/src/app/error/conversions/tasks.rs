use super::super::types::ApiError;
use crate::application::task_relationship::TaskRelationshipApplicationError;
use crate::domains::tasks::api::TaskError;
use crate::domains::tasks::brain::TaskBrainError;
use crate::domains::tasks::command_service::TaskCommandServiceError;
use crate::domains::tasks::core::TaskCoreError;
use crate::domains::tasks::health::TaskHealthError;
use crate::domains::tasks::rules::TaskRuleError;

impl From<TaskError> for ApiError {
    fn from(error: TaskError) -> Self {
        match error {
            TaskError::NotFound => ApiError::NotFound,
            TaskError::MissingProvenance => {
                ApiError::InvalidTaskQuery("task provenance is required")
            }
            TaskError::InvalidProvenanceSpec => {
                ApiError::InvalidTaskQuery("invalid task provenance specification")
            }
            TaskError::UnknownProvenanceKind => {
                ApiError::InvalidTaskQuery("unknown task provenance kind")
            }
            TaskError::MissingProvenanceObservation => {
                ApiError::InvalidTaskQuery("missing task provenance observation")
            }
            TaskError::MissingProvenanceReference => {
                ApiError::InvalidTaskQuery("task provenance reference does not exist")
            }
            TaskError::MissingProvenanceEvidence => {
                ApiError::InvalidTaskQuery("task provenance reference has no observation evidence")
            }
            TaskError::MissingSourceIdentifier => {
                ApiError::InvalidTaskQuery("missing task source identifier")
            }
            _ => {
                tracing::error!(error = %error, "task operation failed");
                ApiError::InvalidCommunicationQuery("task operation failed")
            }
        }
    }
}

impl From<TaskCoreError> for ApiError {
    fn from(error: TaskCoreError) -> Self {
        match error {
            TaskCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task core operation failed");
                ApiError::InvalidCommunicationQuery("task core operation failed")
            }
        }
    }
}

impl From<TaskHealthError> for ApiError {
    fn from(error: TaskHealthError) -> Self {
        tracing::error!(error = %error, "task health failed");
        ApiError::InvalidCommunicationQuery("task health failed")
    }
}

impl From<TaskRuleError> for ApiError {
    fn from(error: TaskRuleError) -> Self {
        match error {
            TaskRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task rule failed");
                ApiError::InvalidCommunicationQuery("task rule failed")
            }
        }
    }
}

impl From<TaskBrainError> for ApiError {
    fn from(error: TaskBrainError) -> Self {
        match error {
            TaskBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task brain failed");
                ApiError::InvalidCommunicationQuery("task brain failed")
            }
        }
    }
}

impl From<TaskCommandServiceError> for ApiError {
    fn from(error: TaskCommandServiceError) -> Self {
        match error {
            TaskCommandServiceError::ObservationCapture { operation, source } => {
                tracing::error!(error = %source, operation, "task command observation capture failed");
                match operation {
                    "task update" => {
                        ApiError::InvalidTaskQuery("task update observation capture failed")
                    }
                    "task status" => {
                        ApiError::InvalidTaskQuery("task status observation capture failed")
                    }
                    "task archive" => {
                        ApiError::InvalidTaskQuery("task archive observation capture failed")
                    }
                    "task analyze" => {
                        ApiError::InvalidTaskQuery("task analyze observation capture failed")
                    }
                    "task evidence" => {
                        ApiError::InvalidTaskQuery("task evidence observation capture failed")
                    }
                    "task relation" => {
                        ApiError::InvalidTaskQuery("task relation observation capture failed")
                    }
                    "task checklist" => {
                        ApiError::InvalidTaskQuery("task checklist observation capture failed")
                    }
                    "task subtask" => {
                        ApiError::InvalidTaskQuery("task subtask observation capture failed")
                    }
                    _ => ApiError::InvalidTaskQuery("task observation capture failed"),
                }
            }
            TaskCommandServiceError::MissingEvidenceSourceId => {
                ApiError::InvalidTaskQuery("task evidence source id is required")
            }
            TaskCommandServiceError::ObservationStore(source) => {
                tracing::error!(error = %source, "task observation store operation failed");
                ApiError::InvalidTaskQuery("task observation store operation failed")
            }
            TaskCommandServiceError::Task(inner) => ApiError::from(inner),
            TaskCommandServiceError::Core(inner) => ApiError::from(inner),
            TaskCommandServiceError::Sqlx(source) => {
                tracing::error!(error = %source, "task command sql operation failed");
                ApiError::InvalidTaskQuery("task command operation failed")
            }
        }
    }
}

impl From<TaskRelationshipApplicationError> for ApiError {
    fn from(error: TaskRelationshipApplicationError) -> Self {
        match error {
            TaskRelationshipApplicationError::Sqlx(source) => {
                tracing::error!(error = %source, "task relationship sql operation failed");
                ApiError::InvalidTaskQuery("task relationship operation failed")
            }
            TaskRelationshipApplicationError::Task(source) => ApiError::from(source),
            TaskRelationshipApplicationError::Observation(source) => {
                tracing::error!(error = %source, "task relationship observation capture failed");
                ApiError::InvalidTaskQuery("task relationship observation capture failed")
            }
            TaskRelationshipApplicationError::RelationshipGraph(source) => ApiError::from(source),
        }
    }
}
