//! Validation for the ephemeral managed Storage runtime configuration.

use std::path::{Component, Path};

use crate::{
    v1::StorageRuntimeConfigurationV1,
    validation::{validate_storage_binding_message, validate_storage_runtime_topology},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRuntimeConfigurationErrorV1 {
    InvalidTopology,
    InvalidVaultContext,
    InvalidBinding,
    InvalidPgBouncerConfigPath,
    InvalidPgBouncerAuthPath,
}

pub fn validate_storage_runtime_configuration(
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<(), StorageRuntimeConfigurationErrorV1> {
    let topology = configuration
        .topology
        .as_ref()
        .ok_or(StorageRuntimeConfigurationErrorV1::InvalidTopology)?;
    validate_storage_runtime_topology(topology)
        .map_err(|_| StorageRuntimeConfigurationErrorV1::InvalidTopology)?;
    valid_vault_context(configuration)
        .then_some(())
        .ok_or(StorageRuntimeConfigurationErrorV1::InvalidVaultContext)?;
    validate_bundles(configuration)?;
    validate_bindings(configuration, topology)
}

fn validate_bundles(
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<(), StorageRuntimeConfigurationErrorV1> {
    configuration.desired_bundles.iter().try_for_each(|bundle| {
        crate::validation::validate_storage_bundle(bundle)
            .map_err(|_| StorageRuntimeConfigurationErrorV1::InvalidBinding)
    })
}

fn validate_bindings(
    configuration: &StorageRuntimeConfigurationV1,
    topology: &crate::v1::StorageRuntimeTopologyV1,
) -> Result<(), StorageRuntimeConfigurationErrorV1> {
    if configuration.desired_bindings.is_empty() {
        return (configuration.pgbouncer_database_config_path.is_empty()
            || valid_config_path(&configuration.pgbouncer_database_config_path))
        .then_some(())
        .ok_or(StorageRuntimeConfigurationErrorV1::InvalidPgBouncerConfigPath)
        .and_then(|()| {
            (configuration.pgbouncer_auth_file_path.is_empty()
                || valid_config_path(&configuration.pgbouncer_auth_file_path))
            .then_some(())
            .ok_or(StorageRuntimeConfigurationErrorV1::InvalidPgBouncerAuthPath)
        });
    }
    valid_config_path(&configuration.pgbouncer_database_config_path)
        .then_some(())
        .ok_or(StorageRuntimeConfigurationErrorV1::InvalidPgBouncerConfigPath)?;
    valid_config_path(&configuration.pgbouncer_auth_file_path)
        .then_some(())
        .ok_or(StorageRuntimeConfigurationErrorV1::InvalidPgBouncerAuthPath)?;
    for binding in &configuration.desired_bindings {
        validate_storage_binding_message(binding)
            .map_err(|_| StorageRuntimeConfigurationErrorV1::InvalidBinding)?;
        binding_matches_topology(binding, topology)
            .then_some(())
            .ok_or(StorageRuntimeConfigurationErrorV1::InvalidBinding)?;
    }
    unique_aliases(&configuration.desired_bindings)
        .then_some(())
        .ok_or(StorageRuntimeConfigurationErrorV1::InvalidBinding)
}

fn binding_matches_topology(
    binding: &crate::v1::StorageBindingV1,
    topology: &crate::v1::StorageRuntimeTopologyV1,
) -> bool {
    binding.storage_instance_id == topology.storage_instance_id
        && binding.storage_generation == topology.storage_generation
        && binding.database_id == topology.database_id
}

fn unique_aliases(bindings: &[crate::v1::StorageBindingV1]) -> bool {
    let mut aliases: Vec<&str> = bindings
        .iter()
        .map(|binding| binding.pool_alias.as_str())
        .collect();
    aliases.sort_unstable();
    aliases.windows(2).all(|pair| pair[0] != pair[1])
}

fn valid_config_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4_096
        && Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Normal(_)))
}

fn valid_vault_context(configuration: &StorageRuntimeConfigurationV1) -> bool {
    configuration.vault_runtime_generation > 0
        && valid_identifier(&configuration.vault_instance_id)
        && configuration.vault_hpke_public_key_x25519.len() == 32
        && configuration
            .vault_hpke_public_key_x25519
            .iter()
            .any(|byte| *byte != 0)
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}
