//! Exact descriptor and capability admission for the Communications owner runtime.

use hermes_communications_api::{
    COMMUNICATION_EVIDENCE_SCHEMA_SHA256, COMMUNICATIONS_QUERY_SCHEMA_SHA256,
};
use hermes_communications_ingress::admission::{
    COMMUNICATION_OBSERVED_MAX_IN_FLIGHT, communication_observed_contract_reference_v1,
};
use hermes_runtime_protocol::v1::{
    BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
    ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    SettingsSchemaV1, StorageNamespaceRequestV1, VaultActionV1, VaultPurposeRequestV1,
    VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_BLOB_CAPABILITY_ID: &str = "communications.blob.v1";
pub const COMMUNICATIONS_EVENTS_CAPABILITY_ID: &str = "communications.events.v1";
pub const COMMUNICATIONS_OBSERVE_CAPABILITY_ID: &str = "communications.observe.v1";
pub const COMMUNICATIONS_QUERY_CAPABILITY_ID: &str = "communications.query.v1";
pub const COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID: &str = "communications.search.index.v1";
pub const COMMUNICATIONS_STORAGE_CAPABILITY_ID: &str = "communications.storage.v1";
pub const COMMUNICATIONS_MODULE_ID: &str = "hermes-communications-runtime";
pub const COMMUNICATIONS_OWNER_ID: &str = "communications";
pub const COMMUNICATIONS_BLOB_QUOTA_BYTES: u64 = 1 << 30;
pub const COMMUNICATIONS_STORAGE_CONNECTION_BUDGET: u32 = 8;
pub const COMMUNICATIONS_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const COMMUNICATIONS_EVENT_MAX_DELIVER: u32 = 8;
pub const COMMUNICATIONS_EVENT_ACK_WAIT_MILLIS: u32 = 30_000;
pub const COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID: &str = "communications.search.index";
pub const COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION: u32 = 1;
pub const COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS: u32 = 60;

#[must_use]
pub fn communications_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        communications_blob_capability_v1(),
        communications_events_capability_v1(),
        communications_observe_capability_v1(),
        communications_query_capability_v1(),
        communications_search_index_capability_v1(),
        communications_storage_capability_v1(),
    ]
}

#[must_use]
pub fn communications_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_events_capability_v1() -> CapabilityDescriptorV1 {
    let recorded = communication_evidence_recorded_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EVENTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(recorded.clone()),
        }],
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: DurableEnvelopeKindV1::Event as i32,
                contract: Some(recorded),
                direction: EventRouteDirectionV1::Publish as i32,
                max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
                max_deliver: 0,
                ack_wait_millis: 0,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_observe_capability_v1() -> CapabilityDescriptorV1 {
    let observed = communication_observed_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_OBSERVE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(observed.clone()),
        }],
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: DurableEnvelopeKindV1::Observation as i32,
                contract: Some(observed),
                direction: EventRouteDirectionV1::Consume as i32,
                max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                max_deliver: COMMUNICATIONS_EVENT_MAX_DELIVER,
                ack_wait_millis: COMMUNICATIONS_EVENT_ACK_WAIT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_query_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_QUERY_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(communications_query_contract_reference_v1()),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_search_index_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID.to_owned(),
                requested_lease_ttl_seconds: COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS,
                allowed_secret_classes: vec![VaultSecretClassV1::OwnerDerivedKey as i32],
                actions: vec![VaultActionV1::IssueOwnerDerivedKey as i32],
                target_scope: VaultTargetScopeV1::OwnerDerivedProjectionKey as i32,
                key_schema_revision: COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: "communications".to_owned(),
                connection_budget: COMMUNICATIONS_STORAGE_CONNECTION_BUDGET,
                timeout_millis: COMMUNICATIONS_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_query_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: "communications.query".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communication_evidence_recorded_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: "communication_evidence_recorded".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: COMMUNICATION_EVIDENCE_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communications_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communications_settings_schema_bytes_v1() -> Vec<u8> {
    communications_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communications_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = communications_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        owner_id: COMMUNICATIONS_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 1,
            maximum_major: 1,
            minimum_revision: 1,
        }),
        capabilities: communications_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: COMMUNICATIONS_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Communications".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::{
        validation::descriptor::{validate_descriptor_v1, validate_settings_schema_v1},
    };

    use super::*;

    #[test]
    fn first_owner_descriptor_is_valid_and_exact() {
        let descriptor = communications_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                COMMUNICATIONS_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_EVENTS_CAPABILITY_ID,
                COMMUNICATIONS_OBSERVE_CAPABILITY_ID,
                COMMUNICATIONS_QUERY_CAPABILITY_ID,
                COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
                COMMUNICATIONS_STORAGE_CAPABILITY_ID,
            ]
        );
        assert_eq!(
            validate_settings_schema_v1(&communications_settings_schema_v1()),
            Ok(())
        );
    }
}
