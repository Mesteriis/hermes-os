//! Request dispatch for owner-private control IPC.

use std::path::Path;

use hermes_gateway_protocol::v1::{
    ApproveModuleRegistrationRequestV1, ApproveModuleRegistrationResponseV1,
    BeginOwnerControlSessionResponseV1, BindBundledManagedReleaseRequestV1,
    BindBundledManagedReleaseResponseV1, BindExternalRuntimeIdentityRequestV1,
    BindExternalRuntimeIdentityResponseV1, BindPlatformTelemetryReleaseRequestV1,
    BindPlatformTelemetryReleaseResponseV1, BindPlatformVaultReleaseRequestV1,
    BindPlatformVaultReleaseResponseV1, CompleteOwnerControlSessionRequestV1,
    CompleteOwnerControlSessionResponseV1, GetModuleRegistrationStatusRequestV1,
    GetModuleRegistrationStatusResponseV1, GetPlatformTelemetryDiagnosticsRequestV1,
    GetPlatformTelemetryDiagnosticsResponseV1, OwnerControlRequestV1, OwnerControlResponseV1,
    StartBundledManagedRuntimeRequestV1, StartBundledManagedRuntimeResponseV1,
    StartPlatformTelemetryRuntimeRequestV1, StartPlatformTelemetryRuntimeResponseV1,
    StartPlatformVaultRuntimeRequestV1, StartPlatformVaultRuntimeResponseV1,
    TransitionModuleRegistrationRequestV1, TransitionModuleRegistrationResponseV1,
    UpdateOperatorSettingsRequestV1, UpdateOperatorSettingsResponseV1,
};
use hermes_kernel_control_store::ModuleRegistrationState;
use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::identity::owner_control::sessions::OwnerControlSessions;
use crate::modules::registration::registry as module_registry;
use crate::modules::settings::mutation as settings_operator_mutation;
use crate::platform::macos::bundled_release as macos_bundled_release_binding;
use crate::platform::macos::managed_launch as macos_managed_runtime_launch;
use crate::platform::telemetry::binding as platform_telemetry_binding;
use crate::platform::telemetry::diagnostics as platform_telemetry_diagnostics;
use crate::platform::telemetry::launch as platform_telemetry_launch;
use crate::platform::vault::binding as platform_vault_binding;
use crate::platform::vault::launch as platform_vault_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

type OwnerResult = hermes_gateway_protocol::v1::owner_control_response_v1::Result;

pub(super) fn handle(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: OwnerControlRequestV1,
) -> OwnerControlResponseV1 {
    response(route(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        sessions,
        request,
    ))
}

fn route(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: OwnerControlRequestV1,
) -> Result<OwnerResult, String> {
    use hermes_gateway_protocol::v1::owner_control_request_v1::Operation;

    match request.operation {
        Some(Operation::GetModuleRegistrationStatus(request)) => status(store, request),
        Some(Operation::ApproveModuleRegistration(request)) => approve(store, sessions, request),
        Some(Operation::TransitionModuleRegistration(request)) => {
            transition(store, sessions, request)
        }
        Some(Operation::BeginOwnerSession(_)) => begin(store, sessions),
        Some(Operation::CompleteOwnerSession(request)) => complete(store, sessions, request),
        Some(Operation::UpdateOperatorSettings(request)) => {
            update_settings(store, sessions, request)
        }
        Some(Operation::BindExternalRuntimeIdentity(request)) => {
            bind_external_identity(store, sessions, request)
        }
        Some(Operation::BindBundledManagedRelease(request)) => {
            bind_managed_release(store, sessions, request)
        }
        Some(Operation::StartBundledManagedRuntime(request)) => {
            start_managed_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Some(Operation::BindPlatformVaultRelease(request)) => {
            bind_platform_vault_release(store, supervisor, sessions, request)
        }
        Some(Operation::StartPlatformVaultRuntime(request)) => start_platform_vault_runtime(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            sessions,
            request,
        ),
        Some(Operation::BindPlatformTelemetryRelease(request)) => {
            bind_platform_telemetry_release(store, supervisor, sessions, request)
        }
        Some(Operation::StartPlatformTelemetryRuntime(request)) => {
            start_platform_telemetry_runtime(
                store,
                data_dir,
                runtime_dir,
                supervisor,
                sessions,
                request,
            )
        }
        Some(Operation::GetPlatformTelemetryDiagnostics(request)) => {
            telemetry_diagnostics(store, supervisor, sessions, request)
        }
        None => Err("owner control operation is unavailable".to_owned()),
    }
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

fn bind_platform_vault_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformVaultReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = platform_vault_binding::bind_current_installed_release(store)?;
        if supervisor.is_active(platform_vault_binding::VAULT_PROCESS_ID)? {
            supervisor.stop(platform_vault_binding::VAULT_PROCESS_ID)?;
        }
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformVaultRelease(BindPlatformVaultReleaseResponseV1 {
            process_id: binding.process_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

fn bind_platform_telemetry_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformTelemetryReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = platform_telemetry_binding::bind_current_installed_release(store)?;
        if supervisor.is_active(platform_telemetry_binding::TELEMETRY_PROCESS_ID)? {
            supervisor.stop(platform_telemetry_binding::TELEMETRY_PROCESS_ID)?;
        }
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformTelemetryRelease(BindPlatformTelemetryReleaseResponseV1 {
            process_id: binding.process_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

fn start_platform_telemetry_runtime(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformTelemetryRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        platform_telemetry_launch::start(supervisor, store, data_dir, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformTelemetryRuntime(StartPlatformTelemetryRuntimeResponseV1 {
            process_id: platform_telemetry_binding::TELEMETRY_PROCESS_ID.to_owned(),
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn telemetry_diagnostics(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: GetPlatformTelemetryDiagnosticsRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        platform_telemetry_diagnostics::read(supervisor)
    })()
    .map(|diagnostics| {
        OwnerResult::GetPlatformTelemetryDiagnostics(GetPlatformTelemetryDiagnosticsResponseV1 {
            process_id: platform_telemetry_binding::TELEMETRY_PROCESS_ID.to_owned(),
            segment_count: diagnostics.segment_count(),
            total_bytes: diagnostics.total_bytes(),
        })
    })
}

fn start_platform_vault_runtime(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformVaultRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        platform_vault_launch::start(supervisor, store, data_dir, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformVaultRuntime(StartPlatformVaultRuntimeResponseV1 {
            process_id: platform_vault_binding::VAULT_PROCESS_ID.to_owned(),
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
