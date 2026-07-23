//! Owns one active managed-child worker and its staged launch cleanup.

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeEventCredentialHandler, ManagedRuntimeExpectation,
    ManagedRuntimeProviderCredentialHandler, ManagedRuntimeBlobSessionHandler, ManagedRuntimeRelayRequest,
    ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;
use crate::runtime::managed::supervisor as managed_child_supervisor;

use super::Inner;

pub(super) struct ActiveWorker {
    pub(super) join: JoinHandle<()>,
    pub(super) relay: SyncSender<ManagedRuntimeRelayRequest>,
    pub(super) ready: Mutex<Option<Receiver<Result<(), String>>>>,
    pub(super) stop_requested: Arc<AtomicBool>,
}

pub(super) struct ActiveWorkerInput {
    pub(super) inner: Arc<Inner>,
    pub(super) registration_id: String,
    pub(super) staged_executable: StagedNativeArtifact,
    pub(super) arguments: Vec<String>,
    pub(super) expectation: ManagedRuntimeExpectation,
    pub(super) policy: ManagedChildExecutionPolicy,
    pub(super) contracts: Option<StagedRuntimeContracts>,
    pub(super) cleanup: Option<Box<dyn FnOnce() + Send>>,
    pub(super) vault_route_handler: Option<Arc<dyn ManagedRuntimeVaultRouteHandler>>,
    pub(super) event_credential_handler: Option<Arc<dyn ManagedRuntimeEventCredentialHandler>>,
    pub(super) provider_credential_handler:
        Option<Arc<dyn ManagedRuntimeProviderCredentialHandler>>,
    pub(super) blob_session_handler: Option<Arc<dyn ManagedRuntimeBlobSessionHandler>>,
}

pub(super) fn new_active_worker(input: ActiveWorkerInput) -> ActiveWorker {
    let ActiveWorkerInput {
        inner,
        registration_id,
        staged_executable,
        arguments,
        expectation,
        policy,
        contracts,
        cleanup,
        vault_route_handler,
        event_credential_handler,
        provider_credential_handler,
        blob_session_handler,
    } = input;
    let shutdown_requested = Arc::clone(&inner.shutdown_requested);
    let stop_requested = Arc::new(AtomicBool::new(false));
    let worker_stop_requested = Arc::clone(&stop_requested);
    let (relay, relay_requests) = mpsc::sync_channel(64);
    let (ready_sender, ready) = mpsc::sync_channel(1);
    let join = std::thread::spawn(move || {
        record_worker_result(
            &inner,
            &registration_id,
            managed_child_supervisor::run_until_shutdown(
                managed_child_supervisor::ManagedChildRunInput {
                    staged_executable: &staged_executable,
                    arguments: &arguments,
                    expectation: &expectation,
                    policy: &policy,
                    shutdown_requested: &shutdown_requested,
                    stop_requested: &worker_stop_requested,
                    relay_requests: &relay_requests,
                    vault_route_handler: vault_route_handler.as_deref(),
                    event_credential_handler: event_credential_handler.as_deref(),
                    provider_credential_handler: provider_credential_handler.as_deref(),
                    blob_session_handler: blob_session_handler.as_deref(),
                    ready_sender: &ready_sender,
                },
            )
            .map(|_| ()),
        );
        remove_staged_launch(staged_executable, contracts, cleanup);
    });
    ActiveWorker {
        join,
        relay,
        ready: Mutex::new(Some(ready)),
        stop_requested,
    }
}

pub(super) fn remove_staged_launch(
    staged_executable: StagedNativeArtifact,
    contracts: Option<StagedRuntimeContracts>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
) {
    let _ = staged_executable.remove();
    if let Some(contracts) = contracts {
        let _ = contracts.remove();
    }
    if let Some(cleanup) = cleanup {
        cleanup();
    }
}

fn record_worker_result(inner: &Inner, registration_id: &str, result: Result<(), String>) {
    if let Err(error) = result {
        if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
            eprintln!("developer_managed_runtime_failed process={registration_id} error={error}");
        }
        let _ = inner
            .failures
            .lock()
            .map(|mut failures| failures.insert(registration_id.to_owned(), error));
    }
}
