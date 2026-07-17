//! Coordinates all private control-plane sockets in the single Kernel process.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::identity::owner_control;
use crate::modules::registration::ipc as registration_ipc;
use crate::recovery;
use crate::runtime::external::ipc as external_session_ipc;
use crate::runtime::lifecycle::shutdown;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const EXIT_POLL: Duration = Duration::from_millis(25);

pub fn serve(
    store: SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    store_path: &Path,
) -> Result<(), String> {
    let shutdown_requested = shutdown::install()?;
    let managed_runtime_supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let store = Arc::new(store);
    let (mut workers, receiver) = start_workers(
        &store,
        data_dir,
        runtime_dir,
        store_path,
        &shutdown_requested,
        &managed_runtime_supervisor,
    );
    let failure = supervise_workers(&receiver, &shutdown_requested);
    managed_runtime_supervisor.shutdown()?;
    join_workers(&mut workers, failure)
}

fn start_workers(
    store: &Arc<SqliteControlStore>,
    data_dir: &Path,
    runtime_dir: &Path,
    store_path: &Path,
    shutdown_requested: &Arc<AtomicBool>,
    managed_runtime_supervisor: &ManagedRuntimeSupervisor,
) -> (
    Vec<std::thread::JoinHandle<()>>,
    mpsc::Receiver<Result<(), String>>,
) {
    let (completed, receiver) = mpsc::channel();
    let mut workers = Vec::with_capacity(4);
    let runtime_dir = runtime_dir.to_path_buf();
    let data_dir = data_dir.to_path_buf();
    let store_path = store_path.to_path_buf();
    workers.push(spawn_worker(
        completed.clone(),
        Arc::clone(&shutdown_requested),
        {
            let store = Arc::clone(store);
            let runtime_dir = runtime_dir.clone();
            move |shutdown| {
                recovery::serve_recovery_socket(&runtime_dir, &store_path, Some(store), shutdown)
            }
        },
    ));
    workers.push(start_owner_worker(
        completed.clone(),
        shutdown_requested,
        store,
        data_dir.clone(),
        runtime_dir.clone(),
        managed_runtime_supervisor.clone(),
    ));
    workers.push(spawn_worker(
        completed.clone(),
        Arc::clone(&shutdown_requested),
        {
            let store = Arc::clone(store);
            let runtime_dir = runtime_dir.clone();
            move |shutdown| registration_ipc::serve(store, &runtime_dir, shutdown)
        },
    ));
    workers.push(spawn_external_runtime_worker(
        completed,
        Arc::clone(&shutdown_requested),
        Arc::clone(store),
        data_dir,
        runtime_dir,
        managed_runtime_supervisor.clone(),
    ));
    (workers, receiver)
}

fn start_owner_worker(
    completed: mpsc::Sender<Result<(), String>>,
    shutdown_requested: &Arc<AtomicBool>,
    store: &Arc<SqliteControlStore>,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    supervisor: ManagedRuntimeSupervisor,
) -> std::thread::JoinHandle<()> {
    let store = Arc::clone(store);
    spawn_worker(completed, Arc::clone(shutdown_requested), move |shutdown| {
        owner_control::serve(store, &data_dir, &runtime_dir, shutdown, supervisor)
    })
}

fn spawn_external_runtime_worker(
    completed: mpsc::Sender<Result<(), String>>,
    shutdown_requested: Arc<AtomicBool>,
    store: Arc<SqliteControlStore>,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    managed_runtime_supervisor: ManagedRuntimeSupervisor,
) -> std::thread::JoinHandle<()> {
    spawn_worker(completed, shutdown_requested, move |shutdown| {
        external_session_ipc::serve(
            store,
            &data_dir,
            &runtime_dir,
            shutdown,
            managed_runtime_supervisor,
        )
    })
}

fn supervise_workers(
    receiver: &mpsc::Receiver<Result<(), String>>,
    shutdown_requested: &Arc<AtomicBool>,
) -> Option<String> {
    let mut failure = None;
    while !shutdown_requested.load(Ordering::Acquire) {
        match receiver.recv_timeout(EXIT_POLL) {
            Ok(Ok(())) => {
                failure = Some("a private control-plane socket stopped unexpectedly".to_owned());
                shutdown_requested.store(true, Ordering::Release);
            }
            Ok(Err(error)) => {
                failure = Some(error);
                shutdown_requested.store(true, Ordering::Release);
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

fn spawn_worker<F>(
    completed: mpsc::Sender<Result<(), String>>,
    shutdown_requested: Arc<AtomicBool>,
    serve: F,
) -> std::thread::JoinHandle<()>
where
    F: FnOnce(Arc<AtomicBool>) -> Result<(), String> + Send + 'static,
{
    std::thread::spawn(move || {
        let result = serve(Arc::clone(&shutdown_requested));
        if result.is_err() {
            shutdown_requested.store(true, Ordering::Release);
        }
        let _ = completed.send(result);
    })
}
