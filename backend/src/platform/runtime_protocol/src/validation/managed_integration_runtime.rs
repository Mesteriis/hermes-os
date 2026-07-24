//! Structural validation for a Kernel-staged provider integration runtime.

use std::path::{Component, Path};

use crate::v1::ManagedIntegrationRuntimeConfigurationV1;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_ENDPOINT_BYTES: usize = 1_024;
const MAX_PRIVATE_PATH_BYTES: usize = 4_096;
const MAX_RUNTIME_ARTIFACTS: usize = 16;
const MAX_RUNTIME_ARTIFACT_BYTES: u64 = 1024 * 1024 * 1024;

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
        || !valid_runtime_artifacts(&configuration.runtime_artifacts)
        || !configuration
            .integration_state_root
            .as_ref()
            .is_none_or(|root| {
                root.state_generation != 0
                    && root.state_layout_revision != 0
                    && valid_private_path(&root.root_path)
            })
    {
        return Err(ManagedIntegrationRuntimeValidationErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn valid_runtime_artifacts(artifacts: &[crate::v1::ManagedRuntimeArtifactBindingV1]) -> bool {
    if artifacts.len() > MAX_RUNTIME_ARTIFACTS {
        return false;
    }
    let mut previous = "";
    for artifact in artifacts {
        if !valid_identifier(&artifact.artifact_id)
            || artifact.artifact_id.as_str() <= previous
            || crate::v1::RuntimeArtifactUseV1::try_from(artifact.r#use)
                .ok()
                .is_none_or(|value| value != crate::v1::RuntimeArtifactUseV1::NativeDynamicLibrary)
            || !valid_private_path(&artifact.staged_path)
            || !(1..=MAX_RUNTIME_ARTIFACT_BYTES).contains(&artifact.size_bytes)
            || artifact.sha256.len() != 32
        {
            return false;
        }
        previous = &artifact.artifact_id;
    }
    true
}

fn valid_private_path(value: &str) -> bool {
    if value.is_empty()
        || value.len() > MAX_PRIVATE_PATH_BYTES
        || value.contains(['\0', '\n', '\r'])
    {
        return false;
    }
    let path = Path::new(value);
    path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Normal(_)))
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
    use super::validate_managed_integration_runtime_configuration;
    use crate::v1::{
        IntegrationStateRootV1, ManagedIntegrationRuntimeConfigurationV1,
        ManagedRuntimeArtifactBindingV1, ManagedStorageRuntimeConfigurationV1,
        RuntimeArtifactUseV1,
    };

    #[test]
    fn staged_artifacts_and_state_root_are_bounded_private_bindings() {
        let mut configuration = valid_configuration();
        assert_eq!(
            validate_managed_integration_runtime_configuration(&configuration),
            Ok(())
        );

        configuration.runtime_artifacts[0].sha256.clear();
        assert!(validate_managed_integration_runtime_configuration(&configuration).is_err());

        let mut configuration = valid_configuration();
        configuration
            .integration_state_root
            .as_mut()
            .expect("state root")
            .root_path = "/private/hermes/state/../other".to_owned();
        assert!(validate_managed_integration_runtime_configuration(&configuration).is_err());
    }

    fn valid_configuration() -> ManagedIntegrationRuntimeConfigurationV1 {
        ManagedIntegrationRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: "telegram".to_owned(),
            registration_id: "registration-1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
            storage: Some(ManagedStorageRuntimeConfigurationV1 {
                database_id: "telegram".to_owned(),
                pgbouncer_host: "127.0.0.1".to_owned(),
                pgbouncer_port: 6432,
                runtime_principal: "telegram_runtime".to_owned(),
                storage_generation: 1,
                credential_revision: 1,
                storage_instance_id: "storage-1".to_owned(),
                owner: "telegram".to_owned(),
                role_epoch: 1,
                pool_alias: "telegram".to_owned(),
                max_connections: 4,
                statement_timeout_millis: 1_000,
                storage_bundle_revision: 1,
                storage_bundle_digest: vec![7; 32],
                vault_instance_id: "vault-1".to_owned(),
                vault_runtime_generation: 1,
                vault_hpke_public_key_x25519: vec![8; 32],
                runtime_instance_id: "runtime-1".to_owned(),
                logical_owner_id: "telegram".to_owned(),
            }),
            event_hub_endpoint: "nats://127.0.0.1:4222".to_owned(),
            event_credential_revision: 1,
            configuration_instance_id: "account-1".to_owned(),
            runtime_artifacts: vec![ManagedRuntimeArtifactBindingV1 {
                artifact_id: "telegram.tdjson.v1".to_owned(),
                r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                staged_path: "/private/hermes/runtime/libtdjson.dylib".to_owned(),
                size_bytes: 1,
                sha256: vec![9; 32],
            }],
            integration_state_root: Some(IntegrationStateRootV1 {
                root_path: "/private/hermes/state/telegram/account-1".to_owned(),
                state_generation: 1,
                state_layout_revision: 1,
            }),
        }
    }
}
