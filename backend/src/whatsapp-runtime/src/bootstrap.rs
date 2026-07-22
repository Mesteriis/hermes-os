//! WhatsApp admitted runtime bootstrap. No secret enters through environment.

use std::os::unix::net::UnixStream;

use hermes_runtime_protocol::v1::ManagedStorageRuntimeConfigurationV1;
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1,
    StorageBindingV1, StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::StorageVaultRouteContextV1;
use hermes_whatsapp_persistence::{
    WhatsAppDurablePersistence, WhatsAppDurablePersistenceError,
};
use hermes_vault_protocol::CredentialLeaseV1;
use zeroize::Zeroizing;

use crate::managed_control::WhatsAppManagedRuntimeIdentity;
use crate::vault_credentials::{
    resolve_credential_lease, resolve_storage_credential, WhatsAppCredentialRouteError,
    WhatsAppVaultRouteContext,
};
use hermes_whatsapp_api::WhatsAppCredentialBinding;

#[derive(Debug)]
pub enum WhatsAppBootstrapError {
    ManagedRuntime(String),
    CredentialRoute,
    AdmissionMismatch,
    InvalidStorageTopology,
    Persistence(WhatsAppDurablePersistenceError),
}

pub struct WhatsAppAdmittedRuntime {
    pub identity: WhatsAppManagedRuntimeIdentity,
    pub control_channel: UnixStream,
    pub durable: WhatsAppDurablePersistence,
}

impl WhatsAppAdmittedRuntime {
    pub fn resolve_session_credential(
        &mut self,
        account_id: &str,
        logical_owner_id: &str,
        configuration_instance_id: &str,
        binding: &WhatsAppCredentialBinding,
        lease: &CredentialLeaseV1,
        vault_context: WhatsAppVaultRouteContext,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, WhatsAppCredentialRouteError> {
        hermes_whatsapp_core::validate_credential_lease(
            account_id,
            logical_owner_id,
            configuration_instance_id,
            self.identity.registration_id(),
            self.identity.runtime_instance_id(),
            self.identity.runtime_generation(),
            self.identity.grant_epoch(),
            vault_context.vault_runtime_generation,
            binding,
            lease,
            now_unix_seconds,
        )
        .map_err(|_| WhatsAppCredentialRouteError::InvalidLease)?;
        resolve_credential_lease(&mut self.control_channel, vault_context, lease)
    }
}

pub async fn open_admitted_runtime(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    runtime_instance_id: &str,
    configuration: ManagedStorageRuntimeConfigurationV1,
) -> Result<WhatsAppAdmittedRuntime, WhatsAppBootstrapError> {
    let (identity, mut control_channel) = WhatsAppManagedRuntimeIdentity::open_inherited(
        descriptor_bytes,
        settings_schema_bytes,
        runtime_instance_id,
    )
    .map_err(WhatsAppBootstrapError::ManagedRuntime)?;
    let storage_binding = storage_binding_from_configuration(&configuration, &identity)?;
    let vault_public_key: [u8; 32] = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    let vault_context = StorageVaultRouteContextV1::new(
        configuration.vault_instance_id.clone(),
        configuration.vault_runtime_generation,
        vault_public_key,
    )
    .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    if identity.runtime_instance_id() != runtime_instance_id
        || identity.runtime_generation() == 0
        || identity.grant_epoch() == 0
        || configuration.runtime_instance_id != identity.runtime_instance_id()
    {
        return Err(WhatsAppBootstrapError::AdmissionMismatch);
    }
    if configuration.pgbouncer_host.is_empty() || configuration.pgbouncer_port == 0 {
        return Err(WhatsAppBootstrapError::InvalidStorageTopology);
    }
    let password: Zeroizing<Vec<u8>> = resolve_storage_credential(
        control_channel.try_clone().map_err(|_| WhatsAppBootstrapError::CredentialRoute)?,
        &storage_binding,
        vault_context,
    )
    .await
    .map_err(|_| WhatsAppBootstrapError::CredentialRoute)?;
    let password = std::str::from_utf8(&password)
        .map_err(|_| WhatsAppBootstrapError::CredentialRoute)?;
    let durable = WhatsAppDurablePersistence::connect_runtime(
        &storage_binding,
        &configuration.database_id,
        &configuration.pgbouncer_host,
        configuration.pgbouncer_port,
        password,
    )
        .await
        .map_err(WhatsAppBootstrapError::Persistence)?;
    durable
        .initialize()
        .await
        .map_err(WhatsAppBootstrapError::Persistence)?;
    Ok(WhatsAppAdmittedRuntime {
        identity,
        control_channel,
        durable,
    })
}

fn storage_binding_from_configuration(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    identity: &WhatsAppManagedRuntimeIdentity,
) -> Result<StorageBindingV1, WhatsAppBootstrapError> {
    if configuration.runtime_instance_id != identity.runtime_instance_id()
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(WhatsAppBootstrapError::AdmissionMismatch);
    }
    let identity_value = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        identity.registration_id().to_owned(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    let max_connections = u16::try_from(configuration.max_connections)
        .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        max_connections,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    let digest: [u8; 32] = configuration
        .storage_bundle_digest
        .as_slice()
        .try_into()
        .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        digest,
    )
    .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)?;
    StorageBindingV1::new(identity_value, fences, access)
        .map_err(|_| WhatsAppBootstrapError::InvalidStorageTopology)
}
