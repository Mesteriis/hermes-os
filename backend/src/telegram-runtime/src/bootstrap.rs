//! Clean-room Telegram process admission and provider session bootstrap.

use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use hermes_runtime_protocol::v1::ManagedStorageRuntimeConfigurationV1;
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::StorageVaultRouteContextV1;
use hermes_telegram_api::{TelegramAccountSetup, TelegramCredentialPurpose, TelegramProviderKind};
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::{TdJsonLibrary, TdlibAuthorizationParameters, TdlibError};
use zeroize::Zeroizing;

use crate::managed_control::TelegramManagedRuntimeIdentity;
use crate::vault_credentials::{
    TelegramCredentialRouteError, TelegramVaultRouteContext, resolve_credential_lease,
    resolve_storage_credential,
};
use crate::{TelegramRuntimeAdmission, TelegramRuntimeComposition};

#[derive(Debug)]
pub enum TelegramBootstrapError {
    ManagedRuntime(String),
    Provider(TdlibError),
    CredentialRoute(TelegramCredentialRouteError),
    Persistence(TelegramDurablePersistenceError),
    InvalidStorageTopology,
    AdmissionMismatch,
    UnsupportedProvider,
    MissingApiHash,
}

pub struct TelegramAdmittedRuntime {
    pub identity: TelegramManagedRuntimeIdentity,
    pub control_channel: UnixStream,
    pub account_id: String,
    pub composition: TelegramRuntimeComposition,
    pub durable: TelegramDurablePersistence,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    library: TdJsonLibrary,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    runtime_instance_id: &str,
    api_id: i64,
    account_id: &str,
    provider_kind: TelegramProviderKind,
    database_directory: PathBuf,
    admission: &TelegramRuntimeAdmission,
    vault_context: TelegramVaultRouteContext,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
) -> Result<TelegramAdmittedRuntime, TelegramBootstrapError> {
    if provider_kind != TelegramProviderKind::User {
        return Err(TelegramBootstrapError::UnsupportedProvider);
    }
    if admission.runtime_instance_id != runtime_instance_id
        || admission.vault_runtime_generation != vault_context.vault_runtime_generation
    {
        return Err(TelegramBootstrapError::AdmissionMismatch);
    }
    let (identity, mut control_channel) = TelegramManagedRuntimeIdentity::open_inherited(
        descriptor_bytes,
        settings_schema_bytes,
        runtime_instance_id,
    )
    .map_err(TelegramBootstrapError::ManagedRuntime)?;
    if identity.registration_id() != admission.module_registration_id
        || identity.runtime_generation() != admission.runtime_generation
        || identity.grant_epoch() != admission.grant_epoch
    {
        return Err(TelegramBootstrapError::AdmissionMismatch);
    }

    let mut api_hash: Option<Zeroizing<Vec<u8>>> = None;
    let mut session_encryption_key: Option<Zeroizing<Vec<u8>>> = None;
    for lease_binding in &admission.credential_leases {
        let secret =
            resolve_credential_lease(&mut control_channel, vault_context, &lease_binding.lease)
                .map_err(TelegramBootstrapError::CredentialRoute)?;
        match lease_binding.binding.purpose {
            TelegramCredentialPurpose::ApiHash => {
                if api_hash.replace(secret).is_some() {
                    return Err(TelegramBootstrapError::AdmissionMismatch);
                }
            }
            TelegramCredentialPurpose::SessionEncryptionKey => {
                if session_encryption_key.replace(secret).is_some() {
                    return Err(TelegramBootstrapError::AdmissionMismatch);
                }
            }
            TelegramCredentialPurpose::BotToken => {
                return Err(TelegramBootstrapError::UnsupportedProvider);
            }
        }
    }
    let api_hash = api_hash.ok_or(TelegramBootstrapError::MissingApiHash)?;
    let storage_binding = storage_binding_from_configuration(&storage_configuration, &identity)?;
    let storage_vault_public_key: [u8; 32] = storage_configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let storage_vault_context = StorageVaultRouteContextV1::new(
        storage_configuration.vault_instance_id.clone(),
        storage_configuration.vault_runtime_generation,
        storage_vault_public_key,
    )
    .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let storage_password = resolve_storage_credential(
        control_channel.try_clone().map_err(|_| {
            TelegramBootstrapError::CredentialRoute(TelegramCredentialRouteError::Unavailable)
        })?,
        &storage_binding,
        storage_vault_context,
    )
    .await
    .map_err(|_| {
        TelegramBootstrapError::CredentialRoute(TelegramCredentialRouteError::Unavailable)
    })?;
    let storage_password = std::str::from_utf8(&storage_password).map_err(|_| {
        TelegramBootstrapError::CredentialRoute(TelegramCredentialRouteError::Rejected)
    })?;
    let durable = TelegramDurablePersistence::connect_runtime(
        &storage_binding,
        &storage_configuration.database_id,
        &storage_configuration.pgbouncer_host,
        storage_configuration.pgbouncer_port,
        storage_password,
    )
    .await
    .map_err(TelegramBootstrapError::Persistence)?;
    durable
        .initialize()
        .await
        .map_err(TelegramBootstrapError::Persistence)?;
    let parameters = TdlibAuthorizationParameters::from_secret_material(
        api_id,
        api_hash,
        database_directory,
        session_encryption_key,
    )
    .map_err(TelegramBootstrapError::Provider)?;
    let account_setup = TelegramAccountSetup {
        account_id: account_id.to_owned(),
        provider_kind,
        display_name: account_id.to_owned(),
        external_account_id: account_id.to_owned(),
        credentials: admission
            .credential_leases
            .iter()
            .map(|binding| binding.binding.clone())
            .collect(),
        qr_authorized: false,
    };
    let mut composition =
        TelegramRuntimeComposition::new_with_account_setup(library, account_setup, parameters)
            .map_err(TelegramBootstrapError::Provider)?;
    composition.set_admission(admission.clone());
    Ok(TelegramAdmittedRuntime {
        identity,
        control_channel,
        account_id: account_id.to_owned(),
        composition,
        durable,
    })
}

fn storage_binding_from_configuration(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    identity: &TelegramManagedRuntimeIdentity,
) -> Result<StorageBindingV1, TelegramBootstrapError> {
    if configuration.runtime_instance_id != identity.runtime_instance_id()
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(TelegramBootstrapError::AdmissionMismatch);
    }
    let identity_value = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        identity.registration_id().to_owned(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let max_connections = u16::try_from(configuration.max_connections)
        .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let budgets =
        StorageEffectiveBudgetsV1::new(max_connections, configuration.statement_timeout_millis)
            .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let digest: [u8; 32] = configuration
        .storage_bundle_digest
        .as_slice()
        .try_into()
        .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        digest,
    )
    .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)?;
    StorageBindingV1::new(identity_value, fences, access)
        .map_err(|_| TelegramBootstrapError::InvalidStorageTopology)
}
