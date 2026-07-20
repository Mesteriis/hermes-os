//! Verifies and persists the immutable SettingsSchema artifact for one module registration.

use hermes_kernel_control_store::{
    ModuleRegistrationState, ModuleRegistryStore, SettingsApplyState, SettingsRegistryStore,
    SettingsSchemaBinding, SettingsSchemaBindingInputV1,
};
use hermes_kernel_control_store_sqlite::StoreError;
use hermes_runtime_protocol::validation::descriptor::{
    decode_descriptor_v1, decode_settings_schema_v1,
};
use sha2::{Digest, Sha256};

pub fn admit<S>(
    store: &S,
    registration_id: &str,
    descriptor_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<SettingsSchemaBinding, String>
where
    S: ModuleRegistryStore<Error = StoreError> + SettingsRegistryStore<Error = StoreError>,
{
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "module registration does not exist".to_owned())?;
    if registration.state() != ModuleRegistrationState::Approved {
        return Err(
            "settings schema admission requires an approved module registration".to_owned(),
        );
    }
    let descriptor = decode_descriptor_v1(descriptor_bytes)
        .map_err(|_| "module descriptor is invalid or exceeds protocol limits".to_owned())?;
    let descriptor_sha256: [u8; 32] = Sha256::digest(descriptor_bytes).into();
    if descriptor_sha256 != *registration.descriptor_sha256() {
        return Err("settings schema descriptor does not match the registration".to_owned());
    }
    let schema_ref = descriptor
        .settings_schema_ref
        .as_ref()
        .ok_or_else(|| "module descriptor does not declare a settings schema".to_owned())?;
    let schema = decode_settings_schema_v1(schema_bytes)
        .map_err(|_| "settings schema is invalid or exceeds protocol limits".to_owned())?;
    let schema_sha256: [u8; 32] = Sha256::digest(schema_bytes).into();
    if schema_ref.major != schema.major
        || schema_ref.revision != schema.revision
        || schema_ref.artifact_size_bytes != schema_bytes.len() as u64
        || schema_ref.sha256.as_slice() != schema_sha256
    {
        return Err("settings schema does not match the descriptor binding".to_owned());
    }
    validate_capability_bindings(&descriptor, &schema)?;
    let binding = SettingsSchemaBinding::new(SettingsSchemaBindingInputV1 {
        registration_id: registration_id.to_owned(),
        schema_major: schema.major,
        schema_revision: schema.revision,
        schema_sha256,
        desired_revision: 0,
        effective_revision: 0,
        apply_state: SettingsApplyState::Current,
        sanitized_reason_code: None,
    });
    store
        .admit_settings_schema(&binding, schema_bytes)
        .map_err(|error| format!("{error:?}"))?;
    Ok(binding)
}

fn validate_capability_bindings(
    descriptor: &hermes_runtime_protocol::v1::ModuleDescriptorV1,
    schema: &hermes_runtime_protocol::v1::SettingsSchemaV1,
) -> Result<(), String> {
    for definition in &schema.definitions {
        if definition.capability_id.is_empty() {
            continue;
        }
        let capability = descriptor
            .capabilities
            .binary_search_by(|item| item.capability_id.cmp(&definition.capability_id))
            .map(|index| &descriptor.capabilities[index])
            .map_err(|_| "settings schema references an unknown capability".to_owned())?;
        if capability
            .settings_definition_ids
            .binary_search(&definition.setting_id)
            .is_err()
        {
            return Err("settings schema definition is absent from its capability".to_owned());
        }
    }
    for capability in &descriptor.capabilities {
        for setting_id in &capability.settings_definition_ids {
            let definition = schema
                .definitions
                .binary_search_by(|item| item.setting_id.cmp(setting_id))
                .map(|index| &schema.definitions[index])
                .map_err(|_| {
                    "module descriptor references an unknown settings definition".to_owned()
                })?;
            if definition.capability_id != capability.capability_id {
                return Err("settings definition is bound to the wrong capability".to_owned());
            }
        }
    }
    Ok(())
}
