//! Kernel-fenced Event Hub consumer for the Communications domain.

use std::{
    os::unix::net::UnixStream,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use hermes_communications_attachment_contract::admission::{
    communication_attachment_blob_admission_observed_contract_reference_v1,
    communication_attachment_safety_verdict_observed_contract_reference_v1,
};
use hermes_communications_domain::COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1;
use hermes_communications_ingress::admission::communication_observed_contract_reference_v1;
use hermes_communications_persistence::CommunicationsDurablePersistence;
use hermes_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
use hermes_runtime_protocol::managed_control::{
    ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    RejectManagedControlRequestsV2,
};
use hermes_runtime_protocol::v1::ContractReferenceV1;
use hermes_runtime_protocol::v1::{
    ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
use hermes_runtime_protocol::validation::module_client::{
    validate_module_client_request_v1, validate_module_client_response_v1,
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    attachment_observation_consumer::{
        consume_next_attachment_blob_admission_observation_v1,
        consume_next_attachment_safety_verdict_observation_v1,
    },
    canonical_outbox::CanonicalEventContextV1,
    client_port::dispatch_module_client_request_v1,
    consumer::{CommunicationsDeliveryErrorV1, consume_next_observation_v1},
    content_ticket_store::CommunicationsContentTicketStoreV1,
    custody_worker::{CommunicationsCustodyWorkerErrorV1, process_next_body_custody_transfer_v1},
    domain_outbox::{CommunicationsDomainOutboxRelayErrorV1, relay_domain_outbox_once},
    search_access::CommunicationsSearchAccessV1,
    search_worker::process_next_derived_index_job_v1,
};

pub struct CommunicationsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct CommunicationsEventRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: CommunicationsSubscribePermitsV1,
    next_consumer: CommunicationsConsumerV1,
    domain_publish_permit: RuntimePublishPermitV1,
    persistence: CommunicationsDurablePersistence,
    search_access: CommunicationsSearchAccessV1,
    content_tickets: Arc<CommunicationsContentTicketStoreV1>,
    runtime_instance_id: String,
    runtime_generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEventRuntimeErrorV1 {
    Admission,
    Unavailable,
}

struct CommunicationsSubscribePermitsV1 {
    observation: RuntimeSubscribePermitV1,
    attachment_blob_admission: RuntimeSubscribePermitV1,
    attachment_safety_verdict: RuntimeSubscribePermitV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommunicationsConsumerV1 {
    Observation,
    AttachmentBlobAdmission,
    AttachmentSafetyVerdict,
}

impl CommunicationsConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Observation => Self::AttachmentBlobAdmission,
            Self::AttachmentBlobAdmission => Self::AttachmentSafetyVerdict,
            Self::AttachmentSafetyVerdict => Self::Observation,
        }
    }
}

impl CommunicationsSubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, CommunicationsEventRuntimeErrorV1> {
        let observation = communication_observed_contract_reference_v1();
        let attachment_blob_admission =
            communication_attachment_blob_admission_observed_contract_reference_v1();
        let attachment_safety_verdict =
            communication_attachment_safety_verdict_observed_contract_reference_v1();
        let mut observation_permit = None;
        let mut attachment_blob_admission_permit = None;
        let mut attachment_safety_verdict_permit = None;
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(CommunicationsEventRuntimeErrorV1::Admission);
            };
            if exact_contract(contract, &observation) {
                replace_once(&mut observation_permit, permit)?;
            } else if exact_contract(contract, &attachment_blob_admission) {
                replace_once(&mut attachment_blob_admission_permit, permit)?;
            } else if exact_contract(contract, &attachment_safety_verdict) {
                replace_once(&mut attachment_safety_verdict_permit, permit)?;
            } else {
                return Err(CommunicationsEventRuntimeErrorV1::Admission);
            }
        }
        Ok(Self {
            observation: observation_permit.ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            attachment_blob_admission: attachment_blob_admission_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            attachment_safety_verdict: attachment_safety_verdict_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
        })
    }
}

fn replace_once(
    slot: &mut Option<RuntimeSubscribePermitV1>,
    permit: RuntimeSubscribePermitV1,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    slot.replace(permit)
        .is_none()
        .then_some(())
        .ok_or(CommunicationsEventRuntimeErrorV1::Admission)
}

fn exact_contract(left: &ContractReferenceV1, right: &ContractReferenceV1) -> bool {
    left.owner == right.owner
        && left.name == right.name
        && left.major == right.major
        && left.revision == right.revision
        && left.schema_sha256 == right.schema_sha256
}

struct CommunicationsNestedRequestDispatcher<'a> {
    persistence: &'a CommunicationsDurablePersistence,
    search_access: &'a mut CommunicationsSearchAccessV1,
    content_tickets: &'a Arc<CommunicationsContentTicketStoreV1>,
}

impl ManagedControlRequestDispatcherV2<UnixStream> for CommunicationsNestedRequestDispatcher<'_> {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: hermes_runtime_protocol::v1::ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let mut reject_nested_request = RejectManagedControlRequestsV2;
                    let response = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(
                            dispatch_module_client_request_v1(
                                self.persistence,
                                self.content_tickets,
                                self.search_access,
                                channel,
                                &mut reject_nested_request,
                                &request,
                            ),
                        )
                    });
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(response),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                Some(request) => ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ClientDelivery(
                        ManagedRuntimeClientDeliveryResponseV1 {
                            response: Some(ModuleClientResponseV1 {
                                protocol_major: 1,
                                request_id: request.request_id,
                                response_payload: Vec::new(),
                                error_code: "REJECTED".to_owned(),
                            }),
                        },
                    )),
                    error_code: String::new(),
                },
                None => ManagedRuntimeControlResponseV1 {
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

impl CommunicationsEventRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
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
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_managed_runtime_v2(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &admission.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            credential_revision,
        )
        .map_err(|_| unavailable_at("event_access"))?;
        let permits = access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        let permits = CommunicationsSubscribePermitsV1::bind(permits)?;
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
        .map_err(|_| unavailable_at("event_connection"))?;
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
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_runtime_credential(&mut leases, &binding)
            .await
            .map_err(|_| unavailable_at("storage_credential"))?;
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
        .map_err(|_| unavailable_at("storage_connection"))?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| unavailable_at("storage_readiness"))?;
        let mut control_channel = leases.into_route_port().into_channel();
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
            .map_err(|_| unavailable_at("search_projection"))?;
        let search_access = CommunicationsSearchAccessV1::open(admission, &storage_configuration)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
        signal_managed_runtime_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            next_consumer: CommunicationsConsumerV1::Observation,
            domain_publish_permit,
            persistence,
            search_access,
            content_tickets: Arc::new(CommunicationsContentTicketStoreV1::new()),
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
        })
    }

    pub async fn try_handle_client_delivery(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| unavailable_at("client_receive"))?
        else {
            return Ok(false);
        };
        let request = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) => request,
                None => {
                    self.control_channel
                        .write_response(
                            correlation_id,
                            ManagedRuntimeControlResponseV1 {
                                result: None,
                                error_code: "managed_runtime_control_invalid_client_delivery"
                                    .to_owned(),
                            },
                        )
                        .map_err(|_| unavailable_at("client_invalid_write"))?;
                    return Ok(true);
                }
            },
            _ => {
                self.control_channel
                    .write_response(
                        correlation_id,
                        ManagedRuntimeControlResponseV1 {
                            result: None,
                            error_code: "managed_runtime_control_unexpected_request".to_owned(),
                        },
                    )
                    .map_err(|_| unavailable_at("client_unexpected_write"))?;
                return Ok(true);
            }
        };
        if validate_module_client_request_v1(&request).is_err() {
            self.control_channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "REJECTED".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    },
                )
                .map_err(|_| unavailable_at("client_rejected_write"))?;
            return Ok(true);
        }
        let mut nested_search_access = self.search_access.clone();
        let mut nested_dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            search_access: &mut nested_search_access,
            content_tickets: &self.content_tickets,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| unavailable_at("client_blocking"))?;
        let response = dispatch_module_client_request_v1(
            &self.persistence,
            &self.content_tickets,
            &mut self.search_access,
            &mut self.control_channel,
            &mut nested_dispatcher,
            &request,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| unavailable_at("client_nonblocking"))?;
        validate_module_client_response_v1(&response)
            .map_err(|_| unavailable_at("client_response_validate"))?;
        if response.request_id != request.request_id {
            return Err(admission_at("client_response_request_id"));
        }
        self.control_channel
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
            .map_err(|_| unavailable_at("client_write"))?;
        Ok(true)
    }

    pub async fn consume_next(&mut self) -> Result<(), CommunicationsDeliveryErrorV1> {
        let canonical_event_context = self.canonical_event_context()?;
        let consumer = self.next_consumer;
        self.next_consumer = consumer.successor();
        match consumer {
            CommunicationsConsumerV1::Observation => consume_next_observation_v1(
                &self.persistence,
                &self.connection,
                &self.permits.observation,
                &canonical_event_context,
            )
            .await
            .map(|_| ()),
            CommunicationsConsumerV1::AttachmentBlobAdmission => {
                consume_next_attachment_blob_admission_observation_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.attachment_blob_admission,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
            }
            CommunicationsConsumerV1::AttachmentSafetyVerdict => {
                consume_next_attachment_safety_verdict_observation_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.attachment_safety_verdict,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
            }
        }
    }

    pub async fn process_next_body_custody_transfer(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            search_access: &mut self.search_access,
            content_tickets: &self.content_tickets,
        };
        match process_next_body_custody_transfer_v1(
            &mut self.control_channel,
            &mut dispatcher,
            &self.persistence,
            &format!("{}:{}", self.runtime_instance_id, self.runtime_generation),
            context.recorded_at_unix_seconds,
        )
        .await
        {
            Ok(processed) => Ok(processed),
            Err(CommunicationsCustodyWorkerErrorV1::RetryPending) => Ok(false),
            Err(CommunicationsCustodyWorkerErrorV1::StorageUnavailable) => {
                Err(CommunicationsEventRuntimeErrorV1::Unavailable)
            }
        }
    }

    pub async fn process_next_derived_index_job(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let mut nested_search_access = self.search_access.clone();
        let mut nested_dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            search_access: &mut nested_search_access,
            content_tickets: &self.content_tickets,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| unavailable_at("search_worker_blocking"))?;
        let result = process_next_derived_index_job_v1(
            &self.persistence,
            &mut self.search_access,
            &mut self.control_channel,
            &mut nested_dispatcher,
            &format!("{}:{}", self.runtime_instance_id, self.runtime_generation),
            context.recorded_at_unix_seconds,
        )
        .await
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| unavailable_at("search_worker_nonblocking"))?;
        result
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

    fn canonical_event_context(
        &self,
    ) -> Result<CanonicalEventContextV1, CommunicationsDeliveryErrorV1> {
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

fn unavailable_at(stage: &str) -> CommunicationsEventRuntimeErrorV1 {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_runtime_startup_unavailable stage={stage}");
    }
    CommunicationsEventRuntimeErrorV1::Unavailable
}

fn admission_at(stage: &str) -> CommunicationsEventRuntimeErrorV1 {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_runtime_admission stage={stage}");
    }
    CommunicationsEventRuntimeErrorV1::Admission
}

async fn resolve_storage_runtime_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, CommunicationsEventRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(lease_id) = leases.issue_runtime_credential(binding).await
            && let Ok(password) = leases.resolve_runtime_credential(binding, lease_id).await
        {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Err(CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn authenticate_managed_runtime_v2(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &CommunicationsRuntimeAdmissionV1,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    let registration_id = response.registration_id;
    let runtime_generation = response.runtime_generation;
    let grant_epoch = response.grant_epoch;
    if registration_id != admission.registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(CommunicationsEventRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_managed_runtime_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &CommunicationsRuntimeAdmissionV1,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
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

#[cfg(test)]
mod tests {
    use super::CommunicationsConsumerV1;

    #[test]
    fn event_consumers_advance_without_empty_consumer_starvation() {
        let first = CommunicationsConsumerV1::Observation;
        let second = first.successor();
        let third = second.successor();

        assert_eq!(
            [first, second, third, third.successor(),],
            [
                CommunicationsConsumerV1::Observation,
                CommunicationsConsumerV1::AttachmentBlobAdmission,
                CommunicationsConsumerV1::AttachmentSafetyVerdict,
                CommunicationsConsumerV1::Observation,
            ]
        );
    }
}
