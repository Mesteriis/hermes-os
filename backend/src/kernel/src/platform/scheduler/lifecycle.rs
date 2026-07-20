//! Fail-closed reconciliation of an already admitted Scheduler runtime.
//!
//! An active Scheduler Storage binding is the durable desired-running record:
//! it is created only by the owner-authorized launch flow. A revoking binding
//! is deliberately excluded, so this worker never turns an intentional stop
//! into an automatic restart.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use hermes_kernel_control_store::{PlatformStorageBindingStateV1, PlatformStorageBindingV1};
use hermes_kernel_control_store_sqlite::SqliteControlStore;

use super::{launch, restart};
use crate::platform::storage::issuance::StorageBindingIssueV1;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const IDLE_POLL_INTERVAL: Duration = Duration::from_millis(250);

enum ReconcileOutcome {
    Idle,
    Active,
    Started,
}

/// Runs for the lifetime of the Kernel control plane. Reconciliation never
/// starts an unbound Scheduler and never retries forever after repeated launch
/// failures; an owner-authorized start/restart must re-establish a healthy
/// runtime before automatic crash recovery resumes.
pub(crate) fn serve(
    store: Arc<SqliteControlStore>,
    kernel: &Path,
    runtime_dir: &Path,
    shutdown_requested: Arc<AtomicBool>,
    supervisor: ManagedRuntimeSupervisor,
) -> Result<(), String> {
    let mut blocked = false;
    while !shutdown_requested.load(Ordering::Acquire) {
        if blocked {
            if scheduler_is_active(&store, &supervisor)? {
                blocked = false;
            }
            wait_for_poll(&shutdown_requested);
            continue;
        }
        let registration_id = active_scheduler_binding(&store)
            .ok()
            .flatten()
            .map(|binding| binding.registration_id().to_owned());
        match reconcile_once(&store, kernel, runtime_dir, &supervisor) {
            Ok(ReconcileOutcome::Idle | ReconcileOutcome::Active | ReconcileOutcome::Started) => {}
            Err(error) => {
                if let Some(registration_id) = registration_id {
                    let _ = supervisor.record_failure(&registration_id, error);
                }
                // A failed reconcile may already have revoked the predecessor and
                // reserved a successor. Retrying would fence that successor again
                // and erase the failure evidence, so require owner intervention.
                blocked = true;
            }
        }
        wait_for_poll(&shutdown_requested);
    }
    Ok(())
}

fn wait_for_poll(shutdown_requested: &AtomicBool) {
    let deadline = std::time::Instant::now() + IDLE_POLL_INTERVAL;
    while !shutdown_requested.load(Ordering::Acquire) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn reconcile_once(
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
) -> Result<ReconcileOutcome, String> {
    let Some(binding) = active_scheduler_binding(store)? else {
        return Ok(ReconcileOutcome::Idle);
    };
    if supervisor.is_active(binding.registration_id())? {
        return Ok(ReconcileOutcome::Active);
    }
    let issue = successor_issue(&binding)?;
    let (reservation, successor) = restart::reserve_successor(
        supervisor,
        store,
        binding.registration_id(),
        binding.capability_id(),
        issue,
    )?;
    launch::start_from_reservation(
        supervisor,
        store,
        kernel,
        runtime_dir,
        reservation,
        &successor,
    )?;
    Ok(ReconcileOutcome::Started)
}

fn scheduler_is_active(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) -> Result<bool, String> {
    active_scheduler_binding(store)?
        .map(|binding| supervisor.is_active(binding.registration_id()))
        .transpose()
        .map(|active| active.unwrap_or(false))
}

pub(crate) fn active_scheduler_binding(
    store: &SqliteControlStore,
) -> Result<Option<PlatformStorageBindingV1>, String> {
    let mut bindings = Vec::new();
    for snapshot in store
        .approved_module_grant_snapshots()
        .map_err(|_| "Scheduler lifecycle registrations are unavailable".to_owned())?
    {
        if snapshot.registration().module_id() != "scheduler" {
            continue;
        }
        let Some(grants) = snapshot.effective_grants() else {
            continue;
        };
        for capability_id in grants.capability_ids() {
            let binding = store
                .platform_storage_binding(snapshot.registration().registration_id(), capability_id)
                .map_err(|_| "Scheduler lifecycle Storage binding is unavailable".to_owned())?;
            if binding
                .as_ref()
                .is_some_and(|value| value.state() == PlatformStorageBindingStateV1::Active)
            {
                bindings.push(binding.expect("binding was checked as present"));
            }
        }
    }
    match bindings.len() {
        0 => Ok(None),
        1 => Ok(bindings.pop()),
        _ => Err("Scheduler lifecycle has multiple active Storage bindings".to_owned()),
    }
}

pub(crate) fn successor_issue(
    binding: &PlatformStorageBindingV1,
) -> Result<StorageBindingIssueV1, String> {
    let role_epoch = binding
        .role_epoch()
        .checked_add(1)
        .ok_or_else(|| "Scheduler Storage role epoch overflowed".to_owned())?;
    let credential_lease_revision = binding
        .credential_lease_revision()
        .checked_add(1)
        .ok_or_else(|| "Scheduler Storage credential revision overflowed".to_owned())?;
    StorageBindingIssueV1::new(
        role_epoch,
        credential_lease_revision,
        binding.storage_bundle_revision(),
        *binding.storage_bundle_digest(),
    )
}
