//! Prepares the fenced Storage child from its exact signed platform binding.

use std::path::Path;
use std::time::Duration;

use hermes_kernel_control_store::PlatformManagedProcessLaunch;
use hermes_kernel_control_store::PlatformStorageBindingStateV1;
use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::infrastructure::filesystem::prepare_owner_private_directory;
use crate::platform::macos::native_launch;
use crate::platform::storage::authorization::authorize_managed_binding;
use crate::platform::storage::binding::STORAGE_PROCESS_ID;
use crate::platform::storage::status;
use crate::platform::storage::topology;
use crate::platform::vault::status as vault_status;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const STORAGE_MODULE_ID: &str = "storage";
const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> Result<u64, String> {
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    start_from_kernel(supervisor, store, &kernel, runtime_dir)
}

pub(crate) fn start_from_kernel(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
) -> Result<u64, String> {
    ensure_inactive(supervisor)?;
    let binding = storage_binding(store)?;
    let topology = topology::current(store)?;
    let (desired_bindings, desired_bundles) = desired_configuration(store)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let runtime_generation = next_runtime_generation(store)?;
    let (prepared, contracts) = prepare_launch(
        kernel,
        &binding,
        &topology,
        &desired_bindings,
        &desired_bundles,
        store.snapshot().instance_id(),
        &vault,
        runtime_dir,
        runtime_generation,
    )?;
    let record = PlatformManagedProcessLaunch::new(
        STORAGE_PROCESS_ID,
        binding.binding_revision(),
        store.snapshot().generation(),
        runtime_generation,
        store.snapshot().grant_epoch(),
    );
    if let Err(error) = store.record_platform_managed_process_launch(&record) {
        let _ = contracts.remove();
        let _ = prepared.remove();
        return Err(format!("{error:?}"));
    }
    let expectation = ManagedRuntimeExpectation::from_platform_fenced_launch(
        STORAGE_PROCESS_ID,
        STORAGE_MODULE_ID,
        &binding,
        &record,
    )?;
    supervisor.start_with_arguments_and_contracts(
        STORAGE_PROCESS_ID.to_owned(),
        prepared.into_staged_executable(),
        inherited_arguments(&contracts),
        expectation,
        ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?,
        contracts,
    )?;
    supervisor.wait_until_ready(STORAGE_PROCESS_ID)?;
    match status::wait_current(store, &supervisor.relay_port()) {
        Ok(status) if status.runtime_generation() == runtime_generation => Ok(runtime_generation),
        Ok(_) | Err(_) => {
            let _ = supervisor.stop(STORAGE_PROCESS_ID);
            Err("Storage runtime did not confirm its managed status".to_owned())
        }
    }
}

fn desired_configuration(
    store: &SqliteControlStore,
) -> Result<
    (
        Vec<hermes_kernel_control_store::PlatformStorageBindingV1>,
        Vec<hermes_kernel_control_store::PlatformStorageBundleV1>,
    ),
    String,
> {
    let bindings = store
        .platform_storage_bindings()
        .map_err(|_| "Storage bindings are unavailable".to_owned())?
        .into_iter()
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .collect::<Vec<_>>();
    validate_desired_bindings(store, &bindings)?;
    load_bundles(store, &bindings).map(|bundles| (bindings, bundles))
}

fn ensure_inactive(supervisor: &ManagedRuntimeSupervisor) -> Result<(), String> {
    (!supervisor.is_active(STORAGE_PROCESS_ID)?)
        .then_some(())
        .ok_or_else(|| "Storage runtime is already active".to_owned())
}

fn load_bundles(
    store: &SqliteControlStore,
    bindings: &[hermes_kernel_control_store::PlatformStorageBindingV1],
) -> Result<Vec<hermes_kernel_control_store::PlatformStorageBundleV1>, String> {
    bindings
        .iter()
        .map(|binding| {
            store
                .platform_storage_bundle(binding.owner_id(), binding.storage_bundle_revision())
                .map_err(|_| "Storage bundle is unavailable".to_owned())?
                .ok_or_else(|| "Storage bundle is unavailable".to_owned())
        })
        .collect()
}

fn validate_desired_bindings(
    store: &SqliteControlStore,
    desired_bindings: &[hermes_kernel_control_store::PlatformStorageBindingV1],
) -> Result<(), String> {
    for binding in desired_bindings {
        let current = authorize_managed_binding(
            store,
            binding.registration_id(),
            binding.runtime_instance_id(),
            binding.runtime_generation(),
            binding.capability_id(),
        )?;
        if current.grant_epoch() != binding.grant_epoch()
            || current.owner_id() != binding.owner_id()
            || current.connection_budget() != binding.connection_budget()
            || current.statement_timeout_millis() != binding.statement_timeout_millis()
        {
            return Err("Storage binding authorization is stale".to_owned());
        }
    }
    Ok(())
}

pub(crate) fn current_launch(
    store: &SqliteControlStore,
) -> Result<PlatformManagedProcessLaunch, String> {
    let binding = storage_binding(store)?;
    let launch = store
        .platform_managed_process_launch(STORAGE_PROCESS_ID)
        .map_err(|_| "Storage runtime is unavailable".to_owned())?
        .ok_or_else(|| "Storage runtime is unavailable".to_owned())?;
    if launch.binding_revision() != binding.binding_revision()
        || launch.kernel_generation() != store.snapshot().generation()
        || launch.grant_epoch() != store.snapshot().grant_epoch()
    {
        return Err("Storage runtime is stale".to_owned());
    }
    Ok(launch)
}

fn storage_binding(
    store: &SqliteControlStore,
) -> Result<hermes_kernel_control_store::PlatformManagedProcessBinding, String> {
    store
        .platform_managed_process_binding(STORAGE_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "Storage release binding is unavailable".to_owned())
}

fn prepare_launch(
    kernel: &Path,
    binding: &hermes_kernel_control_store::PlatformManagedProcessBinding,
    topology: &hermes_kernel_control_store::PlatformStorageTopology,
    desired_bindings: &[hermes_kernel_control_store::PlatformStorageBindingV1],
    desired_bundles: &[hermes_kernel_control_store::PlatformStorageBundleV1],
    vault_instance_id: &str,
    vault: &vault_status::ManagedVaultStatus,
    runtime_dir: &Path,
    runtime_generation: u64,
) -> Result<
    (
        native_launch::PreparedPlatformManagedProcess,
        StagedRuntimeContracts,
    ),
    String,
> {
    let prepared = native_launch::prepare_bound_platform_process(
        kernel,
        binding,
        &runtime_dir
            .join("storage")
            .join(format!("launch-{runtime_generation}"))
            .join("managed"),
    )?;
    let (pgbouncer_directory, pgbouncer_auth_directory) =
        match prepare_pgbouncer_directories(runtime_dir) {
            Ok(directories) => directories,
            Err(error) => {
                let _ = prepared.remove();
                return Err(error);
            }
        };
    let configuration = topology::encoded_managed_macos(
        topology,
        desired_bindings,
        desired_bundles,
        &pgbouncer_directory.join("databases.ini"),
        &pgbouncer_auth_directory.join("users.txt"),
        vault_instance_id,
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )?;
    match StagedRuntimeContracts::stage_with_runtime_configuration(
        &runtime_dir
            .join("storage")
            .join(format!("launch-{runtime_generation}"))
            .join("contracts"),
        prepared.descriptor_bytes(),
        prepared.settings_schema_bytes(),
        Some(&configuration),
    ) {
        Ok(contracts) => Ok((prepared, contracts)),
        Err(error) => {
            let _ = prepared.remove();
            Err(error)
        }
    }
}

fn prepare_pgbouncer_directories(
    runtime_dir: &Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let pgbouncer_directory = runtime_dir.join("storage").join("pgbouncer");
    prepare_owner_private_directory(&pgbouncer_directory)?;
    let pgbouncer_auth_directory = pgbouncer_directory.join("auth");
    prepare_owner_private_directory(&pgbouncer_auth_directory)?;
    Ok((pgbouncer_directory, pgbouncer_auth_directory))
}

fn next_runtime_generation(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_launch(STORAGE_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |record| {
            record
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "Storage runtime generation overflowed".to_owned())
        })
}

pub(crate) fn inherited_arguments(contracts: &StagedRuntimeContracts) -> Vec<String> {
    let mut arguments = vec![
        "serve-inherited".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
    ];
    if let Some(path) = contracts.settings_schema_path() {
        arguments.push("--settings-schema-path".to_owned());
        arguments.push(path.display().to_string());
    }
    if let Some(path) = contracts.runtime_configuration_path() {
        arguments.push("--configuration-path".to_owned());
        arguments.push(path.display().to_string());
    }
    arguments
}
