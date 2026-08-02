use crate::v1::ManagedWorkflowRuntimeConfigurationV1;

use super::managed_runtime_artifact::valid_runtime_artifacts;

const MAX_IDENTIFIER_BYTES: usize = 256;
const MAX_ENDPOINT_BYTES: usize = 512;
const MAX_CONFIGURATION_INSTANCES: usize = 32;
const MAX_SETTINGS_SNAPSHOT_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedWorkflowRuntimeValidationErrorV1 {
    InvalidConfiguration,
}

pub fn validate_managed_workflow_runtime_configuration(
    configuration: &ManagedWorkflowRuntimeConfigurationV1,
) -> Result<(), ManagedWorkflowRuntimeValidationErrorV1> {
    let storage = configuration
        .storage
        .as_ref()
        .ok_or(ManagedWorkflowRuntimeValidationErrorV1::InvalidConfiguration)?;
    if configuration.major != 1
        || !valid_identifier(&configuration.logical_owner_id)
        || !valid_identifier(&configuration.registration_id)
        || !valid_identifier(&configuration.runtime_instance_id)
        || configuration.runtime_generation == 0
        || configuration.grant_epoch == 0
        || !valid_event_hub_endpoint(&configuration.event_hub_endpoint)
        || configuration.event_credential_revision == 0
        || storage.runtime_instance_id != configuration.runtime_instance_id
        || !valid_storage_configuration(storage)
        || !valid_runtime_artifacts(&configuration.runtime_artifacts)
        || !valid_configuration_instances(configuration)
    {
        return Err(ManagedWorkflowRuntimeValidationErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn valid_configuration_instances(configuration: &ManagedWorkflowRuntimeConfigurationV1) -> bool {
    if configuration.configuration_instances.is_empty() {
        return configuration.configuration_instance_id.is_empty()
            && configuration.settings_revision == 0;
    }
    if configuration.configuration_instances.len() > MAX_CONFIGURATION_INSTANCES
        || !valid_identifier(&configuration.configuration_instance_id)
        || configuration.settings_revision == 0
    {
        return false;
    }
    let mut previous_id = "";
    let mut selected_revision = None;
    for instance in &configuration.configuration_instances {
        if !valid_identifier(&instance.configuration_instance_id)
            || instance.configuration_instance_id.as_str() <= previous_id
            || instance.settings_snapshot_bytes.is_empty()
            || instance.settings_snapshot_bytes.len() > MAX_SETTINGS_SNAPSHOT_BYTES
        {
            return false;
        }
        let Ok(snapshot) = crate::validation::descriptor::decode_settings_snapshot_v1(
            &instance.settings_snapshot_bytes,
        ) else {
            return false;
        };
        if snapshot.target_id != instance.configuration_instance_id || snapshot.revision == 0 {
            return false;
        }
        if instance.configuration_instance_id == configuration.configuration_instance_id {
            selected_revision = Some(snapshot.revision);
        }
        previous_id = &instance.configuration_instance_id;
    }
    selected_revision == Some(configuration.settings_revision)
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
        && valid_identifier(&storage.logical_owner_id)
        && storage.logical_owner_id == storage.owner
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
    use crate::v1::ManagedStorageRuntimeConfigurationV1;

    use super::*;

    fn configuration() -> ManagedWorkflowRuntimeConfigurationV1 {
        ManagedWorkflowRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: "owner-1".to_owned(),
            registration_id: "communications-export-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
            storage: Some(ManagedStorageRuntimeConfigurationV1 {
                database_id: "communications_export".to_owned(),
                pgbouncer_host: "localhost".to_owned(),
                pgbouncer_port: 6432,
                runtime_principal: "communications_export_runtime".to_owned(),
                storage_generation: 1,
                credential_revision: 1,
                storage_instance_id: "storage-1".to_owned(),
                owner: "communications_export".to_owned(),
                role_epoch: 1,
                pool_alias: "communications_export".to_owned(),
                max_connections: 1,
                statement_timeout_millis: 1,
                storage_bundle_revision: 1,
                storage_bundle_digest: vec![1; 32],
                vault_instance_id: "vault-1".to_owned(),
                vault_runtime_generation: 1,
                vault_hpke_public_key_x25519: vec![1; 32],
                runtime_instance_id: "runtime-1".to_owned(),
                logical_owner_id: "communications_export".to_owned(),
            }),
            event_hub_endpoint: "nats://localhost:4222".to_owned(),
            event_credential_revision: 1,
            runtime_artifacts: Vec::new(),
            configuration_instance_id: String::new(),
            settings_revision: 0,
            configuration_instances: Vec::new(),
        }
    }

    #[test]
    fn accepts_distinct_human_and_storage_owner_scopes() {
        assert_eq!(
            validate_managed_workflow_runtime_configuration(&configuration()),
            Ok(())
        );
    }

    #[test]
    fn rejects_storage_namespace_that_differs_from_binding_owner() {
        let mut configuration = configuration();
        configuration
            .storage
            .as_mut()
            .expect("storage configuration")
            .logical_owner_id = "another_workflow".to_owned();
        assert_eq!(
            validate_managed_workflow_runtime_configuration(&configuration),
            Err(ManagedWorkflowRuntimeValidationErrorV1::InvalidConfiguration)
        );
    }

    #[test]
    fn rejects_provider_configuration_shape_leak() {
        let mut configuration = configuration();
        configuration.logical_owner_id = "communications_export/provider".to_owned();
        assert_eq!(
            validate_managed_workflow_runtime_configuration(&configuration),
            Err(ManagedWorkflowRuntimeValidationErrorV1::InvalidConfiguration)
        );
    }

    #[test]
    fn accepts_exact_ordered_workflow_settings_catalog() {
        use crate::v1::{ManagedWorkflowConfigurationInstanceV1, SettingsSnapshotV1};
        use prost::Message;

        let mut configuration = configuration();
        configuration.configuration_instance_id = "sync-a".to_owned();
        configuration.settings_revision = 2;
        configuration.configuration_instances = vec![ManagedWorkflowConfigurationInstanceV1 {
            configuration_instance_id: "sync-a".to_owned(),
            settings_snapshot_bytes: SettingsSnapshotV1 {
                target_id: "sync-a".to_owned(),
                revision: 2,
                values: Vec::new(),
            }
            .encode_to_vec(),
        }];
        assert_eq!(
            validate_managed_workflow_runtime_configuration(&configuration),
            Ok(())
        );
        configuration.settings_revision = 3;
        assert_eq!(
            validate_managed_workflow_runtime_configuration(&configuration),
            Err(ManagedWorkflowRuntimeValidationErrorV1::InvalidConfiguration)
        );
    }
}
