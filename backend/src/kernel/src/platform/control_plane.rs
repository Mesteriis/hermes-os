//! Coordinates all private control-plane sockets in the single Kernel process.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::identity::owner_control;
use crate::modules::registration::ipc as registration_ipc;
use crate::platform::blob::session::BlobSessionHandlerV1;
use crate::platform::events::credential::{
    authority::EVENTS_AUTHORITY_REGISTRATION_ID, handler::EventCredentialHandlerV1,
};
use crate::platform::gateway::{self, BrowserGatewayConfigurationV1, BrowserPairingAdmissionV1};
use crate::platform::scheduler::lifecycle as scheduler_lifecycle;
use crate::platform::vault::managed_route::KernelManagedVaultRouteHandler;
use crate::platform::vault::owner_derived_key::OwnerDerivedKeyHandlerV1;
use crate::platform::vault::provider_credential::ProviderCredentialHandlerV1;
use crate::recovery;
use crate::runtime::external::ipc as external_session_ipc;
use crate::runtime::lifecycle::shutdown;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

#[path = "control_plane/worker.rs"]
mod worker;

use worker::{WorkerClassV1, WorkerCompletionV1, spawn as spawn_worker};

const EXIT_POLL: Duration = Duration::from_millis(25);

pub fn serve(
    store: SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    store_path: &Path,
    browser_gateway: Option<BrowserGatewayConfigurationV1>,
) -> Result<(), String> {
    let shutdown_requested = shutdown::install()?;
    let managed_runtime_supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let store = Arc::new(store);
    configure_runtime(&managed_runtime_supervisor, &store, data_dir)?;
    start_development_foundation(
        &managed_runtime_supervisor,
        &store,
        data_dir,
        runtime_dir,
        browser_gateway.as_ref(),
    )?;
    let browser_pairing = browser_pairing(
        &store,
        &managed_runtime_supervisor,
        browser_gateway.as_ref(),
    )?;
    let (mut workers, receiver) = start_workers(ControlPlaneWorkerInputV1 {
        store,
        data_dir: data_dir.to_path_buf(),
        runtime_dir: runtime_dir.to_path_buf(),
        store_path: store_path.to_path_buf(),
        shutdown_requested: Arc::clone(&shutdown_requested),
        managed_runtime_supervisor: managed_runtime_supervisor.clone(),
        browser_gateway,
        browser_pairing,
    });
    let failure = supervise_workers(&receiver, &shutdown_requested);
    managed_runtime_supervisor.shutdown()?;
    join_workers(&mut workers, failure)
}

fn configure_runtime(
    managed_runtime_supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data_dir: &Path,
) -> Result<(), String> {
    let vault_route_handler: Arc<KernelManagedVaultRouteHandler> =
        Arc::new(KernelManagedVaultRouteHandler::new(
            Arc::clone(store),
            data_dir,
            Arc::new(managed_runtime_supervisor.relay_port()),
        ));
    managed_runtime_supervisor.configure_vault_route_handler(vault_route_handler.clone())?;
    managed_runtime_supervisor.configure_provider_credential_handler(Arc::new(
        ProviderCredentialHandlerV1::new(
            Arc::clone(store),
            managed_runtime_supervisor.relay_port(),
            Arc::clone(&vault_route_handler),
        ),
    ))?;
    managed_runtime_supervisor.configure_owner_derived_key_handler(Arc::new(
        OwnerDerivedKeyHandlerV1::new(
            Arc::clone(store),
            managed_runtime_supervisor.relay_port(),
            vault_route_handler,
        ),
    ))?;
    managed_runtime_supervisor.configure_blob_session_handler(Arc::new(
        BlobSessionHandlerV1::new(
            Arc::clone(store),
            managed_runtime_supervisor.relay_port(),
            data_dir.to_path_buf(),
        ),
    ))?;
    managed_runtime_supervisor.configure_event_credential_handler(Arc::new(
        EventCredentialHandlerV1::new(
            Arc::clone(store),
            EVENTS_AUTHORITY_REGISTRATION_ID.to_owned(),
            managed_runtime_supervisor.relay_port(),
        )?,
    ))?;
    Ok(())
}

fn start_development_foundation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data_dir: &Path,
    runtime_dir: &Path,
    browser_gateway: Option<&BrowserGatewayConfigurationV1>,
) -> Result<(), String> {
    if !browser_gateway.is_some_and(BrowserGatewayConfigurationV1::is_lan_development) {
        return Ok(());
    }
    crate::platform::development::start_local_foundation(supervisor, store, data_dir, runtime_dir)
        .map_err(|error| match supervisor.shutdown() {
            Ok(()) => error,
            Err(cleanup_error) => format!(
                "{error}; managed runtime cleanup after developer bootstrap failure also failed: {cleanup_error}"
            ),
        })
}

fn browser_pairing(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    browser_gateway: Option<&BrowserGatewayConfigurationV1>,
) -> Result<Option<Arc<BrowserPairingAdmissionV1>>, String> {
    browser_gateway
        .filter(|configuration| !configuration.is_lan_development())
        .map(|configuration| {
            BrowserPairingAdmissionV1::new(Arc::clone(store), supervisor.clone(), configuration)
        })
        .transpose()
        .map(|pairing| pairing.map(Arc::new))
}

struct ControlPlaneWorkerInputV1 {
    store: Arc<SqliteControlStore>,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    store_path: std::path::PathBuf,
    shutdown_requested: Arc<AtomicBool>,
    managed_runtime_supervisor: ManagedRuntimeSupervisor,
    browser_gateway: Option<BrowserGatewayConfigurationV1>,
    browser_pairing: Option<Arc<BrowserPairingAdmissionV1>>,
}

fn start_workers(
    input: ControlPlaneWorkerInputV1,
) -> (
    Vec<std::thread::JoinHandle<()>>,
    mpsc::Receiver<WorkerCompletionV1>,
) {
    let ControlPlaneWorkerInputV1 {
        store,
        data_dir,
        runtime_dir,
        store_path,
        shutdown_requested,
        managed_runtime_supervisor,
        browser_gateway,
        browser_pairing,
    } = input;
    let (completed, receiver) = mpsc::channel::<WorkerCompletionV1>();
    let mut workers = Vec::with_capacity(6);
    workers.extend(start_boot_workers(BootWorkerInputV1 {
        completed: completed.clone(),
        shutdown_requested: Arc::clone(&shutdown_requested),
        store: Arc::clone(&store),
        data_dir: data_dir.clone(),
        runtime_dir: runtime_dir.clone(),
        store_path,
        supervisor: managed_runtime_supervisor.clone(),
        browser_pairing: browser_pairing.clone(),
    }));
    workers.push(start_registration_worker(
        completed.clone(),
        &shutdown_requested,
        Arc::clone(&store),
        runtime_dir.clone(),
    ));
    workers.push(spawn_external_runtime_worker(
        completed.clone(),
        Arc::clone(&shutdown_requested),
        Arc::clone(&store),
        data_dir,
        runtime_dir.clone(),
        managed_runtime_supervisor.clone(),
    ));
    workers.push(start_scheduler_worker(
        completed.clone(),
        &shutdown_requested,
        Arc::clone(&store),
        runtime_dir,
        managed_runtime_supervisor.clone(),
    ));
    workers.extend(browser_gateway_worker(
        &completed,
        &shutdown_requested,
        &store,
        managed_runtime_supervisor,
        browser_gateway,
        browser_pairing,
    ));
    (workers, receiver)
}

struct BootWorkerInputV1 {
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    store_path: std::path::PathBuf,
    supervisor: ManagedRuntimeSupervisor,
    browser_pairing: Option<Arc<BrowserPairingAdmissionV1>>,
}

fn start_boot_workers(input: BootWorkerInputV1) -> Vec<std::thread::JoinHandle<()>> {
    let BootWorkerInputV1 {
        completed,
        shutdown_requested,
        store,
        data_dir,
        runtime_dir,
        store_path,
        supervisor,
        browser_pairing,
    } = input;
    vec![
        start_recovery_worker(
            completed.clone(),
            &shutdown_requested,
            Arc::clone(&store),
            runtime_dir.clone(),
            store_path,
        ),
        start_owner_worker(
            completed.clone(),
            &shutdown_requested,
            &store,
            data_dir,
            runtime_dir,
            supervisor,
            browser_pairing,
        ),
    ]
}

fn start_scheduler_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    runtime_dir: std::path::PathBuf,
    supervisor: ManagedRuntimeSupervisor,
) -> std::thread::JoinHandle<()> {
    spawn_scheduler_lifecycle_worker(
        completed,
        Arc::clone(shutdown_requested),
        store,
        runtime_dir,
        supervisor,
    )
}

fn browser_gateway_worker(
    completed: &mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
    store: &Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    browser_gateway: Option<BrowserGatewayConfigurationV1>,
    browser_pairing: Option<Arc<BrowserPairingAdmissionV1>>,
) -> Option<std::thread::JoinHandle<()>> {
    let configuration = browser_gateway?;
    Some(start_browser_gateway_worker(
        completed.clone(),
        shutdown_requested,
        Arc::clone(store),
        supervisor,
        configuration,
        browser_pairing,
    ))
}

fn start_registration_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    runtime_dir: std::path::PathBuf,
) -> std::thread::JoinHandle<()> {
    spawn_worker(
        completed,
        WorkerClassV1::Critical,
        "registration",
        Arc::clone(shutdown_requested),
        move |shutdown| registration_ipc::serve(Arc::clone(&store), &runtime_dir, shutdown),
    )
}

fn start_recovery_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    runtime_dir: std::path::PathBuf,
    store_path: std::path::PathBuf,
) -> std::thread::JoinHandle<()> {
    spawn_worker(
        completed,
        WorkerClassV1::Critical,
        "recovery",
        Arc::clone(shutdown_requested),
        move |shutdown| {
            recovery::serve_recovery_socket(
                &runtime_dir,
                &store_path,
                Some(Arc::clone(&store)),
                shutdown,
            )
        },
    )
}

fn start_browser_gateway_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    configuration: BrowserGatewayConfigurationV1,
    pairing: Option<Arc<BrowserPairingAdmissionV1>>,
) -> std::thread::JoinHandle<()> {
    spawn_worker(
        completed,
        WorkerClassV1::Restartable,
        "browser_gateway",
        Arc::clone(shutdown_requested),
        move |shutdown| {
            gateway::serve(
                Arc::clone(&store),
                supervisor.clone(),
                configuration.clone(),
                pairing.clone(),
                shutdown,
            )
        },
    )
}

fn start_owner_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
    store: &Arc<SqliteControlStore>,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    supervisor: ManagedRuntimeSupervisor,
    browser_pairing: Option<Arc<BrowserPairingAdmissionV1>>,
) -> std::thread::JoinHandle<()> {
    let store = Arc::clone(store);
    spawn_worker(
        completed,
        WorkerClassV1::Critical,
        "owner",
        Arc::clone(shutdown_requested),
        move |shutdown| {
            owner_control::serve(
                Arc::clone(&store),
                &data_dir,
                &runtime_dir,
                shutdown,
                supervisor.clone(),
                browser_pairing.clone(),
            )
        },
    )
}

fn spawn_external_runtime_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    managed_runtime_supervisor: ManagedRuntimeSupervisor,
) -> std::thread::JoinHandle<()> {
    spawn_worker(
        completed,
        WorkerClassV1::Restartable,
        "external_runtime",
        shutdown_requested,
        move |shutdown| {
            external_session_ipc::serve(
                Arc::clone(&store),
                &data_dir,
                &runtime_dir,
                shutdown,
                managed_runtime_supervisor.clone(),
            )
        },
    )
}

fn spawn_scheduler_lifecycle_worker(
    completed: mpsc::Sender<WorkerCompletionV1>,
    shutdown_requested: Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    runtime_dir: std::path::PathBuf,
    managed_runtime_supervisor: ManagedRuntimeSupervisor,
) -> std::thread::JoinHandle<()> {
    spawn_worker(
        completed,
        WorkerClassV1::Restartable,
        "scheduler_lifecycle",
        shutdown_requested,
        move |shutdown| {
            let kernel = std::env::current_exe()
                .map_err(|_| "Kernel executable path is unavailable".to_owned())?;
            scheduler_lifecycle::serve(
                Arc::clone(&store),
                &kernel,
                &runtime_dir,
                shutdown,
                managed_runtime_supervisor.clone(),
            )
        },
    )
}

fn supervise_workers(
    receiver: &mpsc::Receiver<WorkerCompletionV1>,
    shutdown_requested: &Arc<AtomicBool>,
) -> Option<String> {
    let mut failure = None;
    while !shutdown_requested.load(Ordering::Acquire) {
        match receiver.recv_timeout(EXIT_POLL) {
            Ok(completion) if completion.class == WorkerClassV1::Critical => {
                failure = Some(completion.result.err().unwrap_or_else(|| {
                    "a critical control-plane worker stopped unexpectedly".to_owned()
                }));
                shutdown_requested.store(true, Ordering::Release);
            }
            Ok(completion) => {
                if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                    let result = completion
                        .result
                        .err()
                        .unwrap_or_else(|| "stopped".to_owned());
                    eprintln!(
                        "developer_control_plane_restarting worker={} error={result}",
                        completion.label
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                failure =
                    Some("private control-plane workers disconnected unexpectedly".to_owned());
                shutdown_requested.store(true, Ordering::Release);
            }
        }
    }

    failure
}

fn join_workers(
    workers: &mut Vec<std::thread::JoinHandle<()>>,
    mut failure: Option<String>,
) -> Result<(), String> {
    for worker in workers.drain(..) {
        if worker.join().is_err() && failure.is_none() {
            failure = Some("a private control-plane worker panicked".to_owned());
        }
    }
    failure.map_or(Ok(()), Err)
}
