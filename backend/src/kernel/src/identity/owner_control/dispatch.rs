//! Request dispatch for owner-private control IPC.

mod platform;
mod scheduler;

use std::path::Path;

use sha2::{Digest, Sha256};

use hermes_gateway_protocol::v1::{
    ApproveModuleRegistrationRequestV1, ApproveModuleRegistrationResponseV1,
    BeginBrowserPairingRequestV1, BeginBrowserPairingResponseV1,
    BeginOwnerControlSessionResponseV1, BindBundledManagedReleaseRequestV1,
    BindBundledManagedReleaseResponseV1, BindExternalRuntimeIdentityRequestV1,
    BindExternalRuntimeIdentityResponseV1, CompleteOwnerControlSessionRequestV1,
    CompleteOwnerControlSessionResponseV1, GetModuleRegistrationStatusRequestV1,
    GetModuleRegistrationStatusResponseV1, OwnerControlRequestV1, OwnerControlResponseV1,
    ReserveBundledManagedRuntimeRequestV1, ReserveBundledManagedRuntimeResponseV1,
    StartBundledManagedRuntimeRequestV1, StartBundledManagedRuntimeResponseV1,
    StartReservedDomainRuntimeRequestV1, StartReservedDomainRuntimeResponseV1,
    StartReservedIntegrationRuntimeRequestV1, StartReservedIntegrationRuntimeResponseV1,
    TransitionModuleRegistrationRequestV1, TransitionModuleRegistrationResponseV1,
    UpdateOperatorSettingsRequestV1, UpdateOperatorSettingsResponseV1,
};
use hermes_kernel_control_store::{
    ModuleRegistrationState, PlatformStorageBindingStateV1, SettingsApplyState,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::{
    v1::{
        ManagedDomainRuntimeConfigurationV1, ManagedIntegrationHostBridgeConfigurationV1,
        ManagedIntegrationRuntimeConfigurationV1,
    },
    validation::{
        descriptor::decode_settings_snapshot_v1,
        integration_host_bridge::validate_managed_integration_host_bridge_configuration,
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};

use crate::identity::owner_control::sessions::OwnerControlSessions;
use crate::modules::registration::registry as module_registry;
use crate::modules::settings::mutation as settings_operator_mutation;
use crate::platform::gateway::BrowserPairingAdmissionV1;
use crate::platform::macos::bundled_release as macos_bundled_release_binding;
use crate::platform::macos::managed_launch as macos_managed_runtime_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(super) type OwnerResult = hermes_gateway_protocol::v1::owner_control_response_v1::Result;

pub(super) fn handle(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    request: OwnerControlRequestV1,
) -> OwnerControlResponseV1 {
    response(route(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        browser_pairing,
        sessions,
        request,
    ))
}

fn route(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    request: OwnerControlRequestV1,
) -> Result<OwnerResult, String> {
    let Some(operation) = request.operation else {
        return Err("owner control operation is unavailable".to_owned());
    };

    route_operation(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        browser_pairing,
        sessions,
        operation,
    )
}

fn route_operation(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    operation: hermes_gateway_protocol::v1::owner_control_request_v1::Operation,
) -> Result<OwnerResult, String> {
    use hermes_gateway_protocol::v1::owner_control_request_v1::Operation;

    match operation {
        Operation::GetModuleRegistrationStatus(request) => status(store, request),
        Operation::ApproveModuleRegistration(request) => approve(store, sessions, request),
        Operation::TransitionModuleRegistration(request) => transition(store, sessions, request),
        Operation::BeginOwnerSession(_) => begin(store, sessions),
        Operation::CompleteOwnerSession(request) => complete(store, sessions, request),
        Operation::BeginBrowserPairing(request) => {
            begin_browser_pairing(store, sessions, browser_pairing, request)
        }
        Operation::UpdateOperatorSettings(request) => update_settings(store, sessions, request),
        Operation::BindExternalRuntimeIdentity(request) => {
            bind_external_identity(store, sessions, request)
        }
        Operation::BindBundledManagedRelease(request) => {
            bind_managed_release(store, sessions, request)
        }
        Operation::StartBundledManagedRuntime(request) => {
            start_managed_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::ReserveBundledManagedRuntime(request) => {
            reserve_managed_runtime(store, supervisor, sessions, request)
        }
        Operation::StartReservedIntegrationRuntime(request) => {
            start_reserved_integration_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::StartReservedDomainRuntime(request) => {
            start_reserved_domain_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::StartReservedSchedulerRuntime(request) => {
            scheduler::start_reserved(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::UpsertSchedulerSchedule(request) => {
            scheduler::upsert(store, supervisor, sessions, request)
        }
        Operation::RestartSchedulerRuntime(request) => {
            scheduler::restart(store, runtime_dir, supervisor, sessions, request)
        }
        operation => platform::route(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            sessions,
            operation,
        ),
    }
}

fn begin_browser_pairing(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    request: BeginBrowserPairingRequestV1,
) -> Result<OwnerResult, String> {
    let owner = sessions.authorized_owner(store, &request.owner_session_id)?;
    let browser_pairing =
        browser_pairing.ok_or_else(|| "browser Gateway pairing is unavailable".to_owned())?;
    let pairing = browser_pairing.begin(owner.owner_id(), owner.device_id(), unix_millis()?)?;
    Ok(OwnerResult::BeginBrowserPairing(
        BeginBrowserPairingResponseV1 {
            pairing_id: pairing.pairing_id().to_owned(),
            expires_at_unix_millis: pairing.expires_at_unix_millis(),
        },
    ))
}

fn reserve_managed_runtime(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ReserveBundledManagedRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        macos_managed_runtime_launch::reserve(supervisor, store, &request.registration_id)
    })()
    .map(|reservation| {
        OwnerResult::ReserveBundledManagedRuntime(ReserveBundledManagedRuntimeResponseV1 {
            registration_id: reservation.registration_id().to_owned(),
            runtime_instance_id: reservation.runtime_instance_id().to_owned(),
            runtime_generation: reservation.runtime_generation(),
            grant_epoch: reservation.grant_epoch(),
        })
    })
}

fn start_reserved_integration_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedIntegrationRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let reservation =
            macos_managed_runtime_launch::load(supervisor, store, &request.registration_id)?;
        let registration = store
            .module_registration(&request.registration_id)
            .map_err(|_| "managed integration registration is unavailable".to_owned())?
            .ok_or_else(|| "managed integration registration is unavailable".to_owned())?;
        let binding = store
            .platform_storage_binding(&request.registration_id, &request.storage_capability_id)
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
        let settings_snapshot_bytes = admitted_settings_snapshot(store, &request.registration_id)?;
        let configuration = ManagedIntegrationRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: registration.owner_id().to_owned(),
            registration_id: request.registration_id.clone(),
            runtime_instance_id: reservation.runtime_instance_id().to_owned(),
            runtime_generation: reservation.runtime_generation(),
            grant_epoch: reservation.grant_epoch(),
            storage: Some(storage),
            event_hub_endpoint: event_topology.nats_endpoint().to_owned(),
            event_credential_revision: event_topology.credential_revision(),
            configuration_instance_id: request.configuration_instance_id.clone(),
        };
        validate_managed_integration_runtime_configuration(&configuration)
            .map_err(|_| "managed integration runtime configuration is invalid".to_owned())?;
        let host_bridge_configuration = host_bridge_configuration(
            request.request_host_bridge,
            runtime_dir,
            store.snapshot().instance_id(),
            registration.owner_id(),
            &reservation,
        )?;
        let host_bridge_socket_path = host_bridge_configuration
            .as_ref()
            .map(|configuration| configuration.socket_path.clone());
        let runtime_generation = match host_bridge_configuration {
            Some(host_bridge_configuration) => {
                macos_managed_runtime_launch::start_staged_with_host_bridge_configuration(
                    supervisor,
                    runtime_dir,
                    reservation,
                    configuration,
                    settings_snapshot_bytes,
                    host_bridge_configuration,
                )?
            }
            None => macos_managed_runtime_launch::start_reserved_integration(
                supervisor,
                runtime_dir,
                reservation,
                configuration,
                settings_snapshot_bytes,
            )?,
        };
        Ok((runtime_generation, host_bridge_socket_path))
    })()
    .map(|(runtime_generation, host_bridge_socket_path)| {
        OwnerResult::StartReservedIntegrationRuntime(StartReservedIntegrationRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
            host_bridge_socket_path,
        })
    })
}

fn start_reserved_domain_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedDomainRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let reservation =
            macos_managed_runtime_launch::load(supervisor, store, &request.registration_id)?;
        let registration = store
            .module_registration(&request.registration_id)
            .map_err(|_| "managed domain registration is unavailable".to_owned())?
            .ok_or_else(|| "managed domain registration is unavailable".to_owned())?;
        let binding = store
            .platform_storage_binding(&request.registration_id, &request.storage_capability_id)
            .map_err(|_| "managed domain Storage binding is unavailable".to_owned())?
            .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
            .ok_or_else(|| "managed domain Storage binding is unavailable".to_owned())?;
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
        let configuration = ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: registration.owner_id().to_owned(),
            registration_id: request.registration_id.clone(),
            runtime_instance_id: reservation.runtime_instance_id().to_owned(),
            runtime_generation: reservation.runtime_generation(),
            grant_epoch: reservation.grant_epoch(),
            storage: Some(storage),
            event_hub_endpoint: event_topology.nats_endpoint().to_owned(),
            event_credential_revision: event_topology.credential_revision(),
        };
        validate_managed_domain_runtime_configuration(&configuration)
            .map_err(|_| "managed domain runtime configuration is invalid".to_owned())?;
        macos_managed_runtime_launch::start_reserved_domain(
            supervisor,
            runtime_dir,
            reservation,
            configuration,
        )
    })()
    .map(|runtime_generation| {
        OwnerResult::StartReservedDomainRuntime(StartReservedDomainRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
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
    crate::infrastructure::filesystem::ensure_owner_private_directory(&parent)
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

fn admitted_settings_snapshot(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<Vec<u8>, String> {
    let binding = store
        .settings_schema_binding(registration_id)
        .map_err(|_| "managed integration settings are unavailable".to_owned())?
        .ok_or_else(|| "managed integration settings are unavailable".to_owned())?;
    if binding.desired_revision() == 0
        || binding.desired_revision() != binding.effective_revision()
        || binding.apply_state() != SettingsApplyState::Current
    {
        return Err("managed integration settings are not current".to_owned());
    }
    let (revision, bytes) = store
        .desired_settings_snapshot(registration_id)
        .map_err(|_| "managed integration settings are unavailable".to_owned())?
        .ok_or_else(|| "managed integration settings are unavailable".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&bytes)
        .map_err(|_| "managed integration settings are unavailable".to_owned())?;
    if revision != binding.desired_revision()
        || snapshot.target_id != registration_id
        || snapshot.revision != binding.desired_revision()
    {
        return Err("managed integration settings are stale".to_owned());
    }
    Ok(bytes)
}

fn status(
    store: &SqliteControlStore,
    request: GetModuleRegistrationStatusRequestV1,
) -> Result<OwnerResult, String> {
    module_registry::status(store, &request.registration_id).map(|status| {
        let attestation = status.external_runtime_attestation();
        OwnerResult::GetModuleRegistrationStatus(GetModuleRegistrationStatusResponseV1 {
            registration_id: status.registration().registration_id().to_owned(),
            module_id: status.registration().module_id().to_owned(),
            owner_id: status.registration().owner_id().to_owned(),
            registration_state: status.registration().state().as_str().to_owned(),
            grant_epoch: status.registration().grant_epoch(),
            effective_capability_count: status.effective_capability_count() as u32,
            external_runtime_id: attestation
                .map_or_else(String::new, |item| item.runtime_id().to_owned()),
            external_runtime_generation: attestation.map_or(0, |item| item.runtime_generation()),
        })
    })
}

fn approve(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: ApproveModuleRegistrationRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        module_registry::approve_after_owner_authorization(
            store,
            &request.registration_id,
            &request.capability_id,
        )
    })()
    .map(|grants| {
        OwnerResult::ApproveModuleRegistration(ApproveModuleRegistrationResponseV1 {
            registration_id: grants.registration_id().to_owned(),
            grant_epoch: grants.grant_epoch(),
            effective_capability_count: grants.capability_ids().len() as u32,
        })
    })
}

fn transition(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: TransitionModuleRegistrationRequestV1,
) -> Result<OwnerResult, String> {
    transition_target(&request.target_state)
        .and_then(|next| {
            sessions.authorize(store, &request.owner_session_id)?;
            module_registry::transition_after_owner_authorization(
                store,
                &request.registration_id,
                next,
            )
        })
        .map(|registration| {
            OwnerResult::TransitionModuleRegistration(TransitionModuleRegistrationResponseV1 {
                registration_id: registration.registration_id().to_owned(),
                registration_state: registration.state().as_str().to_owned(),
                grant_epoch: registration.grant_epoch(),
            })
        })
}

fn begin(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
) -> Result<OwnerResult, String> {
    sessions.begin(store).map(|challenge| {
        OwnerResult::BeginOwnerSession(BeginOwnerControlSessionResponseV1 {
            challenge_id: challenge.challenge_id().to_owned(),
            challenge_bytes: challenge.bytes().to_vec(),
            kernel_instance_id: challenge.kernel_instance_id().to_owned(),
            owner_id: challenge.owner_id().to_owned(),
            device_id: challenge.device_id().to_owned(),
            control_store_generation: challenge.control_store_generation(),
            expires_at_unix_millis: challenge.expires_at_unix_millis(),
        })
    })
}

fn complete(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: CompleteOwnerControlSessionRequestV1,
) -> Result<OwnerResult, String> {
    sessions
        .complete(store, &request.challenge_id, &request.signature_raw)
        .map(|session| {
            OwnerResult::CompleteOwnerSession(CompleteOwnerControlSessionResponseV1 {
                owner_session_id: session.session_id().to_owned(),
                expires_at_unix_millis: session.expires_at_unix_millis(),
            })
        })
}

fn update_settings(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: UpdateOperatorSettingsRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        settings_operator_mutation::commit_after_owner_authorization(
            store,
            &request.registration_id,
            request.expected_revision,
            &request.snapshot_bytes,
        )
    })()
    .map(|desired_revision| {
        OwnerResult::UpdateOperatorSettings(UpdateOperatorSettingsResponseV1 {
            registration_id: request.registration_id,
            desired_revision,
            apply_state: "pending_validation".to_owned(),
        })
    })
}

fn bind_external_identity(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: BindExternalRuntimeIdentityRequestV1,
) -> Result<OwnerResult, String> {
    let public_key_sec1: [u8; 65] = request
        .public_key_sec1
        .try_into()
        .map_err(|_| "external runtime public key is invalid".to_owned())?;
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        module_registry::bind_external_runtime_identity_after_owner_authorization(
            store,
            &request.registration_id,
            public_key_sec1,
        )
    })()
    .map(|registration| {
        OwnerResult::BindExternalRuntimeIdentity(BindExternalRuntimeIdentityResponseV1 {
            registration_id: registration.registration_id().to_owned(),
            grant_epoch: registration.grant_epoch(),
        })
    })
}

fn bind_managed_release(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: BindBundledManagedReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        macos_bundled_release_binding::bind_current_installed_release(
            store,
            &request.registration_id,
            &request.artifact_id,
        )
    })()
    .map(|binding| {
        OwnerResult::BindBundledManagedRelease(BindBundledManagedReleaseResponseV1 {
            registration_id: binding.registration_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

fn start_managed_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartBundledManagedRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        macos_managed_runtime_launch::start(
            supervisor,
            store,
            runtime_dir,
            &request.registration_id,
        )
    })()
    .map(|runtime_generation| {
        OwnerResult::StartBundledManagedRuntime(StartBundledManagedRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn transition_target(value: &str) -> Result<ModuleRegistrationState, String> {
    match value {
        "suspended" => Ok(ModuleRegistrationState::Suspended),
        "revoked" => Ok(ModuleRegistrationState::Revoked),
        _ => Err("owner control transition is unavailable".to_owned()),
    }
}

fn response(result: Result<OwnerResult, String>) -> OwnerControlResponseV1 {
    match result {
        Ok(result) => OwnerControlResponseV1 {
            result: Some(result),
            error_code: String::new(),
        },
        Err(_) => OwnerControlResponseV1 {
            result: None,
            error_code: "operation_denied".to_owned(),
        },
    }
}

fn unix_millis() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .map_err(|_| "owner control clock is unavailable".to_owned())
}
