//! Durable successor identity and Storage binding for a crashed Scheduler runtime.

use hermes_kernel_control_store::{PlatformStorageBindingStateV1, PlatformStorageBindingV1};
use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::platform::macos::managed_launch::{self, ManagedLaunchReservation};
use crate::platform::storage::issuance::{StorageBindingIssueV1, issue_managed};
use crate::platform::storage::revocation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

/// Fences a predecessor before reserving the fresh Scheduler identity bound to its successor.
pub(crate) fn reserve_successor(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
    storage_capability_id: &str,
    issue: StorageBindingIssueV1,
) -> Result<(ManagedLaunchReservation, PlatformStorageBindingV1), String> {
    revoke_predecessor(supervisor, store, registration_id, storage_capability_id)?;
    let reservation = managed_launch::reserve(supervisor, store, registration_id)?;
    let binding = issue_managed(
        store,
        reservation.registration_id(),
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        storage_capability_id,
        issue,
    )?;
    Ok((reservation, binding))
}

fn revoke_predecessor(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
    storage_capability_id: &str,
) -> Result<(), String> {
    let predecessor = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(|_| "Scheduler Storage binding is unavailable".to_owned())?;
    if let Some(predecessor) = predecessor {
        let revoking = match predecessor.state() {
            PlatformStorageBindingStateV1::Active => store
                .begin_platform_storage_binding_revocation(
                    registration_id,
                    storage_capability_id,
                    predecessor.binding_revision(),
                )
                .map_err(|_| {
                    "Scheduler Storage binding cannot be reserved for revocation".to_owned()
                })?,
            PlatformStorageBindingStateV1::Revoking => predecessor,
        };
        revocation::fence_reserved_binding(supervisor, store, &revoking)?;
    }
    if supervisor.is_active(registration_id)? {
        supervisor.stop(registration_id)?;
    }
    Ok(())
}
