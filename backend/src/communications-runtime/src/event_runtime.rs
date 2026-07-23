//! Kernel-fenced Event Hub consumer for the Communications domain.

use std::{
    io,
    os::{fd::AsRawFd, unix::net::UnixStream},
    time::{SystemTime, UNIX_EPOCH},
};

use hermes_communications_persistence::CommunicationsDurablePersistence;
use hermes_communications_domain::COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1;
use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1, RuntimeSubscribePermitV1,
    request_managed_runtime_event_access,
};
use hermes_runtime_protocol::v1::{
    ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeClientDeliveryResponseV1,
    ModuleClientResponseV1, ManagedStorageRuntimeConfigurationV1,
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_runtime_protocol::validation::module_client::{validate_module_client_request_v1, validate_module_client_response_v1};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use prost::Message;

use crate::{
    canonical_outbox::CanonicalEventContextV1,
    consumer::{CommunicationsDeliveryErrorV1, consume_next_observation_v1},
    custody_worker::process_next_body_custody_transfer_v1,
    domain_outbox::{CommunicationsDomainOutboxRelayErrorV1, relay_domain_outbox_once},
    search_access::CommunicationsSearchAccessV1,
    search_worker::process_next_derived_index_job_v1,
};

const MAX_FRAME_BYTES: usize = 512 * 1024;

pub struct CommunicationsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct CommunicationsEventRuntimeV1 {
    control_channel: UnixStream,
    connection: RuntimeJetStreamConnection,
    observation_permit: RuntimeSubscribePermitV1,
    domain_publish_permit: RuntimePublishPermitV1,
    persistence: CommunicationsDurablePersistence,
    search_access: CommunicationsSearchAccessV1,
    runtime_instance_id: String,
    runtime_generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEventRuntimeErrorV1 {
    Admission,
    Unavailable,
}

impl CommunicationsEventRuntimeV1 {
    pub async fn open(
        control_channel: &mut UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &CommunicationsRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, CommunicationsEventRuntimeErrorV1> {
        if descriptor_bytes.is_empty()
            || settings_schema_bytes.is_empty()
            || admission.logical_owner_id.trim().is_empty()
            || admission.registration_id.trim().is_empty()
            || admission.runtime_instance_id.trim().is_empty()
            || admission.runtime_generation == 0
            || admission.grant_epoch == 0
            || credential_revision == 0
            || event_hub_endpoint.trim().is_empty()
        {
            return Err(CommunicationsEventRuntimeErrorV1::Admission);
        }
        authenticate_managed_runtime(
            control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let access = request_managed_runtime_event_access(
            control_channel,
            &admission.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            credential_revision,
        )
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let mut permits = access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        if permits.len() != 1 {
            return Err(CommunicationsEventRuntimeErrorV1::Admission);
        }
        let observation_permit = permits.pop().ok_or(CommunicationsEventRuntimeErrorV1::Admission)?;
        let domain_publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV1::new(
                control_channel
                    .try_clone()
                    .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?,
            ),
            vault_context,
        );
        let lease_id = leases
            .issue_runtime_credential(&binding)
            .await
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let password = leases
            .resolve_runtime_credential(&binding, lease_id)
            .await
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        let persistence = CommunicationsDurablePersistence::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let started_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        persistence
            .reconcile_search_projection_jobs(
                COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1,
                i64::try_from(started_at_unix_seconds.as_secs())
                    .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?,
            )
            .await
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let search_access = CommunicationsSearchAccessV1::open(
            control_channel
                .try_clone()
                .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?,
            admission,
            &storage_configuration,
        )
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        control_channel
            .set_nonblocking(true)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel: control_channel.try_clone().map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?,
            connection,
            observation_permit,
            domain_publish_permit,
            persistence,
            search_access,
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
        })
    }

    pub async fn try_handle_client_delivery(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let Some(frame) = peek_complete_frame(&self.control_channel)? else { return Ok(false); };
        let request = ManagedRuntimeClientDeliveryRequestV1::decode(frame.as_slice())
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?
            .request
            .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?;
        if validate_module_client_request_v1(&request).is_err() {
            if read_frame(&mut self.control_channel)? != frame {
                return Err(CommunicationsEventRuntimeErrorV1::Admission);
            }
            write_frame(
                &mut self.control_channel,
                &ManagedRuntimeClientDeliveryResponseV1 {
                    response: Some(ModuleClientResponseV1 {
                        protocol_major: 1,
                        request_id: request.request_id,
                        response_payload: Vec::new(),
                        error_code: "REJECTED".to_owned(),
                    }),
                }
                .encode_to_vec(),
            )?;
            return Ok(true);
        }
        if read_frame(&mut self.control_channel)? != frame { return Err(CommunicationsEventRuntimeErrorV1::Admission); }
        let payload = crate::query_client_port::handle_module_query_request_v1(
            &self.persistence,
            &mut self.search_access,
            &request.encode_to_vec(),
        )
        .await;
        let response = match payload {
            Ok(payload) => ModuleClientResponseV1::decode(payload.as_slice())
                .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?,
            Err(_) => ModuleClientResponseV1 {
                protocol_major: 1,
                request_id: request.request_id,
                response_payload: Vec::new(),
                error_code: "UNAVAILABLE".to_owned(),
            },
        };
        validate_module_client_response_v1(&response).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        if response.request_id != request.request_id {
            return Err(CommunicationsEventRuntimeErrorV1::Admission);
        }
        write_frame(
            &mut self.control_channel,
            &ManagedRuntimeClientDeliveryResponseV1 { response: Some(response) }.encode_to_vec(),
        )?;
        Ok(true)
    }

    pub async fn consume_next(&mut self) -> Result<(), CommunicationsDeliveryErrorV1> {
        let canonical_event_context = self.canonical_event_context()?;
        consume_next_observation_v1(&self.persistence, &self.connection, &self.observation_permit, &canonical_event_context)
            .await
            .map(|_| ())?;
        Ok(())
    }

    pub async fn process_next_body_custody_transfer(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        process_next_body_custody_transfer_v1(
            &mut self.control_channel,
            &self.persistence,
            &format!("{}:{}", self.runtime_instance_id, self.runtime_generation),
            context.recorded_at_unix_seconds,
        )
        .await
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
    }

    pub async fn process_next_derived_index_job(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        process_next_derived_index_job_v1(
            &self.persistence,
            &mut self.search_access,
            &format!("{}:{}", self.runtime_instance_id, self.runtime_generation),
            context.recorded_at_unix_seconds,
        )
        .await
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
    }

    pub async fn reconcile_search_projection_jobs(
        &self,
    ) -> Result<usize, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        self.persistence
            .reconcile_search_projection_jobs(
                COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1,
                context.recorded_at_unix_seconds,
            )
            .await
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
    }

    fn canonical_event_context(&self) -> Result<CanonicalEventContextV1, CommunicationsDeliveryErrorV1> {
        let recorded_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
        Ok(CanonicalEventContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            recorded_at_unix_seconds: i64::try_from(recorded_at.as_secs())
                .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?,
            recorded_at_nanos: i32::try_from(recorded_at.subsec_nanos())
                .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?,
        })
    }

    pub async fn relay_domain_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, CommunicationsDomainOutboxRelayErrorV1> {
        relay_domain_outbox_once(
            &self.persistence,
            &self.connection,
            &self.domain_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }
}

fn authenticate_managed_runtime(
    control_channel: &mut UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &CommunicationsRuntimeAdmissionV1,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    const CONTROL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
    control_channel
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| control_channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    write_frame(
        control_channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                descriptor_bytes,
                settings_schema_bytes,
            })),
        }.encode_to_vec(),
    )?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(control_channel)?.as_slice())
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    let (registration_id, runtime_generation, grant_epoch) = match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && !value.registration_id.is_empty()
                && value.runtime_generation != 0
                && value.grant_epoch != 0 =>
        {
            (value.registration_id, value.runtime_generation, value.grant_epoch)
        }
        _ => return Err(CommunicationsEventRuntimeErrorV1::Admission),
    };
    if registration_id != admission.registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(CommunicationsEventRuntimeErrorV1::Admission);
    }
    write_frame(
        control_channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                registration_id,
                runtime_generation,
                grant_epoch,
            })),
        }.encode_to_vec(),
    )?;
    control_channel
        .set_read_timeout(None)
        .and_then(|_| control_channel.set_write_timeout(None))
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, CommunicationsEventRuntimeErrorV1> {
    let length = usize::try_from(read_varint(channel)?).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    if length > MAX_FRAME_BYTES { return Err(CommunicationsEventRuntimeErrorV1::Unavailable); }
    use std::io::Read;
    let mut bytes = vec![0_u8; length];
    channel.read_exact(&mut bytes).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    Ok(bytes)
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    use std::io::Write;
    let mut length = u32::try_from(bytes.len()).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    channel.write_all(&prefix[..=index]).and_then(|_| channel.write_all(bytes)).and_then(|_| channel.flush())
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn peek_complete_frame(channel: &UnixStream) -> Result<Option<Vec<u8>>, CommunicationsEventRuntimeErrorV1> {
    let mut header = [0_u8; 5];
    let length = unsafe {
        libc::recv(
            channel.as_raw_fd(),
            header.as_mut_ptr().cast(),
            header.len(),
            libc::MSG_PEEK,
        )
    };
    if length < 0 {
        return if io::Error::last_os_error().kind() == io::ErrorKind::WouldBlock {
            Ok(None)
        } else {
            Err(CommunicationsEventRuntimeErrorV1::Unavailable)
        };
    }
    if length == 0 { return Err(CommunicationsEventRuntimeErrorV1::Unavailable); }
    let (payload_length, prefix_length) = decode_peeked_length(
        &header[..usize::try_from(length).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?],
    )?;
    if payload_length == 0 || payload_length > MAX_FRAME_BYTES {
        return Err(CommunicationsEventRuntimeErrorV1::Unavailable);
    }
    let mut framed = vec![0_u8; prefix_length + payload_length];
    let received = unsafe {
        libc::recv(
            channel.as_raw_fd(),
            framed.as_mut_ptr().cast(),
            framed.len(),
            libc::MSG_PEEK,
        )
    };
    if received < 0 {
        return Err(CommunicationsEventRuntimeErrorV1::Unavailable);
    }
    if usize::try_from(received).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)? < framed.len() {
        return Ok(None);
    }
    Ok(Some(framed[prefix_length..].to_vec()))
}

fn decode_peeked_length(bytes: &[u8]) -> Result<(usize, usize), CommunicationsEventRuntimeErrorV1> {
    let mut value = 0_usize;
    for (index, byte) in bytes.iter().copied().enumerate() {
        value |= usize::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return Ok((value, index + 1));
        }
    }
    Err(CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn read_varint(channel: &mut UnixStream) -> Result<u64, CommunicationsEventRuntimeErrorV1> {
    use std::io::Read;
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel.read_exact(&mut byte).map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 { return Ok(value); }
    }
    Err(CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &CommunicationsRuntimeAdmissionV1,
) -> Result<StorageBindingV1, CommunicationsEventRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(CommunicationsEventRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)
}
