use crate::v1::ManagedEngineRuntimeConfigurationV1;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_ENDPOINT_BYTES: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedEngineRuntimeValidationErrorV1 {
    InvalidConfiguration,
}

pub fn validate_managed_engine_runtime_configuration(
    configuration: &ManagedEngineRuntimeConfigurationV1,
) -> Result<(), ManagedEngineRuntimeValidationErrorV1> {
    let storage = configuration
        .storage
        .as_ref()
        .ok_or(ManagedEngineRuntimeValidationErrorV1::InvalidConfiguration)?;
    if configuration.major != 1
        || !valid_identifier(&configuration.logical_owner_id)
        || !valid_identifier(&configuration.registration_id)
        || !valid_identifier(&configuration.runtime_instance_id)
        || configuration.runtime_generation == 0
        || configuration.grant_epoch == 0
        || !valid_event_hub_configuration(
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        )
        || configuration.settings_revision == 0
        || storage.logical_owner_id != configuration.logical_owner_id
        || storage.runtime_instance_id != configuration.runtime_instance_id
        || !valid_storage_configuration(storage)
    {
        return Err(ManagedEngineRuntimeValidationErrorV1::InvalidConfiguration);
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
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_event_hub_endpoint(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ENDPOINT_BYTES
        && value.starts_with("nats://")
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn valid_event_hub_configuration(endpoint: &str, credential_revision: u64) -> bool {
    (endpoint.is_empty() && credential_revision == 0)
        || (credential_revision != 0 && valid_event_hub_endpoint(endpoint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::ManagedStorageRuntimeConfigurationV1;

    fn configuration() -> ManagedEngineRuntimeConfigurationV1 {
        ManagedEngineRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: "attachment_security".to_owned(),
            registration_id: "attachment-security-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 7,
            grant_epoch: 11,
            storage: Some(ManagedStorageRuntimeConfigurationV1 {
                database_id: "attachment_security".to_owned(),
                pgbouncer_host: "localhost".to_owned(),
                pgbouncer_port: 6432,
                runtime_principal: "attachment_security_runtime".to_owned(),
                storage_generation: 1,
                credential_revision: 3,
                storage_instance_id: "storage-1".to_owned(),
                owner: "attachment_security".to_owned(),
                role_epoch: 2,
                pool_alias: "attachment_security".to_owned(),
                max_connections: 2,
                statement_timeout_millis: 1_000,
                storage_bundle_revision: 5,
                storage_bundle_digest: vec![7; 32],
                vault_instance_id: "vault-1".to_owned(),
                vault_runtime_generation: 11,
                vault_hpke_public_key_x25519: vec![9; 32],
                runtime_instance_id: "runtime-1".to_owned(),
                logical_owner_id: "attachment_security".to_owned(),
            }),
            event_hub_endpoint: "nats://127.0.0.1:4222".to_owned(),
            event_credential_revision: 13,
            settings_revision: 17,
        }
    }

    #[test]
    fn accepts_exact_engine_runtime_fences() {
        assert_eq!(
            validate_managed_engine_runtime_configuration(&configuration()),
            Ok(())
        );
    }

    #[test]
    fn accepts_an_exact_eventless_engine_configuration() {
        let mut configuration = configuration();
        configuration.event_hub_endpoint.clear();
        configuration.event_credential_revision = 0;
        assert_eq!(
            validate_managed_engine_runtime_configuration(&configuration),
            Ok(())
        );
    }

    #[test]
    fn rejects_a_partial_event_hub_configuration() {
        let mut endpoint_without_revision = configuration();
        endpoint_without_revision.event_credential_revision = 0;
        assert_eq!(
            validate_managed_engine_runtime_configuration(&endpoint_without_revision),
            Err(ManagedEngineRuntimeValidationErrorV1::InvalidConfiguration)
        );

        let mut revision_without_endpoint = configuration();
        revision_without_endpoint.event_hub_endpoint.clear();
        assert_eq!(
            validate_managed_engine_runtime_configuration(&revision_without_endpoint),
            Err(ManagedEngineRuntimeValidationErrorV1::InvalidConfiguration)
        );
    }

    #[test]
    fn rejects_missing_settings_identity() {
        let mut configuration = configuration();
        configuration.settings_revision = 0;
        assert_eq!(
            validate_managed_engine_runtime_configuration(&configuration),
            Err(ManagedEngineRuntimeValidationErrorV1::InvalidConfiguration)
        );
    }

    #[test]
    fn rejects_cross_owner_storage_binding() {
        let mut configuration = configuration();
        configuration
            .storage
            .as_mut()
            .expect("storage")
            .logical_owner_id = "communications".to_owned();
        assert_eq!(
            validate_managed_engine_runtime_configuration(&configuration),
            Err(ManagedEngineRuntimeValidationErrorV1::InvalidConfiguration)
        );
    }
}
