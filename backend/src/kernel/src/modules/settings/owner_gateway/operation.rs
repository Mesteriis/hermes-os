//! Canonical Settings mutation and managed-integration apply execution.

use std::path::Path;

use hermes_gateway_protocol::v1::{
    ApplyOwnerManagedIntegrationSettingsReceiptV1, ApplyOwnerManagedIntegrationSettingsV1,
    CommitOwnerModuleSettingsResponseV1, UpdateOwnerModuleSettingsReceiptV1,
    UpdateOwnerModuleSettingsV1, commit_owner_module_settings_response_v1,
};
use hermes_gateway_runtime::OwnerModuleSettingsRouteErrorV1;
use hermes_kernel_control_store::SettingsApplyState;
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;

use super::values::canonical_snapshot;
use crate::modules::settings::{managed_integration, mutation};
use crate::runtime::lifecycle::{integration_launch, supervisor::ManagedRuntimeSupervisor};

pub(super) fn update_desired(
    store: &SqliteControlStore,
    operation_id: Vec<u8>,
    update: UpdateOwnerModuleSettingsV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let snapshot = canonical_snapshot(
        &update.registration_id,
        update.expected_desired_revision,
        update.values,
    )?;
    let desired_revision = mutation::commit_after_owner_authorization(
        store,
        &update.registration_id,
        update.expected_desired_revision,
        &snapshot.encode_to_vec(),
    )
    .map_err(map_mutation_error)?;
    let binding = store
        .settings_schema_binding(&update.registration_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::Internal)?;
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(commit_owner_module_settings_response_v1::Result::Updated(
            UpdateOwnerModuleSettingsReceiptV1 {
                registration_id: update.registration_id,
                desired_revision,
                apply_state: binding.apply_state().as_str().to_owned(),
            },
        )),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_managed_integration(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    operation_id: Vec<u8>,
    apply: ApplyOwnerManagedIntegrationSettingsV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let prepared = managed_integration::prepare(
        store,
        supervisor,
        &apply.registration_id,
        &apply.storage_capability_id,
        apply.expected_desired_revision,
    )
    .map_err(map_apply_preparation_error)?;
    let launch = integration_launch::launch_reserved(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        &apply.registration_id,
        &apply.storage_capability_id,
        &apply.configuration_instance_id,
        apply.request_host_bridge,
        Some(prepared.snapshot_bytes().to_vec()),
    );
    let (runtime_generation, host_bridge_socket_path) = match launch {
        Ok(launch) => launch,
        Err(_) => {
            managed_integration::block_after_launch_failure(
                store,
                &apply.registration_id,
                prepared.revision(),
            );
            return Err(OwnerModuleSettingsRouteErrorV1::Unavailable);
        }
    };
    managed_integration::wait_for_ready_and_confirm(
        store,
        supervisor,
        &apply.registration_id,
        prepared.revision(),
    )
    .map_err(|_| OwnerModuleSettingsRouteErrorV1::Unavailable)?;
    let binding = store
        .settings_schema_binding(&apply.registration_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::Internal)?;
    if binding.effective_revision() != prepared.revision()
        || binding.apply_state() != SettingsApplyState::Current
    {
        return Err(OwnerModuleSettingsRouteErrorV1::Internal);
    }
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(commit_owner_module_settings_response_v1::Result::Applied(
            ApplyOwnerManagedIntegrationSettingsReceiptV1 {
                registration_id: apply.registration_id,
                effective_revision: binding.effective_revision(),
                runtime_generation,
                apply_state: binding.apply_state().as_str().to_owned(),
                host_bridge_socket_path,
            },
        )),
    })
}

fn map_mutation_error(error: String) -> OwnerModuleSettingsRouteErrorV1 {
    if error.contains("conflict") {
        OwnerModuleSettingsRouteErrorV1::Conflict
    } else if error.contains("not admitted") || error.contains("unavailable") {
        OwnerModuleSettingsRouteErrorV1::NotFound
    } else {
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    }
}

fn map_apply_preparation_error(error: String) -> OwnerModuleSettingsRouteErrorV1 {
    if error.contains("revision") || error.contains("active") {
        OwnerModuleSettingsRouteErrorV1::Conflict
    } else if error.contains("unavailable") {
        OwnerModuleSettingsRouteErrorV1::NotFound
    } else {
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    }
}
