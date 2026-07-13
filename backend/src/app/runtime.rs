use super::router::ApplicationComponents;
use hermes_desktop_runtime::{
    NoopRuntimeLifecycleObserver, RuntimeShutdownConfig, RuntimeSupervisor,
};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub(crate) struct RuntimeLease(Arc<RuntimeLeaseState>);

struct RuntimeLeaseState {
    shutdown: CancellationToken,
}

impl RuntimeLease {
    fn cancellation(&self) -> CancellationToken {
        self.0.shutdown.clone()
    }
}

impl Drop for RuntimeLeaseState {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

pub(crate) struct ApplicationRuntime {
    lease: RuntimeLease,
    termination: CancellationToken,
    fatal_runtime_error: Arc<Mutex<Option<String>>>,
    provider_supervisor: Option<JoinHandle<()>>,
}

impl ApplicationRuntime {
    pub(crate) fn lease(&self) -> RuntimeLease {
        self.lease.clone()
    }

    pub(crate) fn termination_signal(&self) -> CancellationToken {
        self.termination.clone()
    }

    pub(crate) async fn shutdown(mut self) -> Option<String> {
        self.lease.cancellation().cancel();
        if let Some(supervisor) = self.provider_supervisor.take()
            && let Err(error) = supervisor.await
        {
            tracing::warn!(error = %error, "provider runtime supervisor join failed during shutdown");
        }
        self.fatal_runtime_error
            .lock()
            .ok()
            .and_then(|error| error.clone())
    }
}

pub(crate) fn start_application_runtime(components: &ApplicationComponents) -> ApplicationRuntime {
    crate::application::bootstrap::start_background_services(components.bootstrap.clone());

    let lease = RuntimeLease(Arc::new(RuntimeLeaseState {
        shutdown: CancellationToken::new(),
    }));
    let termination = CancellationToken::new();
    let fatal_runtime_error = Arc::new(Mutex::new(None));
    let mut runtime_tasks =
        crate::app::vault_reconciliation::lifecycle::host_vault_manifest_reconciliation_task(
            &components.state,
        )
        .into_iter()
        .collect::<Vec<_>>();
    runtime_tasks.extend(crate::application::bootstrap::zulip::runtime_task_specs(
        components.bootstrap.clone(),
    ));
    runtime_tasks.extend(crate::application::bootstrap::mail::runtime_task_specs(
        components.bootstrap.clone(),
    ));
    runtime_tasks.extend(crate::application::bootstrap::whatsapp_runtime_task_specs(
        components.bootstrap.clone(),
    ));
    runtime_tasks.extend(crate::application::bootstrap::telegram::runtime_task_specs(
        components.bootstrap.clone(),
    ));
    runtime_tasks.extend(crate::application::bootstrap::zoom_runtime_task_specs(
        components.bootstrap.clone(),
    ));
    runtime_tasks.extend(
        crate::application::bootstrap::yandex_telemost_runtime_task_specs(
            components.bootstrap.clone(),
        ),
    );
    runtime_tasks.extend(crate::application::bootstrap::core_runtime_task_specs(
        components.bootstrap.clone(),
    ));
    let lifecycle_runtime = crate::app::runtime_lifecycle::build_lifecycle_projection_runtime(
        &runtime_tasks,
        components.state.database.pool().cloned(),
    );
    if let Some(lifecycle_runtime) = lifecycle_runtime.as_ref() {
        runtime_tasks.push(lifecycle_runtime.task.clone());
    }
    let lifecycle_observer = lifecycle_runtime
        .map(|lifecycle_runtime| lifecycle_runtime.observer)
        .unwrap_or_else(|| std::sync::Arc::new(NoopRuntimeLifecycleObserver));
    let provider_supervisor = RuntimeSupervisor::new(
        runtime_tasks,
        RuntimeShutdownConfig::default(),
        lifecycle_observer,
    )
    .map(|supervisor| {
        let shutdown_signal = lease.cancellation();
        let termination = termination.clone();
        let fatal_runtime_error = Arc::clone(&fatal_runtime_error);
        tokio::spawn(async move {
            if let Err(error) = supervisor.run_until(shutdown_signal.cancelled()).await {
                tracing::error!(error = %error, "provider runtime supervisor stopped unexpectedly");
                if let Ok(mut failure) = fatal_runtime_error.lock() {
                    *failure = Some(error.to_string());
                }
                termination.cancel();
            }
        })
    })
    .map_err(|error| {
        tracing::error!(error = %error, "failed to construct provider runtime supervisor");
    })
    .ok();

    ApplicationRuntime {
        lease,
        termination,
        fatal_runtime_error,
        provider_supervisor,
    }
}

#[cfg(test)]
mod tests {
    use super::{Arc, CancellationToken, RuntimeLease, RuntimeLeaseState};

    #[test]
    fn runtime_lease_cancels_only_after_the_last_clone_drops() {
        let lease = RuntimeLease(Arc::new(RuntimeLeaseState {
            shutdown: CancellationToken::new(),
        }));
        let cancellation = lease.cancellation();
        let retained = lease.clone();

        drop(lease);
        assert!(!cancellation.is_cancelled());

        drop(retained);
        assert!(cancellation.is_cancelled());
    }
}
