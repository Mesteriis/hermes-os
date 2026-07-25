//! Kernel-admitted Zulip runtime composition.
//!
//! This module owns process admission and integration resources only. It does
//! not reach Communications persistence or construct business state.

use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::time::Duration;

use hermes_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use hermes_communications_ingress::{
    BodyAdmissionFailureV1, BodyBlobReceiptV1, COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID, COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
};
use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access_v2,
};
use hermes_managed_vault_client::{
    ManagedProviderCredentialClientV2, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1, ManagedProviderCredentialRequestV1,
};
use hermes_runtime_protocol::v1::{
    BlobDataOperationV1, ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_runtime_protocol::validation::module_client::{
    validate_module_client_request_v1, validate_module_client_response_v1,
};
use hermes_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
        RejectManagedControlRequestsV2,
    },
    validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use hermes_vault_protocol::SecretClassV1;
use hermes_zulip_api::{
    ZulipAccountV1, ZulipCommandOperationStatusV1, ZulipCommandV1, ZulipEventQueueV1,
    command_blob_intent,
};
use hermes_zulip_core::credential_lease_purpose;
use hermes_zulip_http::ZulipHttpConfigV1;
use hermes_zulip_persistence::ZulipDurablePersistence;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::admission::{ZULIP_BLOB_CAPABILITY_ID, ZULIP_CREDENTIAL_LEASE_TTL_SECONDS};
use crate::{
    ZulipCommunicationsOutboxRelayError, ZulipRuntimeAdmissionV1, ZulipRuntimeErrorV1,
    ZulipRuntimeIdentityV1, acquire_event_queue, poll_once, relay_communications_outbox_once,
};
use zeroize::Zeroizing;

const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ZulipAdmittedRuntimeV1 {
    pub control_channel: ManagedControlChannelV2<UnixStream>,
    pub durable: ZulipDurablePersistence,
    http: ZulipHttpConfigV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    identity: ZulipRuntimeIdentityV1,
    blob_materializer: Mutex<Option<crate::blob::ZulipBlobMaterializer<BlobDataClient>>>,
    blob_write_materializer: Mutex<Option<crate::blob::ZulipBlobWriteMaterializer<BlobDataClient>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipRuntimeTickV1 {
    pub dispatched_command: bool,
    pub accepted_observations: usize,
    pub relayed_observations: usize,
}

#[derive(Debug)]
pub enum ZulipBootstrapErrorV1 {
    Admission,
    Control,
    Storage,
    Credential,
    PersistenceConnect,
    EventHub,
}

#[derive(Debug)]
pub enum ZulipRuntimeTickErrorV1 {
    Command(super::ZulipRuntimeErrorV1),
    Poll(super::ZulipRuntimeErrorV1),
    Relay(ZulipCommunicationsOutboxRelayError),
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    control_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &ZulipRuntimeAdmissionV1,
    account: ZulipAccountV1,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<ZulipAdmittedRuntimeV1, ZulipBootstrapErrorV1> {
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admission.logical_owner_id != "zulip"
        || admission.configuration_instance_id.trim().is_empty()
        || admission.runtime_instance_id.trim().is_empty()
        || account.account_id.trim().is_empty()
        || event_hub_endpoint.trim().is_empty()
        || event_credential_revision == 0
    {
        return Err(ZulipBootstrapErrorV1::Admission);
    }
    control_channel
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| control_channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| ZulipBootstrapErrorV1::Control)?;
    let mut control_channel = ManagedControlChannelV2::new(control_channel);
    let identity = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| ZulipBootstrapErrorV1::Control)?;
    let registration_id = identity.registration_id;
    let runtime_generation = identity.runtime_generation;
    let grant_epoch = identity.grant_epoch;
    if registration_id != admission.module_registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(ZulipBootstrapErrorV1::Admission);
    }

    let provider_context = provider_credential_context(admission, &storage_configuration)?;
    let api_key_revision =
        crate::api_key_revision(admission).map_err(|_| ZulipBootstrapErrorV1::Admission)?;
    let purpose = credential_lease_purpose(
        &account.account_id,
        &admission.configuration_instance_id,
        api_key_revision,
    )
    .map_err(|_| ZulipBootstrapErrorV1::Admission)?;
    let api_key = {
        let mut provider_credentials = ManagedProviderCredentialClientV2::new(&mut control_channel);
        let mut dispatcher = RejectManagedControlRequestsV2;
        provider_credentials
            .resolve(
                &mut dispatcher,
                &provider_context,
                ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: &admission.configuration_instance_id,
                    purpose_id: purpose.purpose_id(),
                    credential_revision: api_key_revision,
                    ttl_seconds: ZULIP_CREDENTIAL_LEASE_TTL_SECONDS,
                    secret_class: SecretClassV1::ProviderCredential,
                },
            )
            .map_err(map_provider_credential_error)?
    };
    let http = http_config_from_resolved_api_key(account, api_key)
        .map_err(|_| ZulipBootstrapErrorV1::Credential)?;

    let binding = storage_binding(&storage_configuration, admission)?;
    let storage_context = StorageVaultRouteContextV1::new(
        storage_configuration.vault_instance_id.clone(),
        storage_configuration.vault_runtime_generation,
        storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| ZulipBootstrapErrorV1::Storage)?,
    )
    .map_err(|_| ZulipBootstrapErrorV1::Storage)?;
    let mut storage_leases = StorageVaultLeaseAdapterV1::new(
        InheritedKernelVaultRouteV2::new(control_channel),
        storage_context,
    );
    let lease_id = storage_leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|_| ZulipBootstrapErrorV1::Credential)?;
    let password = storage_leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|_| ZulipBootstrapErrorV1::Credential)?;
    let mut control_channel = storage_leases.into_route_port().into_channel();
    let password = std::str::from_utf8(&password).map_err(|_| ZulipBootstrapErrorV1::Credential)?;
    let durable = ZulipDurablePersistence::connect_runtime(
        &binding,
        &storage_configuration.database_id,
        &storage_configuration.pgbouncer_host,
        storage_configuration.pgbouncer_port,
        password,
    )
    .await
    .map_err(|_| ZulipBootstrapErrorV1::PersistenceConnect)?;

    let event_access = request_managed_runtime_event_access_v2(
        &mut control_channel,
        &admission.logical_owner_id,
        &admission.module_registration_id,
        &admission.runtime_instance_id,
        admission.runtime_generation,
        admission.grant_epoch,
        event_credential_revision,
    )
    .map_err(|_| ZulipBootstrapErrorV1::EventHub)?;
    let nats_identity = RuntimeNatsIdentity::new(
        admission.runtime_instance_id.clone(),
        admission.runtime_generation,
        admission.grant_epoch,
    )
    .map_err(|_| ZulipBootstrapErrorV1::EventHub)?;
    let event_publish_permit = event_access
        .publish_permit(
            &admission.module_registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ZulipBootstrapErrorV1::EventHub)?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        nats_identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|_| ZulipBootstrapErrorV1::EventHub)?;
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id,
            runtime_generation,
            grant_epoch,
        })
        .map_err(|_| ZulipBootstrapErrorV1::Control)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .and_then(|_| control_channel.inner_mut().set_nonblocking(true))
        .map_err(|_| ZulipBootstrapErrorV1::Control)?;
    Ok(ZulipAdmittedRuntimeV1 {
        control_channel,
        durable,
        http,
        event_connection,
        event_publish_permit,
        identity: ZulipRuntimeIdentityV1 {
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
        },
        blob_materializer: Mutex::new(None),
        blob_write_materializer: Mutex::new(None),
    })
}

fn provider_credential_context(
    admission: &ZulipRuntimeAdmissionV1,
    configuration: &ManagedStorageRuntimeConfigurationV1,
) -> Result<ManagedProviderCredentialContextV1, ZulipBootstrapErrorV1> {
    let vault_public_key_x25519 = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| ZulipBootstrapErrorV1::Admission)?;
    if configuration.vault_runtime_generation != admission.vault_runtime_generation {
        return Err(ZulipBootstrapErrorV1::Admission);
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

fn map_provider_credential_error(error: ManagedProviderCredentialErrorV1) -> ZulipBootstrapErrorV1 {
    match error {
        ManagedProviderCredentialErrorV1::InvalidContext => ZulipBootstrapErrorV1::Admission,
        ManagedProviderCredentialErrorV1::Rejected
        | ManagedProviderCredentialErrorV1::Unavailable => ZulipBootstrapErrorV1::Credential,
    }
}

fn http_config_from_resolved_api_key(
    account: ZulipAccountV1,
    api_key: Zeroizing<Vec<u8>>,
) -> Result<ZulipHttpConfigV1, ZulipRuntimeErrorV1> {
    let api_key =
        String::from_utf8(api_key.to_vec()).map_err(|_| ZulipRuntimeErrorV1::Credential)?;
    ZulipHttpConfigV1::new(account, api_key).map_err(|_| ZulipRuntimeErrorV1::Credential)
}

impl ZulipAdmittedRuntimeV1 {
    pub async fn try_handle_client_delivery(
        &mut self,
        requested_at_unix_seconds: i64,
    ) -> Result<bool, ZulipBootstrapErrorV1> {
        let Some((correlation_id, control_request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| ZulipBootstrapErrorV1::Control)?
        else {
            return Ok(false);
        };
        let request = match control_request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => request,
                _ => {
                    write_control_error(
                        &mut self.control_channel,
                        correlation_id,
                        "managed_runtime_control_invalid_client_delivery",
                    )?;
                    return Ok(true);
                }
            },
            _ => {
                write_control_error(
                    &mut self.control_channel,
                    correlation_id,
                    "managed_runtime_control_unexpected_request",
                )?;
                return Ok(true);
            }
        };
        let payload = crate::client_port::handle_client_request(
            self,
            &request.encode_to_vec(),
            requested_at_unix_seconds,
        )
        .await
        .map_err(|_| ZulipBootstrapErrorV1::Admission)?;
        let response = ModuleClientResponseV1::decode(payload.as_slice())
            .map_err(|_| ZulipBootstrapErrorV1::Admission)?;
        validate_module_client_response_v1(&response)
            .map_err(|_| ZulipBootstrapErrorV1::Admission)?;
        write_client_delivery_response(&mut self.control_channel, correlation_id, response)?;
        Ok(true)
    }

    pub async fn acquire_event_queue(
        &self,
    ) -> Result<ZulipEventQueueV1, super::ZulipRuntimeErrorV1> {
        acquire_event_queue(&self.durable, &self.http).await
    }

    pub async fn poll_once(
        &mut self,
        queue: &mut ZulipEventQueueV1,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<usize, super::ZulipRuntimeErrorV1> {
        let durable = &self.durable;
        let identity = &self.identity;
        let http = &self.http;
        let control_channel = &mut self.control_channel;
        poll_once(
            durable,
            identity,
            http,
            queue,
            recorded_at_unix_seconds,
            recorded_at_nanos,
            &mut |plaintext| admit_inbound_plaintext(control_channel, plaintext),
        )
        .await
    }

    pub async fn submit_command(
        &self,
        command: &ZulipCommandV1,
        requested_at_unix_seconds: i64,
    ) -> Result<hermes_zulip_api::ZulipCommandReceiptV1, super::ZulipRuntimeErrorV1> {
        super::submit_command(&self.durable, command, requested_at_unix_seconds).await
    }

    pub async fn execute_next_command(
        &mut self,
        dispatched_at_unix_seconds: i64,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, super::ZulipRuntimeErrorV1> {
        super::execute_next_command_with_blob(
            &self.durable,
            &self.http,
            Some(&self.blob_materializer),
            Some(&self.blob_write_materializer),
            |command, operation| {
                authorize_blob_session(
                    &mut self.control_channel,
                    &self.blob_materializer,
                    &self.blob_write_materializer,
                    command,
                    operation,
                )
            },
            dispatched_at_unix_seconds,
            completed_at_unix_seconds,
        )
        .await
    }

    pub async fn command_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<ZulipCommandOperationStatusV1>, super::ZulipRuntimeErrorV1> {
        super::command_operation_status(&self.durable, operation_id).await
    }

    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, ZulipCommunicationsOutboxRelayError> {
        relay_communications_outbox_once(
            &self.durable,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }

    /// Runs one admitted integration lifecycle tick. Scheduling, shutdown and
    /// time acquisition remain owned by the caller; this method only orders
    /// the three provider-local runtime phases.
    pub async fn run_tick(
        &mut self,
        queue: &mut ZulipEventQueueV1,
        now_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<ZulipRuntimeTickV1, ZulipRuntimeTickErrorV1> {
        let dispatched_command = self
            .execute_next_command(now_unix_seconds, now_unix_seconds)
            .await
            .map_err(ZulipRuntimeTickErrorV1::Command)?;
        let accepted_observations = self
            .poll_once(queue, now_unix_seconds, recorded_at_nanos)
            .await
            .map_err(ZulipRuntimeTickErrorV1::Poll)?;
        let relayed_observations = match self.relay_communications_outbox(now_unix_seconds).await {
            Ok(relayed) => relayed,
            Err(ZulipCommunicationsOutboxRelayError::Unavailable) => 0,
            Err(error @ ZulipCommunicationsOutboxRelayError::Persistence(_)) => {
                return Err(ZulipRuntimeTickErrorV1::Relay(error));
            }
        };
        Ok(ZulipRuntimeTickV1 {
            dispatched_command,
            accepted_observations,
            relayed_observations,
        })
    }
}

fn admit_inbound_plaintext(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    plaintext: &[u8],
) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
    if plaintext.is_empty() || plaintext.len() > 256 * 1024 {
        return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
    }
    let mut reference_id = [0_u8; 16];
    getrandom::fill(&mut reference_id).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    if reference_id.iter().all(|byte| *byte == 0) {
        return Err(BodyAdmissionFailureV1::SourceUnavailable);
    }
    let sha256: [u8; 32] = Sha256::digest(plaintext).into();
    control_channel
        .inner_mut()
        .set_nonblocking(false)
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let mut dispatcher = ZulipBusyControlDispatcher;
    let delivery = request_managed_blob_session_v2(
        control_channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: ZULIP_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: u64::try_from(plaintext.len())
                .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
                module_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID,
                capability_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
            }),
        },
    );
    let restored = control_channel.inner_mut().set_nonblocking(true);
    let delivery = delivery.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
    restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let custody_transfer_source_proof = delivery.custody_transfer_source_proof;
    BlobDataClient::new(delivery.data_socket_path)
        .and_then(|client| {
            client.write(delivery.grant, delivery.channel_binding, plaintext.to_vec())
        })
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    Ok(BodyBlobReceiptV1 {
        blob_ref: format!(
            "blob-content:{}",
            reference_id
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ),
        reference_id,
        declared_bytes: u64::try_from(plaintext.len())
            .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
        sha256,
        custody_transfer_source_proof,
    })
}

fn authorize_blob_session(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    reader: &Mutex<Option<crate::blob::ZulipBlobMaterializer<BlobDataClient>>>,
    writer: &Mutex<Option<crate::blob::ZulipBlobWriteMaterializer<BlobDataClient>>>,
    command: &ZulipCommandV1,
    operation: BlobDataOperationV1,
) -> Result<(), super::ZulipRuntimeErrorV1> {
    let blob = command_blob_intent(command).ok_or(super::ZulipRuntimeErrorV1::Credential)?;
    control_channel
        .inner_mut()
        .set_nonblocking(false)
        .map_err(|_| super::ZulipRuntimeErrorV1::Credential)?;
    let mut dispatcher = ZulipBusyControlDispatcher;
    let delivery = request_managed_blob_session_v2(
        control_channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: ZULIP_BLOB_CAPABILITY_ID,
            operation,
            reference_id: &blob.reference_id,
            declared_size: blob.declared_size,
            backup_class: blob.backup_class,
            receipt_sha256: None,
            custody_target: None,
        },
    );
    let restored = control_channel.inner_mut().set_nonblocking(true);
    let delivery = delivery.map_err(|_| super::ZulipRuntimeErrorV1::Credential)?;
    restored.map_err(|_| super::ZulipRuntimeErrorV1::Credential)?;
    let session = crate::blob::ZulipBlobSessionV1 {
        blob_ref: blob.blob_ref.clone(),
        grant: delivery.grant,
        channel_binding: delivery.channel_binding,
        declared_size: blob.declared_size,
    };
    match operation {
        BlobDataOperationV1::BlobDataOperationReadRangeV1 => {
            let mut current = reader
                .lock()
                .map_err(|_| super::ZulipRuntimeErrorV1::Credential)?;
            if current.is_none() {
                *current = Some(crate::blob::ZulipBlobMaterializer::new(
                    BlobDataClient::new(delivery.data_socket_path)
                        .map_err(|_| super::ZulipRuntimeErrorV1::Credential)?,
                ));
            }
            current
                .as_mut()
                .ok_or(super::ZulipRuntimeErrorV1::Credential)?
                .register(session)
        }
        BlobDataOperationV1::BlobDataOperationWriteV1 => {
            let mut current = writer
                .lock()
                .map_err(|_| super::ZulipRuntimeErrorV1::Credential)?;
            if current.is_none() {
                *current = Some(crate::blob::ZulipBlobWriteMaterializer::new(
                    BlobDataClient::new(delivery.data_socket_path)
                        .map_err(|_| super::ZulipRuntimeErrorV1::Credential)?,
                ));
            }
            current
                .as_mut()
                .ok_or(super::ZulipRuntimeErrorV1::Credential)?
                .register(session)
        }
        BlobDataOperationV1::BlobDataOperationCustodyTransferV1
        | BlobDataOperationV1::BlobDataOperationUnspecifiedV1 => {
            Err(super::ZulipRuntimeErrorV1::Credential)
        }
    }
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ZulipRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ZulipBootstrapErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.owner != admission.logical_owner_id
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ZulipBootstrapErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.module_registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ZulipBootstrapErrorV1::Storage)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ZulipBootstrapErrorV1::Storage)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections).map_err(|_| ZulipBootstrapErrorV1::Storage)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ZulipBootstrapErrorV1::Storage)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ZulipBootstrapErrorV1::Storage)?,
    )
    .map_err(|_| ZulipBootstrapErrorV1::Storage)?;
    StorageBindingV1::new(identity, fences, access).map_err(|_| ZulipBootstrapErrorV1::Storage)
}

struct ZulipBusyControlDispatcher;

impl ManagedControlRequestDispatcherV2<UnixStream> for ZulipBusyControlDispatcher {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "RUNTIME_BUSY".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

fn write_client_delivery_response(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    response: ModuleClientResponseV1,
) -> Result<(), ZulipBootstrapErrorV1> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ClientDelivery(
                    ManagedRuntimeClientDeliveryResponseV1 {
                        response: Some(response),
                    },
                )),
                error_code: String::new(),
            },
        )
        .map_err(|_| ZulipBootstrapErrorV1::Control)
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    error_code: &str,
) -> Result<(), ZulipBootstrapErrorV1> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| ZulipBootstrapErrorV1::Control)
}

#[cfg(test)]
mod control_dispatch_tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
    use hermes_runtime_protocol::v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlAckV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ModuleClientRequestV1,
        managed_runtime_control_frame_v2::Frame, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    };
    use hermes_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;

    use super::ZulipBusyControlDispatcher;

    #[test]
    fn nested_client_delivery_gets_a_correlated_busy_response_without_stealing_platform_reply() {
        let (runtime, kernel) = UnixStream::pair().expect("control pair");
        let kernel = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(kernel);
            let (platform_id, _) = channel.receive_request().expect("platform request");
            channel
                .write_request(
                    [7; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::ClientDelivery(
                            ManagedRuntimeClientDeliveryRequestV1 {
                                request: Some(ModuleClientRequestV1 {
                                    protocol_major: 1,
                                    module_id: "hermes-zulip-runtime".to_owned(),
                                    owner_id: "zulip".to_owned(),
                                    contract: Some(ContractReferenceV1 {
                                        owner: "zulip".to_owned(),
                                        name: "query".to_owned(),
                                        major: 1,
                                        revision: 1,
                                        schema_sha256: vec![1; 32],
                                    }),
                                    request_id: 41,
                                    request_payload: vec![1],
                                }),
                            },
                        )),
                    },
                )
                .expect("client delivery");
            let nested = channel.read_frame().expect("busy response");
            assert_eq!(
                nested.correlation_id,
                vec![7; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            let Some(Frame::Response(response)) = nested.frame else {
                panic!("nested response");
            };
            let Some(ControlResult::ClientDelivery(delivery)) = response.result else {
                panic!("client delivery response");
            };
            assert_eq!(
                delivery.response.expect("module response").error_code,
                "RUNTIME_BUSY"
            );
            channel
                .write_response(
                    platform_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("platform response");
        });

        let mut channel = ManagedControlChannelV2::new(runtime);
        let mut dispatcher = ZulipBusyControlDispatcher;
        let response = channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1::default())),
                },
                &mut dispatcher,
            )
            .expect("correlated platform response");
        assert!(matches!(response.result, Some(ControlResult::Ack(_))));
        kernel.join().expect("kernel join");
    }
}
