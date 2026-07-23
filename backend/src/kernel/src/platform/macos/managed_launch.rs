//! Prepares one fenced macOS managed runtime launch and hands it to the Kernel supervisor.

use std::path::Path;
use std::time::Duration;

use hermes_kernel_control_store::{ManagedLaunchRecord, PlatformStorageBindingStateV1};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::{
    v1::{
        ManagedDomainRuntimeConfigurationV1, ManagedIntegrationHostBridgeConfigurationV1,
        ManagedIntegrationRuntimeConfigurationV1, ManagedStorageRuntimeConfigurationV1,
    },
    validation::{
        integration_host_bridge::validate_managed_integration_host_bridge_configuration,
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use prost::Message;

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::infrastructure::filesystem::new_instance_id;
use crate::platform::macos::host_bridge_descriptor;
use crate::platform::macos::native_launch;
use crate::platform::{storage, vault::status as vault_status};
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub(crate) struct ManagedLaunchReservation {
    registration_id: String,
    binding: hermes_kernel_control_store::BundledManagedLaunchBinding,
    record: ManagedLaunchRecord,
    expectation: ManagedRuntimeExpectation,
    policy: ManagedChildExecutionPolicy,
}

impl ManagedLaunchReservation {
    #[must_use]
    pub(crate) fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub(crate) fn binding(&self) -> &hermes_kernel_control_store::BundledManagedLaunchBinding {
        &self.binding
    }

    #[must_use]
    pub(crate) fn runtime_instance_id(&self) -> &str {
        self.record.runtime_instance_id()
    }

    #[must_use]
    pub(crate) fn runtime_generation(&self) -> u64 {
        self.record.runtime_generation()
    }

    #[must_use]
    pub(crate) fn grant_epoch(&self) -> u64 {
        self.record.grant_epoch()
    }

    pub(crate) fn into_launch_parts(
        self,
    ) -> (
        String,
        ManagedRuntimeExpectation,
        ManagedChildExecutionPolicy,
    ) {
        (self.registration_id, self.expectation, self.policy)
    }

    /// Scheduler restarts need a successor runtime identity and cannot reuse this reservation.
    pub(crate) fn into_single_attempt_launch_parts(
        self,
    ) -> Result<
        (
            String,
            ManagedRuntimeExpectation,
            ManagedChildExecutionPolicy,
        ),
        String,
    > {
        let policy = ManagedChildExecutionPolicy::new(1, self.policy.max_runtime())?;
        Ok((self.registration_id, self.expectation, policy))
    }
}

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
) -> Result<u64, String> {
    let reservation = reserve(supervisor, store, registration_id)?;
    let kernel_executable = selected_kernel_executable()?;
    let staged = native_launch::stage_bound_installed_release(
        &kernel_executable,
        reservation.binding(),
        &runtime_dir.join("managed"),
    )?;
    let runtime_generation = reservation.runtime_generation();
    let (registration_id, expectation, policy) = reservation.into_launch_parts();
    supervisor.start(registration_id, staged, expectation, policy)?;
    Ok(runtime_generation)
}

fn selected_kernel_executable() -> Result<std::path::PathBuf, String> {
    #[cfg(test)]
    if let Some(executable) = std::env::var_os("HERMES_TEST_KERNEL_EXECUTABLE") {
        return Ok(std::path::PathBuf::from(executable));
    }
    std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())
}

pub fn start_with_storage_configuration(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
) -> Result<u64, String> {
    let reservation = reserve(supervisor, store, registration_id)?;
    let storage_binding = store
        .platform_storage_bindings()
        .map_err(|_| "managed runtime Storage binding is unavailable".to_owned())?
        .into_iter()
        .find(|binding| {
            binding.registration_id() == registration_id
                && binding.state() == PlatformStorageBindingStateV1::Active
        })
        .ok_or_else(|| "managed runtime Storage binding is unavailable".to_owned())?;
    let topology = storage::topology::current(store)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let configuration = storage::topology::to_managed_runtime_configuration(
        &topology,
        &storage_binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )?;
    start_staged_with_configuration(supervisor, store, runtime_dir, reservation, configuration)
}

fn start_staged_with_configuration(
    supervisor: &ManagedRuntimeSupervisor,
    _store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedStorageRuntimeConfigurationV1,
) -> Result<u64, String> {
    start_staged_with_configurations(supervisor, runtime_dir, reservation, configuration, None)
}

/// Starts one already-reserved provider integration from a Kernel-staged,
/// provider-neutral configuration. Provider settings and credentials are not
/// represented here.
pub(crate) fn start_reserved_integration(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedIntegrationRuntimeConfigurationV1,
    settings_snapshot_bytes: Vec<u8>,
) -> Result<u64, String> {
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "managed integration runtime configuration is invalid".to_owned())?;
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed integration runtime configuration is stale".to_owned());
    }
    start_staged_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        configuration.encode_to_vec(),
        Some(settings_snapshot_bytes),
        None,
        None,
    )
}

/// Starts one already-reserved business domain from a Kernel-staged domain
/// configuration. It is deliberately separate from integration launch so no
/// provider configuration instance or host bridge can enter a domain runtime.
pub(crate) fn start_reserved_domain(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedDomainRuntimeConfigurationV1,
) -> Result<u64, String> {
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "managed domain runtime configuration is invalid".to_owned())?;
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed domain runtime configuration is stale".to_owned());
    }
    start_staged_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        configuration.encode_to_vec(),
        None,
        None,
        None,
    )
}

pub(crate) fn start_staged_with_host_bridge_configuration(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedIntegrationRuntimeConfigurationV1,
    settings_snapshot_bytes: Vec<u8>,
    host_bridge_configuration: ManagedIntegrationHostBridgeConfigurationV1,
) -> Result<u64, String> {
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "managed integration runtime configuration is invalid".to_owned())?;
    validate_managed_integration_host_bridge_configuration(&host_bridge_configuration)
        .map_err(|_| "managed integration host bridge configuration is invalid".to_owned())?;
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
        || host_bridge_configuration.registration_id != reservation.registration_id()
        || host_bridge_configuration.runtime_instance_id != reservation.runtime_instance_id()
        || host_bridge_configuration.runtime_generation != reservation.runtime_generation()
        || host_bridge_configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed integration host bridge configuration is stale".to_owned());
    }
    let descriptor = host_bridge_descriptor::publish(runtime_dir, &host_bridge_configuration)?;
    start_staged_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        configuration.encode_to_vec(),
        Some(settings_snapshot_bytes),
        Some(host_bridge_configuration),
        Some(Box::new(move || descriptor.remove())),
    )
}

fn start_staged_with_configurations(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedStorageRuntimeConfigurationV1,
    host_bridge_configuration: Option<ManagedIntegrationHostBridgeConfigurationV1>,
) -> Result<u64, String> {
    start_staged_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        configuration.encode_to_vec(),
        None,
        host_bridge_configuration,
        None,
    )
}

fn start_staged_with_configuration_bytes(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    runtime_configuration_bytes: Vec<u8>,
    settings_snapshot_bytes: Option<Vec<u8>>,
    host_bridge_configuration: Option<ManagedIntegrationHostBridgeConfigurationV1>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
) -> Result<u64, String> {
    let kernel_executable = selected_kernel_executable()?;
    let prepared = native_launch::prepare_bound_managed_runtime(
        &kernel_executable,
        reservation.binding(),
        &runtime_dir
            .join("managed")
            .join(format!("launch-{}", reservation.runtime_generation())),
    )?;
    let host_bridge_configuration_bytes = host_bridge_configuration
        .as_ref()
        .map(prost::Message::encode_to_vec);
    let contracts = match (
        settings_snapshot_bytes,
        host_bridge_configuration_bytes.as_deref(),
    ) {
        (Some(settings_snapshot_bytes), Some(host_bridge_configuration_bytes)) => {
            StagedRuntimeContracts::stage_with_runtime_host_bridge_and_settings_snapshot(
                &runtime_dir
                    .join("managed")
                    .join(format!("launch-{}", reservation.runtime_generation()))
                    .join("contracts"),
                prepared.descriptor_bytes(),
                prepared.settings_schema_bytes(),
                Some(&settings_snapshot_bytes),
                Some(&runtime_configuration_bytes),
                Some(host_bridge_configuration_bytes),
            )?
        }
        (Some(settings_snapshot_bytes), None) => {
            StagedRuntimeContracts::stage_with_runtime_configuration_and_settings_snapshot(
                &runtime_dir
                    .join("managed")
                    .join(format!("launch-{}", reservation.runtime_generation()))
                    .join("contracts"),
                prepared.descriptor_bytes(),
                prepared.settings_schema_bytes(),
                &settings_snapshot_bytes,
                &runtime_configuration_bytes,
            )?
        }
        (None, Some(host_bridge_configuration_bytes)) => {
            StagedRuntimeContracts::stage_with_runtime_and_host_bridge_configuration(
                &runtime_dir
                    .join("managed")
                    .join(format!("launch-{}", reservation.runtime_generation()))
                    .join("contracts"),
                prepared.descriptor_bytes(),
                prepared.settings_schema_bytes(),
                Some(&runtime_configuration_bytes),
                Some(host_bridge_configuration_bytes),
            )?
        }
        (None, None) => StagedRuntimeContracts::stage_with_runtime_and_host_bridge_configuration(
            &runtime_dir
                .join("managed")
                .join(format!("launch-{}", reservation.runtime_generation()))
                .join("contracts"),
            prepared.descriptor_bytes(),
            prepared.settings_schema_bytes(),
            Some(&runtime_configuration_bytes),
            None,
        )?,
    };
    let mut arguments = vec![
        "serve-inherited".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
    ];
    let settings_schema_path = contracts
        .settings_schema_path()
        .ok_or_else(|| "managed runtime settings schema is unavailable".to_owned())?;
    arguments.push("--settings-schema-path".to_owned());
    arguments.push(settings_schema_path.display().to_string());
    if let Some(path) = contracts.settings_snapshot_path() {
        arguments.push("--settings-snapshot-path".to_owned());
        arguments.push(path.display().to_string());
    }
    let configuration_path = contracts
        .runtime_configuration_path()
        .ok_or_else(|| "managed runtime configuration is unavailable".to_owned())?;
    arguments.push("--runtime-configuration-path".to_owned());
    arguments.push(configuration_path.display().to_string());
    arguments.push("--runtime-instance-id".to_owned());
    arguments.push(reservation.runtime_instance_id().to_owned());
    if let Some(path) = contracts.host_bridge_configuration_path() {
        arguments.push("--host-bridge-configuration-path".to_owned());
        arguments.push(path.display().to_string());
    }
    let runtime_generation = reservation.runtime_generation();
    let (registration_id, expectation, policy) = reservation.into_launch_parts();
    if let Some(cleanup) = cleanup {
        supervisor.start_with_arguments_contracts_and_cleanup(
            crate::runtime::lifecycle::supervisor::ManagedRuntimeLaunchRequest {
                registration_id,
                staged_executable: prepared.into_staged_executable(),
                arguments,
                expectation,
                policy,
                contracts: Some(contracts),
                cleanup: Some(cleanup),
            },
        )?;
    } else {
        supervisor.start_with_arguments_and_contracts(
            registration_id,
            prepared.into_staged_executable(),
            arguments,
            expectation,
            policy,
            contracts,
        )?;
    }
    Ok(runtime_generation)
}

pub(crate) fn reserve(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<ManagedLaunchReservation, String> {
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
    let record = ManagedLaunchRecord::new(
        registration_id,
        new_instance_id()?,
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
    Ok(ManagedLaunchReservation {
        registration_id: registration_id.to_owned(),
        binding,
        record,
        expectation,
        policy,
    })
}

/// Reconstructs an unstarted durable reservation after a separate owner-control step.
pub(crate) fn load(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<ManagedLaunchReservation, String> {
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
    let record = store
        .effective_managed_launch_record(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch reservation is unavailable".to_owned())?;
    if record.registration_id() != registration_id
        || record.binding_revision() != binding.binding_revision()
        || record.kernel_generation() != store.snapshot().generation()
        || record.grant_epoch() != registration.grant_epoch()
    {
        return Err("managed launch reservation is stale".to_owned());
    }
    let expectation =
        ManagedRuntimeExpectation::from_fenced_launch(&registration, &binding, &record)?;
    let policy = ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?;
    Ok(ManagedLaunchReservation {
        registration_id: registration_id.to_owned(),
        binding,
        record,
        expectation,
        policy,
    })
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
