//! Translation of a validated module descriptor into Control Store request records.

use hermes_kernel_control_store::{
    ModuleBlobQuotaRequestV1, ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1,
    ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleSchedulerJobRequestV1,
    ModuleStorageRequestV1, ModuleVaultPurposeRequestV1,
};
use hermes_runtime_protocol::{
    v1::{
        DurableEnvelopeKindV1, EventRouteDirectionV1, EventSubscriptionRequirementV1,
        VaultActionV1, VaultSecretClassV1, VaultTargetScopeV1,
        capability_request_v1::Request as CapabilityRequest,
    },
    validation::descriptor::decode_descriptor_v1,
};
use sha2::{Digest, Sha256};

pub(super) struct DescriptorRegistrationRequests {
    module_id: String,
    owner_id: String,
    descriptor_sha256: [u8; 32],
    capability_ids: Vec<String>,
    storage: Vec<DescriptorStorageRequest>,
    events: Vec<DescriptorEventRouteRequest>,
    blobs: Vec<DescriptorBlobQuotaRequest>,
    scheduler: Vec<DescriptorSchedulerJobRequest>,
    vault_purposes: Vec<DescriptorVaultPurposeRequest>,
}

pub(super) struct BoundRegistrationRequests {
    pub(super) storage: Vec<ModuleStorageRequestV1>,
    pub(super) events: Vec<ModuleEventRouteRequestV1>,
    pub(super) blobs: Vec<ModuleBlobQuotaRequestV1>,
    pub(super) scheduler: Vec<ModuleSchedulerJobRequestV1>,
    pub(super) vault_purposes: Vec<ModuleVaultPurposeRequestV1>,
}

impl DescriptorRegistrationRequests {
    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
        let descriptor = decode_descriptor_v1(bytes)
            .map_err(|_| "module descriptor is invalid or exceeds protocol limits".to_owned())?;
        Ok(Self {
            module_id: descriptor.module_id.clone(),
            owner_id: descriptor.owner_id.clone(),
            descriptor_sha256: Sha256::digest(bytes).into(),
            capability_ids: descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.clone())
                .collect(),
            storage: storage_requests(&descriptor)?,
            events: event_route_requests(&descriptor)?,
            blobs: blob_quota_requests(&descriptor)?,
            scheduler: scheduler_job_requests(&descriptor)?,
            vault_purposes: vault_purpose_requests(&descriptor)?,
        })
    }

    pub(super) fn module_id(&self) -> String {
        self.module_id.clone()
    }

    pub(super) fn owner_id(&self) -> String {
        self.owner_id.clone()
    }

    pub(super) const fn descriptor_sha256(&self) -> [u8; 32] {
        self.descriptor_sha256
    }

    pub(super) fn capability_ids(&self) -> &[String] {
        &self.capability_ids
    }

    pub(super) fn bind(&self, registration: &ModuleRegistration) -> BoundRegistrationRequests {
        BoundRegistrationRequests {
            storage: bind_storage_requests(&self.storage, registration),
            events: bind_event_route_requests(&self.events, registration),
            blobs: bind_blob_quota_requests(&self.blobs, registration),
            scheduler: bind_scheduler_job_requests(&self.scheduler, registration),
            vault_purposes: bind_vault_purpose_requests(&self.vault_purposes, registration),
        }
    }
}

fn bind_storage_requests(
    requests: &[DescriptorStorageRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleStorageRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleStorageRequestV1::new(
                registration.registration_id(),
                &request.capability_id,
                &request.owner_id,
                request.connection_budget,
                request.statement_timeout_millis,
            )
        })
        .collect()
}

fn bind_event_route_requests(
    requests: &[DescriptorEventRouteRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleEventRouteRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
                registration_id: registration.registration_id().to_owned(),
                capability_id: request.capability_id.clone(),
                envelope_kind: request.envelope_kind,
                contract_owner: request.contract_owner.clone(),
                contract_name: request.contract_name.clone(),
                contract_major: request.contract_major,
                contract_revision: request.contract_revision,
                contract_schema_sha256: request.contract_schema_sha256,
                direction: request.direction,
                max_in_flight: request.max_in_flight,
                delivery_policy: request.delivery_policy,
            })
        })
        .collect()
}

fn bind_blob_quota_requests(
    requests: &[DescriptorBlobQuotaRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleBlobQuotaRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleBlobQuotaRequestV1::new(
                registration.registration_id(),
                &request.capability_id,
                registration.owner_id(),
                request.max_bytes,
            )
        })
        .collect()
}

fn bind_scheduler_job_requests(
    requests: &[DescriptorSchedulerJobRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleSchedulerJobRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleSchedulerJobRequestV1::new(
                registration.registration_id(),
                &request.capability_id,
                &request.owner,
                &request.name,
                request.major,
                request.revision,
                request.schema_sha256,
            )
        })
        .collect()
}

fn bind_vault_purpose_requests(
    requests: &[DescriptorVaultPurposeRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleVaultPurposeRequestV1> {
    requests.iter().map(|request| ModuleVaultPurposeRequestV1::new(
        registration.registration_id(), &request.capability_id, &request.purpose_id,
        request.requested_lease_ttl_seconds, request.secret_class, request.action,
        request.target_scope,
    )).collect()
}

struct DescriptorStorageRequest {
    capability_id: String,
    owner_id: String,
    connection_budget: u16,
    statement_timeout_millis: u32,
}

struct DescriptorEventRouteRequest {
    capability_id: String,
    envelope_kind: ModuleEventEnvelopeKindV1,
    contract_owner: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    direction: ModuleEventRouteDirectionV1,
    max_in_flight: u16,
    delivery_policy: Option<ModuleEventDeliveryPolicyV1>,
}

struct DescriptorBlobQuotaRequest {
    capability_id: String,
    max_bytes: u64,
}

struct DescriptorSchedulerJobRequest {
    capability_id: String,
    owner: String,
    name: String,
    major: u32,
    revision: u32,
    schema_sha256: [u8; 32],
}

struct DescriptorVaultPurposeRequest {
    capability_id: String,
    purpose_id: String,
    requested_lease_ttl_seconds: u16,
    secret_class: u8,
    action: u8,
    target_scope: u8,
}

fn event_route_requests(
    descriptor: &hermes_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorEventRouteRequest>, String> {
    descriptor
        .capabilities
        .iter()
        .flat_map(|capability| {
            capability
                .requests
                .iter()
                .map(move |request| (capability, request))
        })
        .filter_map(|(capability, request)| match request.request.as_ref() {
            Some(CapabilityRequest::EventRoute(route)) => Some((capability, route)),
            _ => None,
        })
        .map(|(capability, route)| descriptor_event_route(capability, route))
        .collect()
}

fn descriptor_event_route(
    capability: &hermes_runtime_protocol::v1::CapabilityDescriptorV1,
    route: &hermes_runtime_protocol::v1::EventRouteRequestV1,
) -> Result<DescriptorEventRouteRequest, String> {
    let contract = route
        .contract
        .as_ref()
        .ok_or_else(|| "module Event route request is invalid".to_owned())?;
    let contract_schema_sha256 = contract
        .schema_sha256
        .as_slice()
        .try_into()
        .map_err(|_| "module Event route request is invalid".to_owned())?;
    Ok(DescriptorEventRouteRequest {
        capability_id: capability.capability_id.clone(),
        envelope_kind: event_envelope_kind(route.envelope_kind)?,
        contract_owner: contract.owner.clone(),
        contract_name: contract.name.clone(),
        contract_major: contract.major,
        contract_revision: contract.revision,
        contract_schema_sha256,
        direction: event_route_direction(route.direction)?,
        max_in_flight: u16::try_from(route.max_in_flight)
            .map_err(|_| "module Event route request is invalid".to_owned())?,
        delivery_policy: event_delivery_policy(route)?,
    })
}

fn event_envelope_kind(value: i32) -> Result<ModuleEventEnvelopeKindV1, String> {
    match DurableEnvelopeKindV1::try_from(value).ok() {
        Some(DurableEnvelopeKindV1::Command) => Ok(ModuleEventEnvelopeKindV1::Command),
        Some(DurableEnvelopeKindV1::Event) => Ok(ModuleEventEnvelopeKindV1::Event),
        Some(DurableEnvelopeKindV1::Observation) => Ok(ModuleEventEnvelopeKindV1::Observation),
        Some(DurableEnvelopeKindV1::Result) => Ok(ModuleEventEnvelopeKindV1::Result),
        Some(DurableEnvelopeKindV1::Ack) => Ok(ModuleEventEnvelopeKindV1::Ack),
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn event_route_direction(value: i32) -> Result<ModuleEventRouteDirectionV1, String> {
    match EventRouteDirectionV1::try_from(value).ok() {
        Some(EventRouteDirectionV1::Publish) => Ok(ModuleEventRouteDirectionV1::Publish),
        Some(EventRouteDirectionV1::Consume) => Ok(ModuleEventRouteDirectionV1::Consume),
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn event_delivery_policy(
    route: &hermes_runtime_protocol::v1::EventRouteRequestV1,
) -> Result<Option<ModuleEventDeliveryPolicyV1>, String> {
    match EventRouteDirectionV1::try_from(route.direction).ok() {
        Some(EventRouteDirectionV1::Publish) => Ok(None),
        Some(EventRouteDirectionV1::Consume) => Ok(Some(ModuleEventDeliveryPolicyV1::new(
            event_subscription_requirement(route.subscription_requirement)?,
            u8::try_from(route.max_deliver)
                .map_err(|_| "module Event route request is invalid".to_owned())?,
            route.ack_wait_millis,
        ))),
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn event_subscription_requirement(
    value: i32,
) -> Result<ModuleEventSubscriptionRequirementV1, String> {
    match EventSubscriptionRequirementV1::try_from(value).ok() {
        Some(EventSubscriptionRequirementV1::Required) => {
            Ok(ModuleEventSubscriptionRequirementV1::Required)
        }
        Some(EventSubscriptionRequirementV1::Optional) => {
            Ok(ModuleEventSubscriptionRequirementV1::Optional)
        }
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn blob_quota_requests(
    descriptor: &hermes_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorBlobQuotaRequest>, String> {
    descriptor
        .capabilities
        .iter()
        .map(blob_quota_request_for_capability)
        .collect::<Result<Vec<_>, _>>()
        .map(|requests| requests.into_iter().flatten().collect())
}

fn blob_quota_request_for_capability(
    capability: &hermes_runtime_protocol::v1::CapabilityDescriptorV1,
) -> Result<Option<DescriptorBlobQuotaRequest>, String> {
    let requests = capability
        .requests
        .iter()
        .filter_map(|request| match request.request.as_ref() {
            Some(CapabilityRequest::BlobQuota(blob)) => Some(blob),
            _ => None,
        })
        .collect::<Vec<_>>();
    match requests.as_slice() {
        [] => Ok(None),
        [request] => Ok(Some(DescriptorBlobQuotaRequest {
            capability_id: capability.capability_id.clone(),
            max_bytes: request.max_bytes,
        })),
        _ => Err("module Blob quota request is invalid".to_owned()),
    }
}

fn storage_requests(
    descriptor: &hermes_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorStorageRequest>, String> {
    let mut requests = Vec::new();
    for capability in &descriptor.capabilities {
        let requested = capability
            .requests
            .iter()
            .filter_map(|request| match request.request.as_ref() {
                Some(CapabilityRequest::StorageNamespace(storage)) => Some(storage),
                _ => None,
            })
            .collect::<Vec<_>>();
        if requested.len() > 1 {
            return Err("module Storage request is invalid".to_owned());
        }
        if let Some(request) = requested.first() {
            if request.owner_id != descriptor.owner_id {
                return Err("module Storage request owner is invalid".to_owned());
            }
            requests.push(DescriptorStorageRequest {
                capability_id: capability.capability_id.clone(),
                owner_id: request.owner_id.clone(),
                connection_budget: u16::try_from(request.connection_budget)
                    .map_err(|_| "module Storage request is invalid".to_owned())?,
                statement_timeout_millis: request.timeout_millis,
            });
        }
    }
    Ok(requests)
}

fn scheduler_job_requests(
    descriptor: &hermes_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorSchedulerJobRequest>, String> {
    let mut requests = Vec::new();
    for capability in &descriptor.capabilities {
        for request in
            capability
                .requests
                .iter()
                .filter_map(|request| match request.request.as_ref() {
                    Some(CapabilityRequest::SchedulerJob(scheduler)) => Some(scheduler),
                    _ => None,
                })
        {
            let job_kind = request
                .job_kind
                .as_ref()
                .ok_or_else(|| "module Scheduler job request is invalid".to_owned())?;
            let schema_sha256 = job_kind
                .schema_sha256
                .as_slice()
                .try_into()
                .map_err(|_| "module Scheduler job request is invalid".to_owned())?;
            if job_kind.owner != descriptor.owner_id || job_kind.major > u32::from(u16::MAX) {
                return Err("module Scheduler job request owner is invalid".to_owned());
            }
            requests.push(DescriptorSchedulerJobRequest {
                capability_id: capability.capability_id.clone(),
                owner: job_kind.owner.clone(),
                name: job_kind.name.clone(),
                major: job_kind.major,
                revision: job_kind.revision,
                schema_sha256,
            });
        }
    }
    Ok(requests)
}

fn vault_purpose_requests(
    descriptor: &hermes_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorVaultPurposeRequest>, String> {
    let mut result = Vec::new();
    for capability in &descriptor.capabilities {
        for purpose in capability.requests.iter().filter_map(|request| match request.request.as_ref() {
            Some(CapabilityRequest::VaultPurpose(purpose)) => Some(purpose),
            _ => None,
        }) {
            if VaultTargetScopeV1::try_from(purpose.target_scope).ok()
                != Some(VaultTargetScopeV1::ConfigurationInstance)
            {
                return Err("module Vault purpose target scope is invalid".to_owned());
            }
            let ttl = u16::try_from(purpose.requested_lease_ttl_seconds)
                .map_err(|_| "module Vault purpose request is invalid".to_owned())?;
            for secret_class in &purpose.allowed_secret_classes {
                let secret_class = VaultSecretClassV1::try_from(*secret_class).ok()
                    .filter(|value| *value != VaultSecretClassV1::Unspecified)
                    .ok_or_else(|| "module Vault purpose request is invalid".to_owned())? as u8;
                for action in &purpose.actions {
                    let action = VaultActionV1::try_from(*action).ok()
                        .filter(|value| *value != VaultActionV1::Unspecified)
                        .ok_or_else(|| "module Vault purpose request is invalid".to_owned())? as u8;
                    result.push(DescriptorVaultPurposeRequest {
                        capability_id: capability.capability_id.clone(),
                        purpose_id: purpose.purpose_id.clone(),
                        requested_lease_ttl_seconds: ttl,
                        secret_class,
                        action,
                        target_scope: VaultTargetScopeV1::ConfigurationInstance as u8,
                    });
                }
            }
        }
    }
    Ok(result)
}
