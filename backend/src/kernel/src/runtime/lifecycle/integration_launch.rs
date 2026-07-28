//! Shared managed-integration launch composition below owner-control transports.

use std::path::Path;

use hermes_kernel_control_store::{PlatformStorageBindingStateV1, SettingsApplyState};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::{
    SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID,
    v1::{
        ManagedIntegrationConfigurationInstanceV1, ManagedIntegrationHostBridgeConfigurationV1,
        ManagedIntegrationRuntimeConfigurationV1,
    },
    validation::{
        descriptor::decode_settings_snapshot_v1,
        integration_host_bridge::validate_managed_integration_host_bridge_configuration,
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use sha2::{Digest, Sha256};

use crate::platform::macos::managed_launch as macos_managed_runtime_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(crate) struct AdmittedSettingsSnapshotV1 {
    pub(crate) revision: u64,
    pub(crate) bytes: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_reserved(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    storage_capability_id: &str,
    configuration_instance_id: &str,
    request_host_bridge: bool,
    settings_snapshot_bytes: Option<Vec<u8>>,
) -> Result<(u64, Option<String>), String> {
    let reservation = macos_managed_runtime_launch::load(supervisor, store, registration_id)?;
    let registration = store
        .module_registration(registration_id)
        .map_err(|_| "managed integration registration is unavailable".to_owned())?
        .ok_or_else(|| "managed integration registration is unavailable".to_owned())?;
    let granted_capability_ids = store
        .module_grant_snapshot(registration_id)
        .map_err(|_| "managed integration grants are unavailable".to_owned())?
        .and_then(|snapshot| {
            snapshot
                .effective_grants()
                .map(|grants| grants.capability_ids().to_vec())
        })
        .ok_or_else(|| "managed integration grants are unavailable".to_owned())?;
    let binding = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(|_| "managed integration Storage binding is unavailable".to_owned())?
        .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
        .ok_or_else(|| "managed integration Storage binding is unavailable".to_owned())?;
    let storage_topology = crate::platform::storage::topology::current(store)?;
    let vault = crate::platform::vault::status::read_current(store, &supervisor.relay_port())?;
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &storage_topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )?;
    let event_topology = store
        .platform_event_hub_topology()
        .map_err(|_| "Event Hub topology is unavailable".to_owned())?
        .ok_or_else(|| "Event Hub topology is unavailable".to_owned())?;
    let settings_snapshot_bytes = match settings_snapshot_bytes {
        Some(bytes) => bytes,
        None => admitted_settings_snapshot(store, registration_id)?.bytes,
    };
    let configuration_instances = if granted_capability_ids
        .iter()
        .any(|capability_id| capability_id == SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID)
    {
        admitted_configuration_instances(
            store,
            registration_id,
            configuration_instance_id,
            &settings_snapshot_bytes,
        )?
    } else {
        Vec::new()
    };
    let configuration = ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: registration.owner_id().to_owned(),
        registration_id: registration_id.to_owned(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        storage: Some(storage),
        event_hub_endpoint: event_topology.nats_endpoint().to_owned(),
        event_credential_revision: event_topology.credential_revision(),
        configuration_instance_id: configuration_instance_id.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances,
    };
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "managed integration runtime configuration is invalid".to_owned())?;
    let host_bridge_configuration = host_bridge_configuration(
        request_host_bridge,
        runtime_dir,
        store.snapshot().instance_id(),
        registration.owner_id(),
        &reservation,
    )?;
    let host_bridge_socket_path = host_bridge_configuration
        .as_ref()
        .map(|configuration| configuration.socket_path.clone());
    let launch_configuration =
        macos_managed_runtime_launch::ManagedIntegrationLaunchConfiguration {
            runtime: configuration,
            settings_snapshot_bytes,
            granted_capability_ids: &granted_capability_ids,
        };
    let runtime_generation = match host_bridge_configuration {
        Some(host_bridge_configuration) => {
            macos_managed_runtime_launch::start_staged_with_host_bridge_configuration(
                supervisor,
                data_dir,
                runtime_dir,
                reservation,
                launch_configuration,
                host_bridge_configuration,
            )?
        }
        None => macos_managed_runtime_launch::start_reserved_integration(
            supervisor,
            data_dir,
            runtime_dir,
            reservation,
            launch_configuration,
        )?,
    };
    Ok((runtime_generation, host_bridge_socket_path))
}

fn admitted_configuration_instances(
    store: &SqliteControlStore,
    registration_id: &str,
    selected_configuration_instance_id: &str,
    selected_snapshot_bytes: &[u8],
) -> Result<Vec<ManagedIntegrationConfigurationInstanceV1>, String> {
    let selected_snapshot = decode_settings_snapshot_v1(selected_snapshot_bytes)
        .map_err(|_| "managed integration settings catalog is invalid".to_owned())?;
    if selected_snapshot.target_id != selected_configuration_instance_id
        || selected_snapshot.revision == 0
    {
        return Err("managed integration settings catalog is invalid".to_owned());
    }

    let mut selected_found = false;
    let mut instances = Vec::new();
    for target in store
        .settings_configuration_targets(registration_id)
        .map_err(|_| "managed integration settings catalog is unavailable".to_owned())?
    {
        let target_id = target.configuration_instance_id();
        let snapshot_bytes = if target_id == selected_configuration_instance_id {
            selected_found = true;
            selected_snapshot_bytes.to_vec()
        } else {
            if target.effective_revision() == 0
                || target.desired_revision() != target.effective_revision()
                || target.apply_state() != SettingsApplyState::Current
            {
                continue;
            }
            let (revision, bytes) = store
                .desired_settings_snapshot_for_target(registration_id, target_id)
                .map_err(|_| "managed integration settings catalog is unavailable".to_owned())?
                .ok_or_else(|| "managed integration settings catalog is unavailable".to_owned())?;
            let snapshot = decode_settings_snapshot_v1(&bytes)
                .map_err(|_| "managed integration settings catalog is invalid".to_owned())?;
            if revision != target.effective_revision()
                || snapshot.target_id != target_id
                || snapshot.revision != target.effective_revision()
            {
                return Err("managed integration settings catalog is stale".to_owned());
            }
            bytes
        };
        instances.push(ManagedIntegrationConfigurationInstanceV1 {
            configuration_instance_id: target_id.to_owned(),
            settings_snapshot_bytes: snapshot_bytes,
            integration_state_root: None,
        });
    }
    if !selected_found {
        return Err("managed integration settings target is unavailable".to_owned());
    }
    instances.sort_by(|left, right| {
        left.configuration_instance_id
            .cmp(&right.configuration_instance_id)
    });
    Ok(instances)
}

fn host_bridge_configuration(
    requested: bool,
    runtime_dir: &Path,
    kernel_instance_id: &str,
    owner_id: &str,
    reservation: &macos_managed_runtime_launch::ManagedLaunchReservation,
) -> Result<Option<ManagedIntegrationHostBridgeConfigurationV1>, String> {
    if !requested {
        return Ok(None);
    }
    let parent = runtime_dir.join("host-bridges");
    crate::infrastructure::filesystem::prepare_owner_private_directory(&parent)
        .map_err(|_| "host bridge socket parent is invalid".to_owned())?;
    let mut route_name = Sha256::new();
    for field in [
        kernel_instance_id,
        owner_id,
        reservation.registration_id(),
        reservation.runtime_instance_id(),
    ] {
        route_name.update(field.as_bytes());
        route_name.update([0]);
    }
    route_name.update(reservation.runtime_generation().to_be_bytes());
    route_name.update(reservation.grant_epoch().to_be_bytes());
    let digest = route_name.finalize();
    let route_name = format!(
        "host-{}.sock",
        digest[..16]
            .iter()
            .map(|value| format!("{value:02x}"))
            .collect::<String>(),
    );
    let path = parent.join(route_name);
    let socket_path = path
        .to_str()
        .filter(|value| !value.is_empty() && value.len() <= 96)
        .ok_or_else(|| "host bridge socket path is invalid".to_owned())?;
    if std::fs::symlink_metadata(&path).is_ok() {
        return Err("host bridge socket path must be absent".to_owned());
    }
    let mut binding = Sha256::new();
    for field in [
        kernel_instance_id,
        owner_id,
        reservation.registration_id(),
        reservation.runtime_instance_id(),
        socket_path,
    ] {
        binding.update(field.as_bytes());
        binding.update([0]);
    }
    binding.update(reservation.runtime_generation().to_be_bytes());
    binding.update(reservation.grant_epoch().to_be_bytes());
    let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
        major: 1,
        kernel_instance_id: kernel_instance_id.to_owned(),
        owner_id: owner_id.to_owned(),
        registration_id: reservation.registration_id().to_owned(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        socket_path: socket_path.to_owned(),
        route_binding_sha256: binding.finalize().to_vec(),
    };
    validate_managed_integration_host_bridge_configuration(&configuration)
        .map_err(|_| "host bridge socket path is invalid".to_owned())?;
    Ok(Some(configuration))
}

pub(crate) fn admitted_settings_snapshot(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<AdmittedSettingsSnapshotV1, String> {
    let binding = store
        .settings_schema_binding(registration_id)
        .map_err(|_| "managed module settings are unavailable".to_owned())?
        .ok_or_else(|| "managed module settings are unavailable".to_owned())?;
    if binding.desired_revision() == 0
        || binding.desired_revision() != binding.effective_revision()
        || binding.apply_state() != SettingsApplyState::Current
    {
        return Err("managed module settings are not current".to_owned());
    }
    let (revision, bytes) = store
        .desired_settings_snapshot(registration_id)
        .map_err(|_| "managed module settings are unavailable".to_owned())?
        .ok_or_else(|| "managed module settings are unavailable".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&bytes)
        .map_err(|_| "managed module settings are unavailable".to_owned())?;
    if revision != binding.desired_revision()
        || snapshot.target_id != registration_id
        || snapshot.revision != binding.desired_revision()
    {
        return Err("managed module settings are stale".to_owned());
    }
    Ok(AdmittedSettingsSnapshotV1 { revision, bytes })
}
