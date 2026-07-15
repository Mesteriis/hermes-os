use super::router::ApplicationComponents;
use hermes_communications_postgres::runtime_leases::ProviderRuntimeLeaseStore;
use hermes_desktop_runtime::{
    NoopRuntimeLifecycleObserver, RuntimeShutdownConfig, RuntimeSupervisor,
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[path = "runtime_task_catalog.rs"]
mod runtime_task_catalog;

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
    lease_pool: Option<PgPool>,
}

impl ApplicationRuntime {
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
        if let Some(pool) = self.lease_pool.take()
            && let Err(error) = revoke_runtime_leases(&pool).await
        {
            tracing::warn!(error = %error, "provider runtime lease revoke failed during shutdown");
        }
        self.fatal_runtime_error
            .lock()
            .ok()
            .and_then(|error| error.clone())
    }
}

pub(crate) fn start_application_runtime(components: &ApplicationComponents) -> ApplicationRuntime {
    let lease = RuntimeLease(Arc::new(RuntimeLeaseState {
        shutdown: CancellationToken::new(),
    }));
    let termination = CancellationToken::new();
    let fatal_runtime_error = Arc::new(Mutex::new(None));
    let mut runtime_tasks = runtime_task_catalog::collect(components);
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
        let lease_pool = components.state.database.pool().cloned();
        tokio::spawn(async move {
            if let Some(pool) = lease_pool
                && let Err(error) = acquire_in_process_leases(&pool).await
            {
                tracing::error!(error = %error, "provider runtime lease acquisition failed");
                if let Ok(mut failure) = fatal_runtime_error.lock() {
                    *failure = Some(error);
                }
                termination.cancel();
                return;
            }
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
        lease_pool: components.state.database.pool().cloned(),
    }
}

async fn acquire_in_process_leases(pool: &sqlx::PgPool) -> Result<(), String> {
    let accounts = sqlx::query_as::<_, (String, String, Value)>(
        "SELECT account_id, provider_kind, config FROM communication_provider_accounts",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;
    let leases = ProviderRuntimeLeaseStore::new(pool.clone());
    for (account_id, provider, config) in accounts {
        let topology = config
            .get("runtime_topology")
            .and_then(Value::as_str)
            .filter(|value| {
                matches!(
                    *value,
                    "in_process" | "shared_connector" | "per_account_connector"
                )
            })
            .unwrap_or("in_process");
        leases
            .acquire(
                provider.as_str(),
                account_id.as_str(),
                topology,
                "hermes-hub-backend",
                chrono::Duration::hours(1),
            )
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

async fn revoke_runtime_leases(pool: &PgPool) -> Result<(), String> {
    let accounts = sqlx::query_as::<_, (String, String)>(
        "SELECT account_id, provider_kind FROM communication_provider_accounts",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;
    let leases = ProviderRuntimeLeaseStore::new(pool.clone());
    for (account_id, provider) in accounts {
        leases
            .revoke(provider.as_str(), account_id.as_str(), "hermes-hub-backend")
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
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
