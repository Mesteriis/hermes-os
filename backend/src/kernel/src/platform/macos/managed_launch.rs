//! Prepares one fenced macOS managed runtime launch and hands it to the Kernel supervisor.

use std::path::Path;
use std::time::Duration;

use hermes_kernel_control_store::ManagedLaunchRecord;
use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::platform::macos::native_launch;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
) -> Result<u64, String> {
    if supervisor.is_active(registration_id)? {
        return Err("managed runtime is already active for this registration".to_owned());
    }
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch registration does not exist".to_owned())?;
    let binding = store
        .effective_bundled_managed_launch_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch binding is unavailable".to_owned())?;
    let runtime_generation = next_runtime_generation(store, registration_id)?;
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    let staged = native_launch::stage_bound_installed_release(
        &kernel_executable,
        &binding,
        &runtime_dir.join("managed"),
    )?;
    let record = ManagedLaunchRecord::new(
        registration_id,
        binding.binding_revision(),
        store.snapshot().generation(),
        runtime_generation,
        registration.grant_epoch(),
    );
    let expectation =
        ManagedRuntimeExpectation::from_fenced_launch(&registration, &binding, &record)?;
    let policy = ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?;
    store
        .record_managed_launch(&record)
        .map_err(|error| format!("{error:?}"))?;
    supervisor.start(registration_id.to_owned(), staged, expectation, policy)?;
    Ok(runtime_generation)
}

fn next_runtime_generation(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<u64, String> {
    store
        .effective_managed_launch_record(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |record| {
            record
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "managed runtime generation overflowed".to_owned())
        })
}
