use super::super::types::ApiError;
use crate::domains::tasks::api::TaskError;
use crate::domains::tasks::brain::TaskBrainError;
use crate::domains::tasks::core::TaskCoreError;
use crate::domains::tasks::health::TaskHealthError;
use crate::domains::tasks::rules::TaskRuleError;

impl From<TaskError> for ApiError {
    fn from(error: TaskError) -> Self {
        match error {
            TaskError::NotFound => ApiError::NotFound,
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
