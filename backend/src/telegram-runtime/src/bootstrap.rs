//! Clean-room Telegram process admission and provider session bootstrap.

use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access_v2,
};
use hermes_managed_vault_client::{
    ManagedProviderCredentialClientV2, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1, ManagedProviderCredentialRequestV1,
};
use hermes_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, RejectManagedControlRequestsV2,
    },
    v1::{ManagedIntegrationRuntimeConfigurationV1, ManagedStorageRuntimeConfigurationV1},
    validation::managed_integration_runtime::validate_managed_integration_runtime_configuration,
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::StorageVaultRouteContextV1;
use hermes_telegram_api::client_contract::TELEGRAM_OWNER_ID;
use hermes_telegram_api::{
    TelegramAccountSetup, TelegramCredentialBinding, TelegramCredentialPurpose,
    TelegramProviderKind,
};
use hermes_telegram_automation_persistence::TelegramAutomationPersistence;
use hermes_telegram_calls_persistence::TelegramCallsPersistence;
use hermes_telegram_core::credential_lease_purpose_for_purpose;
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::{TdJsonLibrary, TdlibAuthorizationParameters, TdlibError};
use hermes_vault_protocol::SecretClassV1;

use crate::admission::TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS;
use crate::calls_backfill::complete_calls_realtime_backfill_v1;
use crate::communications_outbox::{
    TelegramCommunicationsOutboxRelayError, relay_communications_outbox_once,
};
use crate::managed_control::TelegramManagedRuntimeIdentity;
use crate::vault_credentials::{TelegramCredentialRouteError, resolve_storage_credential_v2};
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
    CallsBackfill,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramManagedLaunchAdmissionV1 {
    logical_owner_id: String,
    configuration_instance_id: String,
    module_registration_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    vault_runtime_generation: u64,
}

impl TelegramManagedLaunchAdmissionV1 {
    pub fn from_configuration(
        configuration: &ManagedIntegrationRuntimeConfigurationV1,
    ) -> Result<Self, String> {
        validate_managed_integration_runtime_configuration(configuration)
            .map_err(|_| "Telegram runtime configuration is invalid".to_owned())?;
        if configuration.logical_owner_id != TELEGRAM_OWNER_ID {
            return Err("Telegram runtime configuration is invalid".to_owned());
        }
        let storage = configuration
            .storage
            .as_ref()
            .ok_or_else(|| "Telegram runtime configuration is invalid".to_owned())?;
        Ok(Self {
            logical_owner_id: configuration.logical_owner_id.clone(),
            configuration_instance_id: configuration.configuration_instance_id.clone(),
            module_registration_id: configuration.registration_id.clone(),
            runtime_instance_id: configuration.runtime_instance_id.clone(),
            runtime_generation: configuration.runtime_generation,
            grant_epoch: configuration.grant_epoch,
            vault_runtime_generation: storage.vault_runtime_generation,
        })
    }
}

pub struct TelegramAdmittedRuntime {
    pub identity: TelegramManagedRuntimeIdentity,
    pub control_channel: ManagedControlChannelV2<UnixStream>,
    pub account_id: String,
    pub composition: TelegramRuntimeComposition,
    pub durable: TelegramDurablePersistence,
    pub automation: TelegramAutomationPersistence,
    pub calls: TelegramCallsPersistence,
    pub(crate) reconfiguration_context: TelegramProviderReconfigurationContextV1,
    pub(crate) event_connection: RuntimeJetStreamConnection,
    pub(crate) event_publish_permit: RuntimePublishPermitV1,
}

/// Resources owned by the long-lived provider polling loop after admission.
pub struct TelegramAdmittedProviderLoop {
    pub control_channel: ManagedControlChannelV2<UnixStream>,
    pub account_id: String,
    pub composition: TelegramRuntimeComposition,
    pub durable: TelegramDurablePersistence,
    pub automation: TelegramAutomationPersistence,
    pub calls: TelegramCallsPersistence,
    pub(crate) reconfiguration_context: TelegramProviderReconfigurationContextV1,
    pub(crate) event_connection: RuntimeJetStreamConnection,
    pub(crate) event_publish_permit: RuntimePublishPermitV1,
}

#[derive(Clone)]
pub(crate) struct TelegramProviderReconfigurationContextV1 {
    provider_credentials: ManagedProviderCredentialContextV1,
    configuration_instance_id: String,
    api_id: i64,
    database_directory: PathBuf,
    api_hash_revision: u64,
    session_encryption_key_revision: u64,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    library: TdJsonLibrary,
    call_media: Box<dyn hermes_telegram_call_media_contract::TelegramCallSignalingMediaPort>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    runtime_instance_id: &str,
    api_id: i64,
    account_id: &str,
    provider_kind: TelegramProviderKind,
    database_directory: PathBuf,
    launch_admission: &TelegramManagedLaunchAdmissionV1,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<TelegramAdmittedRuntime, TelegramBootstrapError> {
    if provider_kind != TelegramProviderKind::User {
        return Err(TelegramBootstrapError::UnsupportedProvider);
    }
    if launch_admission.runtime_instance_id != runtime_instance_id
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
    if identity.registration_id() != launch_admission.module_registration_id
        || identity.runtime_generation() != launch_admission.runtime_generation
        || identity.grant_epoch() != launch_admission.grant_epoch
    {
        return Err(TelegramBootstrapError::AdmissionMismatch);
    }

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
    let (storage_password, returned_control_channel) =
        resolve_storage_credential_v2(control_channel, &storage_binding, storage_vault_context)
            .await
            .map_err(|error| {
                if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_telegram_storage_credential_error={error:?}");
                }
                TelegramBootstrapError::CredentialRoute(TelegramCredentialRouteError::Unavailable)
            })?;
    control_channel = returned_control_channel;
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
    let automation = TelegramAutomationPersistence::new(durable.shared_owner_pool());
    let calls = TelegramCallsPersistence::new(durable.shared_owner_pool());
    complete_calls_realtime_backfill_v1(&calls, &identity)
        .await
        .map_err(|_| TelegramBootstrapError::CallsBackfill)?;
    let (persisted_account, credential_bindings) = durable
        .account(account_id)
        .await
        .map_err(TelegramBootstrapError::Persistence)?
        .filter(|(account, _)| {
            account.account_id == account_id && account.provider_kind == provider_kind
        })
        .ok_or(TelegramBootstrapError::AdmissionMismatch)?;
    let (api_hash_revision, session_encryption_key_revision) =
        credential_revisions(&credential_bindings)?;
    let admission = TelegramRuntimeAdmission {
        logical_owner_id: launch_admission.logical_owner_id.clone(),
        configuration_instance_id: launch_admission.configuration_instance_id.clone(),
        module_registration_id: launch_admission.module_registration_id.clone(),
        runtime_instance_id: launch_admission.runtime_instance_id.clone(),
        runtime_generation: launch_admission.runtime_generation,
        grant_epoch: launch_admission.grant_epoch,
        vault_runtime_generation: launch_admission.vault_runtime_generation,
        api_hash_revision,
        session_encryption_key_revision,
    };
    let provider_context = provider_credential_context(launch_admission, &storage_configuration)?;
    let (api_hash, session_encryption_key) = {
        let mut provider_credentials = ManagedProviderCredentialClientV2::new(&mut control_channel);
        let mut bootstrap_dispatcher = RejectManagedControlRequestsV2;
        let api_hash_purpose = credential_lease_purpose_for_purpose(
            &admission.configuration_instance_id,
            TelegramCredentialPurpose::ApiHash,
        )
        .map_err(|_| TelegramBootstrapError::AdmissionMismatch)?;
        let api_hash = provider_credentials
            .resolve(
                &mut bootstrap_dispatcher,
                &provider_context,
                ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: &admission.configuration_instance_id,
                    purpose_id: api_hash_purpose.purpose_id(),
                    credential_revision: admission.api_hash_revision,
                    ttl_seconds: TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS,
                    secret_class: SecretClassV1::ProviderCredential,
                },
            )
            .map_err(map_provider_credential_error)?;
        let session_purpose = credential_lease_purpose_for_purpose(
            &admission.configuration_instance_id,
            TelegramCredentialPurpose::SessionEncryptionKey,
        )
        .map_err(|_| TelegramBootstrapError::AdmissionMismatch)?;
        let session_encryption_key = provider_credentials
            .resolve(
                &mut bootstrap_dispatcher,
                &provider_context,
                ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: &admission.configuration_instance_id,
                    purpose_id: session_purpose.purpose_id(),
                    credential_revision: admission.session_encryption_key_revision,
                    ttl_seconds: TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS,
                    secret_class: SecretClassV1::SessionStoreKey,
                },
            )
            .map_err(map_provider_credential_error)?;
        (api_hash, Some(session_encryption_key))
    };
    let event_access = request_managed_runtime_event_access_v2(
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
    let reconfiguration_database_directory = database_directory.clone();
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
        display_name: persisted_account.display_name,
        external_account_id: persisted_account.external_account_id,
        credentials: credential_bindings,
        qr_authorized: false,
    };
    let mut composition =
        TelegramRuntimeComposition::new_with_account_setup(library, account_setup, parameters)
            .map_err(TelegramBootstrapError::Provider)?;
    composition.set_admission(admission.clone());
    composition.install_call_media_port(call_media);
    identity
        .signal_ready(&mut control_channel)
        .map_err(TelegramBootstrapError::ManagedRuntime)?;
    control_channel
        .inner_mut()
        .set_nonblocking(true)
        .map_err(|_| {
            TelegramBootstrapError::ManagedRuntime(
                "Telegram managed-runtime channel is unavailable".to_owned(),
            )
        })?;
    Ok(TelegramAdmittedRuntime {
        identity,
        control_channel,
        account_id: account_id.to_owned(),
        composition,
        durable,
        automation,
        calls,
        reconfiguration_context: TelegramProviderReconfigurationContextV1 {
            provider_credentials: provider_context,
            configuration_instance_id: admission.configuration_instance_id,
            api_id,
            database_directory: reconfiguration_database_directory,
            api_hash_revision: admission.api_hash_revision,
            session_encryption_key_revision: admission.session_encryption_key_revision,
        },
        event_connection,
        event_publish_permit,
    })
}

fn credential_revisions(
    bindings: &[TelegramCredentialBinding],
) -> Result<(u64, u64), TelegramBootstrapError> {
    let mut api_hash_revision = None;
    let mut session_encryption_key_revision = None;
    for binding in bindings {
        if binding.revision == 0 {
            return Err(TelegramBootstrapError::AdmissionMismatch);
        }
        let selected = match binding.purpose {
            TelegramCredentialPurpose::ApiHash => &mut api_hash_revision,
            TelegramCredentialPurpose::SessionEncryptionKey => &mut session_encryption_key_revision,
            TelegramCredentialPurpose::BotToken => {
                return Err(TelegramBootstrapError::AdmissionMismatch);
            }
        };
        if selected.replace(binding.revision).is_some() {
            return Err(TelegramBootstrapError::AdmissionMismatch);
        }
    }
    match (api_hash_revision, session_encryption_key_revision) {
        (Some(api_hash_revision), Some(session_encryption_key_revision)) if bindings.len() == 2 => {
            Ok((api_hash_revision, session_encryption_key_revision))
        }
        _ => Err(TelegramBootstrapError::AdmissionMismatch),
    }
}

fn provider_credential_context(
    admission: &TelegramManagedLaunchAdmissionV1,
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
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_telegram_provider_credential_error={error:?}");
    }
    let route_error = match error {
        ManagedProviderCredentialErrorV1::Unavailable => TelegramCredentialRouteError::Unavailable,
        ManagedProviderCredentialErrorV1::InvalidContext
        | ManagedProviderCredentialErrorV1::Rejected => TelegramCredentialRouteError::Rejected,
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
            automation: self.automation,
            calls: self.calls,
            reconfiguration_context: self.reconfiguration_context,
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

pub(crate) fn resolve_provider_reconfiguration_parameters<D>(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut D,
    context: &TelegramProviderReconfigurationContextV1,
) -> Result<TdlibAuthorizationParameters, TelegramCredentialRouteError>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    let api_hash_purpose = credential_lease_purpose_for_purpose(
        &context.configuration_instance_id,
        TelegramCredentialPurpose::ApiHash,
    )
    .map_err(|_| TelegramCredentialRouteError::Rejected)?;
    let session_purpose = credential_lease_purpose_for_purpose(
        &context.configuration_instance_id,
        TelegramCredentialPurpose::SessionEncryptionKey,
    )
    .map_err(|_| TelegramCredentialRouteError::Rejected)?;
    let mut credentials = ManagedProviderCredentialClientV2::new(control_channel);
    let api_hash = credentials
        .resolve(
            dispatcher,
            &context.provider_credentials,
            ManagedProviderCredentialRequestV1 {
                configuration_instance_id: &context.configuration_instance_id,
                purpose_id: api_hash_purpose.purpose_id(),
                credential_revision: context.api_hash_revision,
                ttl_seconds: TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS,
                secret_class: SecretClassV1::ProviderCredential,
            },
        )
        .map_err(map_reconfiguration_credential_error)?;
    let session_encryption_key = credentials
        .resolve(
            dispatcher,
            &context.provider_credentials,
            ManagedProviderCredentialRequestV1 {
                configuration_instance_id: &context.configuration_instance_id,
                purpose_id: session_purpose.purpose_id(),
                credential_revision: context.session_encryption_key_revision,
                ttl_seconds: TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS,
                secret_class: SecretClassV1::SessionStoreKey,
            },
        )
        .map_err(map_reconfiguration_credential_error)?;
    TdlibAuthorizationParameters::from_secret_material(
        context.api_id,
        api_hash,
        context.database_directory.clone(),
        Some(session_encryption_key),
    )
    .map_err(|_| TelegramCredentialRouteError::Rejected)
}

fn map_reconfiguration_credential_error(
    error: ManagedProviderCredentialErrorV1,
) -> TelegramCredentialRouteError {
    match error {
        ManagedProviderCredentialErrorV1::Unavailable => TelegramCredentialRouteError::Unavailable,
        ManagedProviderCredentialErrorV1::InvalidContext
        | ManagedProviderCredentialErrorV1::Rejected => TelegramCredentialRouteError::Rejected,
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

#[cfg(test)]
mod credential_binding_tests {
    use hermes_telegram_api::{TelegramCredentialBinding, TelegramCredentialPurpose};

    use super::credential_revisions;

    #[test]
    fn selects_one_exact_revision_for_each_admitted_user_purpose() {
        let revisions = credential_revisions(&[
            TelegramCredentialBinding {
                purpose: TelegramCredentialPurpose::ApiHash,
                revision: 7,
            },
            TelegramCredentialBinding {
                purpose: TelegramCredentialPurpose::SessionEncryptionKey,
                revision: 9,
            },
        ])
        .expect("select exact credential revisions");
        assert_eq!(revisions, (7, 9));
    }

    #[test]
    fn rejects_duplicate_or_bot_credential_bindings() {
        assert!(
            credential_revisions(&[
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::ApiHash,
                    revision: 1,
                },
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::ApiHash,
                    revision: 2,
                },
            ])
            .is_err()
        );
        assert!(
            credential_revisions(&[
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::ApiHash,
                    revision: 1,
                },
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::BotToken,
                    revision: 1,
                },
            ])
            .is_err()
        );
    }
}
