use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeLifecycleState {
    Running,
    Error,
    Stopped,
}

impl RuntimeLifecycleState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Error => "error",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeLifecycleUpdate {
    pub source_code: String,
    pub runtime_kind: String,
    pub task_name: String,
    pub state: RuntimeLifecycleState,
    pub error_code: Option<String>,
    pub metadata: Value,
}

impl RuntimeLifecycleUpdate {
    pub fn running(
        source_code: impl Into<String>,
        runtime_kind: impl Into<String>,
        task_name: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            source_code: source_code.into(),
            runtime_kind: runtime_kind.into(),
            task_name: task_name.into(),
            state: RuntimeLifecycleState::Running,
            error_code: None,
            metadata,
        }
    }

    pub fn degraded(
        source_code: impl Into<String>,
        runtime_kind: impl Into<String>,
        task_name: impl Into<String>,
        error_code: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            source_code: source_code.into(),
            runtime_kind: runtime_kind.into(),
            task_name: task_name.into(),
            state: RuntimeLifecycleState::Error,
            error_code: Some(error_code.into()),
            metadata,
        }
    }

    pub fn stopped(
        source_code: impl Into<String>,
        runtime_kind: impl Into<String>,
        task_name: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            source_code: source_code.into(),
            runtime_kind: runtime_kind.into(),
            task_name: task_name.into(),
            state: RuntimeLifecycleState::Stopped,
            error_code: None,
            metadata,
        }
    }
}

#[derive(Debug, Error)]
#[error("signal hub runtime lifecycle persistence failed: {0}")]
pub struct RuntimeLifecyclePortError(pub String);

impl RuntimeLifecyclePortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub type RuntimeLifecyclePortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RuntimeLifecyclePortError>> + Send + 'a>>;

/// Durable Signal Hub projection for supervised runtime lifecycle transitions.
pub trait RuntimeLifecyclePort: Send + Sync {
    fn record_runtime_lifecycle<'a>(
        &'a self,
        update: &'a RuntimeLifecycleUpdate,
    ) -> RuntimeLifecyclePortFuture<'a, ()>;
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{RuntimeLifecycleState, RuntimeLifecycleUpdate};

    #[test]
    fn degraded_update_keeps_only_redacted_error_code() {
        let update = RuntimeLifecycleUpdate::degraded(
            "zulip",
            "zulip_event_ingest",
            "zulip-event-ingest",
            "provider_unavailable",
            json!({"class": "background"}),
        );

        assert_eq!(update.state, RuntimeLifecycleState::Error);
        assert_eq!(update.error_code.as_deref(), Some("provider_unavailable"));
    }
}
