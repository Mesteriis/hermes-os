//! Owns active managed-child workers for one Kernel process.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::runtime::lifecycle::control::{ManagedRuntimeExpectation, ManagedRuntimeRelayRequest};
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;
use crate::runtime::managed::supervisor as managed_child_supervisor;

#[derive(Clone)]
pub struct ManagedRuntimeSupervisor {
    inner: Arc<Inner>,
}

struct Inner {
    shutdown_requested: Arc<AtomicBool>,
    workers: Mutex<HashMap<String, ActiveWorker>>,
}

struct ActiveWorker {
    join: JoinHandle<()>,
    relay: SyncSender<ManagedRuntimeRelayRequest>,
    stop_requested: Arc<AtomicBool>,
}

impl ManagedRuntimeSupervisor {
    #[must_use]
    pub fn new(shutdown_requested: Arc<AtomicBool>) -> Self {
        Self {
            inner: Arc::new(Inner {
                shutdown_requested,
                workers: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn start(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
    ) -> Result<(), String> {
        self.start_with_arguments(
            registration_id,
            staged_executable,
            Vec::new(),
            expectation,
            policy,
        )
    }

    pub fn start_with_arguments(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        arguments: Vec<String>,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
    ) -> Result<(), String> {
        self.start_with_optional_contracts(
            registration_id,
            staged_executable,
            arguments,
            expectation,
            policy,
            None,
        )
    }

    pub fn start_with_arguments_and_contracts(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        arguments: Vec<String>,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
        contracts: StagedRuntimeContracts,
    ) -> Result<(), String> {
        self.start_with_optional_contracts(
            registration_id,
            staged_executable,
            arguments,
            expectation,
            policy,
            Some(contracts),
        )
    }

    fn start_with_optional_contracts(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        arguments: Vec<String>,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
        contracts: Option<StagedRuntimeContracts>,
    ) -> Result<(), String> {
        self.reap_finished();
        if self.inner.shutdown_requested.load(Ordering::Acquire) {
            remove_staged_launch(staged_executable, contracts);
            return Err("managed runtime supervisor is shutting down".to_owned());
        }
        let mut workers = match self.inner.workers.lock() {
            Ok(workers) => workers,
            Err(_) => {
                remove_staged_launch(staged_executable, contracts);
                return Err("managed runtime supervisor state is unavailable".to_owned());
            }
        };
        if workers.contains_key(&registration_id) {
            drop(workers);
            remove_staged_launch(staged_executable, contracts);
            return Err("managed runtime is already active for this registration".to_owned());
        }
        let shutdown_requested = Arc::clone(&self.inner.shutdown_requested);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker_stop_requested = Arc::clone(&stop_requested);
        let (relay_sender, relay_receiver) = mpsc::sync_channel(64);
        let worker = std::thread::spawn(move || {
            let _ = managed_child_supervisor::run_until_shutdown(
                &staged_executable,
                &arguments,
                &expectation,
                &policy,
                &shutdown_requested,
                &worker_stop_requested,
                &relay_receiver,
            );
            let _ = staged_executable.remove();
            if let Some(contracts) = contracts {
                let _ = contracts.remove();
            }
        });
        workers.insert(
            registration_id,
            ActiveWorker {
                join: worker,
                relay: relay_sender,
                stop_requested,
            },
        );
        Ok(())
    }

    pub fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        let sender = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .get(registration_id)
            .map(|worker| worker.relay.clone())
            .ok_or_else(|| "managed runtime is unavailable".to_owned())?;
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        sender
            .try_send(ManagedRuntimeRelayRequest::new(payload, response_sender))
            .map_err(|_| "managed runtime relay is unavailable".to_owned())?;
        response_receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .map_err(|_| "managed runtime relay timed out".to_owned())?
    }

    pub fn is_active(&self, registration_id: &str) -> Result<bool, String> {
        self.reap_finished();
        self.inner
            .workers
            .lock()
            .map(|workers| workers.contains_key(registration_id))
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
    }

    pub fn stop(&self, registration_id: &str) -> Result<(), String> {
        let worker = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .remove(registration_id)
            .ok_or_else(|| "managed runtime is unavailable".to_owned())?;
        worker.stop_requested.store(true, Ordering::Release);
        worker
            .join
            .join()
            .map_err(|_| "managed runtime supervisor worker panicked".to_owned())
    }

    pub fn reap_finished(&self) {
        let finished = match self.inner.workers.lock() {
            Ok(mut workers) => {
                let ids = workers
                    .iter()
                    .filter_map(|(id, worker)| worker.join.is_finished().then(|| id.clone()))
                    .collect::<Vec<_>>();
                ids.into_iter()
                    .filter_map(|id| workers.remove(&id))
                    .collect::<Vec<_>>()
            }
            Err(_) => return,
        };
        for worker in finished {
            let _ = worker.join.join();
        }
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.inner.shutdown_requested.store(true, Ordering::Release);
        let workers = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .drain()
            .map(|(_, worker)| worker.join)
            .collect::<Vec<_>>();
        for worker in workers {
            worker
                .join()
                .map_err(|_| "managed runtime supervisor worker panicked".to_owned())?;
        }
        Ok(())
    }
}

fn remove_staged_launch(
    staged_executable: StagedNativeArtifact,
    contracts: Option<StagedRuntimeContracts>,
) {
    let _ = staged_executable.remove();
    if let Some(contracts) = contracts {
        let _ = contracts.remove();
    }
}
