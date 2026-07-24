//! Structural validation for a Kernel-staged provider integration runtime.

use crate::v1::ManagedIntegrationRuntimeConfigurationV1;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_ENDPOINT_BYTES: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedIntegrationRuntimeValidationErrorV1 {
    InvalidConfiguration,
}

pub fn validate_managed_integration_runtime_configuration(
    configuration: &ManagedIntegrationRuntimeConfigurationV1,
) -> Result<(), ManagedIntegrationRuntimeValidationErrorV1> {
    let storage = configuration
        .storage
        .as_ref()
        .ok_or(ManagedIntegrationRuntimeValidationErrorV1::InvalidConfiguration)?;
    if configuration.major != 1
        || !valid_identifier(&configuration.logical_owner_id)
        || !valid_identifier(&configuration.registration_id)
        || !valid_identifier(&configuration.runtime_instance_id)
        || !valid_identifier(&configuration.configuration_instance_id)
        || configuration.runtime_generation == 0
        || configuration.grant_epoch == 0
        || !valid_event_hub_endpoint(&configuration.event_hub_endpoint)
        || configuration.event_credential_revision == 0
        || storage.logical_owner_id != configuration.logical_owner_id
        || storage.runtime_instance_id != configuration.runtime_instance_id
        || !valid_storage_configuration(storage)
    {
        return Err(ManagedIntegrationRuntimeValidationErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn valid_storage_configuration(storage: &crate::v1::ManagedStorageRuntimeConfigurationV1) -> bool {
    valid_identifier(&storage.database_id)
        && valid_identifier(&storage.pgbouncer_host)
        && storage.pgbouncer_port != 0
        && valid_identifier(&storage.runtime_principal)
        && storage.storage_generation != 0
        && storage.credential_revision != 0
        && valid_identifier(&storage.storage_instance_id)
        && valid_identifier(&storage.owner)
        && storage.role_epoch != 0
        && valid_identifier(&storage.pool_alias)
        && storage.max_connections != 0
        && storage.statement_timeout_millis != 0
        && storage.storage_bundle_revision != 0
        && storage.storage_bundle_digest.len() == 32
        && valid_identifier(&storage.vault_instance_id)
        && storage.vault_runtime_generation != 0
        && storage.vault_hpke_public_key_x25519.len() == 32
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_event_hub_endpoint(value: &str) -> bool {
    value.starts_with("nats://")
        && value.len() > "nats://".len()
        && value.len() <= MAX_ENDPOINT_BYTES
        && value.is_ascii()
        && !value.contains([' ', '\t', '\n', '\r', '#', '?', '@'])
}
