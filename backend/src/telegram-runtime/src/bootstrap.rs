//! Clean-room Telegram process admission and provider session bootstrap.

use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access,
};
use hermes_managed_vault_client::{
    ManagedProviderCredentialClientV1, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1,
};
use hermes_runtime_protocol::v1::ManagedStorageRuntimeConfigurationV1;
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::StorageVaultRouteContextV1;
use hermes_telegram_api::{TelegramAccountSetup, TelegramCredentialPurpose, TelegramProviderKind};
use hermes_telegram_core::credential_lease_purpose_for_purpose;
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::{TdJsonLibrary, TdlibAuthorizationParameters, TdlibError};
use hermes_vault_protocol::{DEFAULT_LEASE_TTL_SECONDS, SecretClassV1};

use crate::managed_control::TelegramManagedRuntimeIdentity;
use crate::communications_outbox::{
    TelegramCommunicationsOutboxRelayError, relay_communications_outbox_once,
};
use crate::vault_credentials::{
    TelegramCredentialRouteError, resolve_storage_credential,
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
    EventHub,
}

pub struct TelegramAdmittedRuntime {
    pub identity: TelegramManagedRuntimeIdentity,
    pub control_channel: UnixStream,
    pub account_id: String,
    pub composition: TelegramRuntimeComposition,
    pub durable: TelegramDurablePersistence,
    pub(crate) event_connection: RuntimeJetStreamConnection,
    pub(crate) event_publish_permit: RuntimePublishPermitV1,
}

/// Resources owned by the long-lived provider polling loop after admission.
pub struct TelegramAdmittedProviderLoop {
    pub control_channel: UnixStream,
    pub account_id: String,
    pub composition: TelegramRuntimeComposition,
    pub durable: TelegramDurablePersistence,
    pub(crate) event_connection: RuntimeJetStreamConnection,
    pub(crate) event_publish_permit: RuntimePublishPermitV1,
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
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<TelegramAdmittedRuntime, TelegramBootstrapError> {
    if provider_kind != TelegramProviderKind::User {
        return Err(TelegramBootstrapError::UnsupportedProvider);
    }
    if admission.runtime_instance_id != runtime_instance_id
        || event_hub_endpoint.trim().is_empty()
        || event_credential_revision == 0
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

    let provider_context = provider_credential_context(admission, &storage_configuration)?;
    let mut provider_credentials = ManagedProviderCredentialClientV1::new(
        control_channel.try_clone().map_err(|_| TelegramBootstrapError::CredentialRoute(TelegramCredentialRouteError::Unavailable))?,
    );
    if admission.api_hash_revision == 0 || admission.session_encryption_key_revision == 0 {
        return Err(TelegramBootstrapError::AdmissionMismatch);
    }
    let api_hash_purpose = credential_lease_purpose_for_purpose(account_id, &admission.configuration_instance_id, TelegramCredentialPurpose::ApiHash)
        .map_err(|_| TelegramBootstrapError::AdmissionMismatch)?;
    let api_hash = provider_credentials.resolve(&provider_context, &admission.configuration_instance_id, api_hash_purpose.purpose_id(), admission.api_hash_revision, DEFAULT_LEASE_TTL_SECONDS, SecretClassV1::ProviderCredential)
        .map_err(map_provider_credential_error)?;
    let session_purpose = credential_lease_purpose_for_purpose(account_id, &admission.configuration_instance_id, TelegramCredentialPurpose::SessionEncryptionKey)
        .map_err(|_| TelegramBootstrapError::AdmissionMismatch)?;
    let session_encryption_key = Some(provider_credentials.resolve(&provider_context, &admission.configuration_instance_id, session_purpose.purpose_id(), admission.session_encryption_key_revision, DEFAULT_LEASE_TTL_SECONDS, SecretClassV1::SessionStoreKey)
        .map_err(map_provider_credential_error)?);
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
    let event_access = request_managed_runtime_event_access(
        &mut control_channel,
        &admission.logical_owner_id,
        identity.registration_id(),
        identity.runtime_instance_id(),
        identity.runtime_generation(),
        identity.grant_epoch(),
        event_credential_revision,
    )
    .map_err(|_| TelegramBootstrapError::EventHub)?;
    let event_identity = RuntimeNatsIdentity::new(
        identity.runtime_instance_id(),
        identity.runtime_generation(),
        identity.grant_epoch(),
    )
    .map_err(|_| TelegramBootstrapError::EventHub)?;
    let event_publish_permit = event_access
        .publish_permit(
            identity.registration_id(),
            identity.runtime_instance_id(),
            identity.runtime_generation(),
            identity.grant_epoch(),
        )
        .map_err(|_| TelegramBootstrapError::EventHub)?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        event_identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|_| TelegramBootstrapError::EventHub)?;
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
        credentials: Vec::new(),
        qr_authorized: true,
    };
    let mut composition =
        TelegramRuntimeComposition::new_with_account_setup(library, account_setup, parameters)
            .map_err(TelegramBootstrapError::Provider)?;
    composition.set_admission(admission.clone());
    control_channel
        .set_nonblocking(true)
        .map_err(|_| TelegramBootstrapError::ManagedRuntime("Telegram managed-runtime channel is unavailable".to_owned()))?;
    Ok(TelegramAdmittedRuntime {
        identity,
        control_channel,
        account_id: account_id.to_owned(),
        composition,
        durable,
        event_connection,
        event_publish_permit,
    })
}

fn provider_credential_context(
    admission: &TelegramRuntimeAdmission,
    configuration: &ManagedStorageRuntimeConfigurationV1,
) -> Result<ManagedProviderCredentialContextV1, TelegramBootstrapError> {
    let vault_public_key_x25519 = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| TelegramBootstrapError::AdmissionMismatch)?;
    if configuration.vault_runtime_generation != admission.vault_runtime_generation {
        return Err(TelegramBootstrapError::AdmissionMismatch);
    }
    Ok(ManagedProviderCredentialContextV1 {
        vault_instance_id: configuration.vault_instance_id.clone(),
        vault_runtime_generation: configuration.vault_runtime_generation,
        vault_public_key_x25519,
        logical_owner_id: admission.logical_owner_id.clone(),
        registration_id: admission.module_registration_id.clone(),
        runtime_instance_id: admission.runtime_instance_id.clone(),
        runtime_generation: admission.runtime_generation,
        grant_epoch: admission.grant_epoch,
    })
}

fn map_provider_credential_error(
    error: ManagedProviderCredentialErrorV1,
) -> TelegramBootstrapError {
    let route_error = match error {
        ManagedProviderCredentialErrorV1::Unavailable => TelegramCredentialRouteError::Unavailable,
        ManagedProviderCredentialErrorV1::InvalidContext | ManagedProviderCredentialErrorV1::Rejected => TelegramCredentialRouteError::Rejected,
    };
    TelegramBootstrapError::CredentialRoute(route_error)
}

impl TelegramAdmittedRuntime {
    #[must_use]
    pub fn into_provider_loop(self) -> TelegramAdmittedProviderLoop {
        TelegramAdmittedProviderLoop {
            control_channel: self.control_channel,
            account_id: self.account_id,
            composition: self.composition,
            durable: self.durable,
            event_connection: self.event_connection,
            event_publish_permit: self.event_publish_permit,
        }
    }

    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, TelegramCommunicationsOutboxRelayError> {
        relay_communications_outbox_once(
            &self.durable,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }
}

impl TelegramAdmittedProviderLoop {
    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, TelegramCommunicationsOutboxRelayError> {
        relay_communications_outbox_once(
            &self.durable,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }
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
