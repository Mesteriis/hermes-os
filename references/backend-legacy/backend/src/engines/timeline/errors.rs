use thiserror::Error;

use crate::platform::projections::ProjectionRunnerError;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TimelineEngineError {
    #[error("timeline event {0} must not be empty")]
    EmptyField(&'static str),
    #[error("timeline period start must not be after period end")]
    InvalidPeriod,
    #[error("timeline gap threshold must be greater than zero")]
    InvalidGapThreshold,
    #[error("event log event `{event_id}` {object_name}.{field_name} must be a non-empty string")]
    InvalidEventLogField {
        event_id: String,
        object_name: &'static str,
        field_name: &'static str,
    },
}

#[derive(Debug, Error)]
pub enum TimelineProjectionError {
    #[error(transparent)]
    Runner(#[from] ProjectionRunnerError),

    #[error(transparent)]
    Timeline(#[from] TimelineEngineError),
}
