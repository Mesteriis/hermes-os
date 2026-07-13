//! Supervised lifecycle primitives for the bundled Hermes desktop runtime.
//!
//! This crate deliberately knows nothing about HTTP, PostgreSQL, providers or
//! vaults. Composition supplies managed tasks and observes lifecycle events.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

pub type RuntimeTaskFuture = Pin<Box<dyn Future<Output = Result<(), RuntimeTaskError>> + Send>>;
pub type RuntimeTaskFactory = Arc<dyn Fn(CancellationToken) -> RuntimeTaskFuture + Send + Sync>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeTaskClass {
    Essential,
    Background,
    Startup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExitPolicy {
    ShutdownRuntime,
    MarkDegraded,
    ExpectedCompletion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeShutdownPhase {
    Quiesce,
    Checkpoint,
    StopProviders,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeLifecycleEvent {
    Started {
        task_name: String,
        class: RuntimeTaskClass,
    },
    Degraded {
        task_name: String,
        error_code: String,
    },
    Stopped {
        task_name: String,
    },
    Aborted {
        task_name: String,
        phase: RuntimeShutdownPhase,
    },
}

pub trait RuntimeLifecycleObserver: Send + Sync {
    fn on_event(&self, event: RuntimeLifecycleEvent);
}

#[derive(Default)]
pub struct NoopRuntimeLifecycleObserver;

impl RuntimeLifecycleObserver for NoopRuntimeLifecycleObserver {
    fn on_event(&self, _: RuntimeLifecycleEvent) {}
}

#[derive(Clone)]
pub struct RuntimeTaskSpec {
    pub name: String,
    pub class: RuntimeTaskClass,
    pub exit_policy: RuntimeExitPolicy,
    /// Opaque owner metadata for composition-level lifecycle projections.
    /// The supervisor does not interpret either field.
    pub lifecycle_target: Option<RuntimeTaskLifecycleTarget>,
    pub task: RuntimeTaskFactory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTaskLifecycleTarget {
    pub source_code: String,
    pub runtime_kind: String,
}

impl RuntimeTaskSpec {
    pub fn new(
        name: impl Into<String>,
        class: RuntimeTaskClass,
        exit_policy: RuntimeExitPolicy,
        task: RuntimeTaskFactory,
    ) -> Self {
        Self {
            name: name.into(),
            class,
            exit_policy,
            lifecycle_target: None,
            task,
        }
    }

    pub fn with_lifecycle_target(
        mut self,
        source_code: impl Into<String>,
        runtime_kind: impl Into<String>,
    ) -> Self {
        self.lifecycle_target = Some(RuntimeTaskLifecycleTarget {
            source_code: source_code.into(),
            runtime_kind: runtime_kind.into(),
        });
        self
    }

    pub fn with_lifecycle_source(self, source_code: impl Into<String>) -> Self {
        let runtime_kind = self.name.clone();
        self.with_lifecycle_target(source_code, runtime_kind)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeShutdownConfig {
    pub quiesce_timeout: Duration,
    pub checkpoint_timeout: Duration,
    pub provider_stop_timeout: Duration,
}

impl Default for RuntimeShutdownConfig {
    fn default() -> Self {
        Self {
            quiesce_timeout: Duration::from_secs(30),
            checkpoint_timeout: Duration::from_secs(5),
            provider_stop_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeShutdownReport {
    pub aborted_tasks: Vec<String>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RuntimeTaskError {
    #[error("{code}")]
    Coded { code: String },
}

impl RuntimeTaskError {
    pub fn code(&self) -> &str {
        match self {
            Self::Coded { code } => code,
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RuntimeSupervisorError {
    #[error("duplicate runtime task name: {0}")]
    DuplicateTaskName(String),
    #[error("essential runtime task {task_name} exited unexpectedly: {error_code}")]
    EssentialTaskExited {
        task_name: String,
        error_code: String,
    },
    #[error("essential runtime task {task_name} panicked")]
    EssentialTaskPanicked { task_name: String },
}

pub struct RuntimeSupervisor {
    tasks: Vec<RuntimeTaskSpec>,
    shutdown: RuntimeShutdownConfig,
    observer: Arc<dyn RuntimeLifecycleObserver>,
}

impl RuntimeSupervisor {
    pub fn new(
        tasks: Vec<RuntimeTaskSpec>,
        shutdown: RuntimeShutdownConfig,
        observer: Arc<dyn RuntimeLifecycleObserver>,
    ) -> Result<Self, RuntimeSupervisorError> {
        let mut names = std::collections::HashSet::new();
        for task in &tasks {
            if !names.insert(task.name.clone()) {
                return Err(RuntimeSupervisorError::DuplicateTaskName(task.name.clone()));
            }
        }
        Ok(Self {
            tasks,
            shutdown,
            observer,
        })
    }

    pub async fn run_until<F>(
        &self,
        shutdown_signal: F,
    ) -> Result<RuntimeShutdownReport, RuntimeSupervisorError>
    where
        F: Future<Output = ()> + Send,
    {
        let cancellation = CancellationToken::new();
        let mut tasks = JoinSet::new();
        let mut running = std::collections::HashSet::new();
        for spec in &self.tasks {
            self.observer.on_event(RuntimeLifecycleEvent::Started {
                task_name: spec.name.clone(),
                class: spec.class,
            });
            let task_name = spec.name.clone();
            let policy = spec.exit_policy;
            let child = cancellation.child_token();
            let task = Arc::clone(&spec.task);
            running.insert(task_name.clone());
            tasks.spawn(async move { (task_name, policy, task(child).await) });
        }

        tokio::pin!(shutdown_signal);
        loop {
            tokio::select! {
                _ = &mut shutdown_signal => {
                    return Ok(self.shutdown(cancellation, &mut tasks, &mut running).await);
                }
                next = tasks.join_next(), if !tasks.is_empty() => {
                    let Some(next) = next else { continue; };
                    match next {
                        Ok((task_name, RuntimeExitPolicy::ExpectedCompletion, Ok(()))) => {
                            running.remove(&task_name);
                            self.observer.on_event(RuntimeLifecycleEvent::Stopped { task_name });
                        }
                        Ok((task_name, RuntimeExitPolicy::ExpectedCompletion, Err(error))) => {
                            running.remove(&task_name);
                            self.observer.on_event(RuntimeLifecycleEvent::Degraded {
                                task_name,
                                error_code: error.code().to_owned(),
                            });
                        }
                        Ok((task_name, RuntimeExitPolicy::MarkDegraded, result)) => {
                            running.remove(&task_name);
                            let error_code = result.err().map(|error| error.code().to_owned())
                                .unwrap_or_else(|| "unexpected_completion".to_owned());
                            self.observer.on_event(RuntimeLifecycleEvent::Degraded { task_name, error_code });
                        }
                        Ok((task_name, RuntimeExitPolicy::ShutdownRuntime, result)) => {
                            running.remove(&task_name);
                            let error_code = result.err().map(|error| error.code().to_owned())
                                .unwrap_or_else(|| "unexpected_completion".to_owned());
                            let _ = self.shutdown(cancellation, &mut tasks, &mut running).await;
                            return Err(RuntimeSupervisorError::EssentialTaskExited { task_name, error_code });
                        }
                        Err(error) => {
                            let _ = self.shutdown(cancellation, &mut tasks, &mut running).await;
                            return Err(RuntimeSupervisorError::EssentialTaskPanicked {
                                task_name: error.id().to_string(),
                            });
                        }
                    }
                }
                else => return Ok(RuntimeShutdownReport::default()),
            }
        }
    }

    async fn shutdown(
        &self,
        cancellation: CancellationToken,
        tasks: &mut JoinSet<(String, RuntimeExitPolicy, Result<(), RuntimeTaskError>)>,
        running: &mut std::collections::HashSet<String>,
    ) -> RuntimeShutdownReport {
        cancellation.cancel();
        let mut report = RuntimeShutdownReport::default();
        for (_phase, duration) in [
            (RuntimeShutdownPhase::Quiesce, self.shutdown.quiesce_timeout),
            (
                RuntimeShutdownPhase::Checkpoint,
                self.shutdown.checkpoint_timeout,
            ),
            (
                RuntimeShutdownPhase::StopProviders,
                self.shutdown.provider_stop_timeout,
            ),
        ] {
            if tasks.is_empty() {
                break;
            }
            let observer = Arc::clone(&self.observer);
            let _ = timeout(duration, async {
                while let Some(result) = tasks.join_next().await {
                    if let Ok((task_name, _, _)) = result {
                        running.remove(&task_name);
                        observer.on_event(RuntimeLifecycleEvent::Stopped { task_name });
                    }
                }
            })
            .await;
        }
        if !tasks.is_empty() {
            tasks.abort_all();
            for task_name in std::mem::take(running) {
                report.aborted_tasks.push(task_name.clone());
                self.observer.on_event(RuntimeLifecycleEvent::Aborted {
                    task_name,
                    phase: RuntimeShutdownPhase::StopProviders,
                });
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingObserver(Mutex<Vec<RuntimeLifecycleEvent>>);

    impl RuntimeLifecycleObserver for RecordingObserver {
        fn on_event(&self, event: RuntimeLifecycleEvent) {
            self.0.lock().expect("observer lock").push(event);
        }
    }

    fn supervisor(
        tasks: Vec<RuntimeTaskSpec>,
        observer: Arc<RecordingObserver>,
    ) -> RuntimeSupervisor {
        RuntimeSupervisor::new(
            tasks,
            RuntimeShutdownConfig {
                quiesce_timeout: Duration::from_millis(20),
                checkpoint_timeout: Duration::from_millis(20),
                provider_stop_timeout: Duration::from_millis(20),
            },
            observer,
        )
        .expect("supervisor")
    }

    #[test]
    fn rejects_duplicate_task_names() {
        let task: RuntimeTaskFactory =
            Arc::new(|_: CancellationToken| Box::pin(async { Ok(()) }) as RuntimeTaskFuture);
        let result = RuntimeSupervisor::new(
            vec![
                RuntimeTaskSpec::new(
                    "same",
                    RuntimeTaskClass::Background,
                    RuntimeExitPolicy::MarkDegraded,
                    Arc::clone(&task),
                ),
                RuntimeTaskSpec::new(
                    "same",
                    RuntimeTaskClass::Background,
                    RuntimeExitPolicy::MarkDegraded,
                    task,
                ),
            ],
            RuntimeShutdownConfig::default(),
            Arc::new(NoopRuntimeLifecycleObserver),
        );

        match result {
            Err(error) => assert_eq!(
                error,
                RuntimeSupervisorError::DuplicateTaskName("same".to_owned())
            ),
            Ok(_) => panic!("duplicate task names must fail"),
        }
    }

    #[test]
    fn lifecycle_source_uses_declared_task_name_as_runtime_kind() {
        let task: RuntimeTaskFactory =
            Arc::new(|_: CancellationToken| Box::pin(async { Ok(()) }) as RuntimeTaskFuture);
        let spec = RuntimeTaskSpec::new(
            "zulip_event_ingest",
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        )
        .with_lifecycle_source("zulip");

        assert_eq!(
            spec.lifecycle_target,
            Some(RuntimeTaskLifecycleTarget {
                source_code: "zulip".to_owned(),
                runtime_kind: "zulip_event_ingest".to_owned(),
            })
        );
    }

    #[tokio::test]
    async fn background_failure_degrades_without_restart() {
        let observer = Arc::new(RecordingObserver::default());
        let task = Arc::new(|_: CancellationToken| {
            Box::pin(async {
                Err(RuntimeTaskError::Coded {
                    code: "provider_unavailable".to_owned(),
                })
            }) as RuntimeTaskFuture
        });
        let report = supervisor(
            vec![RuntimeTaskSpec::new(
                "zulip",
                RuntimeTaskClass::Background,
                RuntimeExitPolicy::MarkDegraded,
                task,
            )],
            Arc::clone(&observer),
        )
        .run_until(async { tokio::time::sleep(Duration::from_millis(10)).await })
        .await
        .expect("background failure must not terminate the runtime");

        assert!(report.aborted_tasks.is_empty());
        assert!(observer.0.lock().expect("observer lock").contains(
            &RuntimeLifecycleEvent::Degraded {
                task_name: "zulip".to_owned(),
                error_code: "provider_unavailable".to_owned(),
            }
        ));
    }

    #[tokio::test]
    async fn essential_failure_stops_the_runtime() {
        let observer = Arc::new(RecordingObserver::default());
        let task = Arc::new(|_: CancellationToken| {
            Box::pin(async {
                Err(RuntimeTaskError::Coded {
                    code: "event_spine_failed".to_owned(),
                })
            }) as RuntimeTaskFuture
        });
        let error = supervisor(
            vec![RuntimeTaskSpec::new(
                "event-spine",
                RuntimeTaskClass::Essential,
                RuntimeExitPolicy::ShutdownRuntime,
                task,
            )],
            observer,
        )
        .run_until(std::future::pending())
        .await
        .expect_err("essential failure must terminate the runtime");

        assert_eq!(
            error,
            RuntimeSupervisorError::EssentialTaskExited {
                task_name: "event-spine".to_owned(),
                error_code: "event_spine_failed".to_owned(),
            }
        );
    }
}
