//! Structural validation for a Kernel-staged business domain runtime.

use crate::v1::ManagedDomainRuntimeConfigurationV1;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_ENDPOINT_BYTES: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedDomainRuntimeValidationErrorV1 {
    InvalidConfiguration,
}

pub fn validate_managed_domain_runtime_configuration(
    configuration: &ManagedDomainRuntimeConfigurationV1,
) -> Result<(), ManagedDomainRuntimeValidationErrorV1> {
    let storage = configuration.storage.as_ref().ok_or(
        ManagedDomainRuntimeValidationErrorV1::InvalidConfiguration,
    )?;
    if configuration.major != 1
        || !valid_identifier(&configuration.logical_owner_id)
        || !valid_identifier(&configuration.registration_id)
        || !valid_identifier(&configuration.runtime_instance_id)
        || configuration.runtime_generation == 0
        || configuration.grant_epoch == 0
        || !valid_event_hub_endpoint(&configuration.event_hub_endpoint)
        || configuration.event_credential_revision == 0
        || storage.logical_owner_id != configuration.logical_owner_id
        || storage.runtime_instance_id != configuration.runtime_instance_id
        || !valid_storage_configuration(storage)
    {
        return Err(ManagedDomainRuntimeValidationErrorV1::InvalidConfiguration);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::ManagedStorageRuntimeConfigurationV1;

    fn configuration() -> ManagedDomainRuntimeConfigurationV1 {
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: "communications".to_owned(),
            registration_id: "communications-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
            storage: Some(ManagedStorageRuntimeConfigurationV1 {
                database_id: "communications".to_owned(),
                pgbouncer_host: "localhost".to_owned(),
                pgbouncer_port: 6432,
                runtime_principal: "communications_runtime".to_owned(),
                storage_generation: 1,
                credential_revision: 1,
                storage_instance_id: "storage-1".to_owned(),
                owner: "communications".to_owned(),
                role_epoch: 1,
                pool_alias: "communications".to_owned(),
                max_connections: 1,
                statement_timeout_millis: 1,
                storage_bundle_revision: 1,
                storage_bundle_digest: vec![1; 32],
                vault_instance_id: "vault-1".to_owned(),
                vault_runtime_generation: 1,
                vault_hpke_public_key_x25519: vec![1; 32],
                runtime_instance_id: "runtime-1".to_owned(),
                logical_owner_id: "communications".to_owned(),
            }),
            event_hub_endpoint: "nats://localhost:4222".to_owned(),
            event_credential_revision: 1,
        }
    }

    #[test]
    fn accepts_exact_domain_runtime_fences() {
        assert_eq!(validate_managed_domain_runtime_configuration(&configuration()), Ok(()));
    }

    #[test]
    fn rejects_provider_configuration_shape_leak() {
        let mut configuration = configuration();
        configuration.logical_owner_id = "communications/provider".to_owned();
        assert_eq!(
            validate_managed_domain_runtime_configuration(&configuration),
            Err(ManagedDomainRuntimeValidationErrorV1::InvalidConfiguration),
        );
    }
}
