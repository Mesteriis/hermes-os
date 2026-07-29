//! Exact technical admission for the independently managed workflow runtime.

use hermes_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
};
use hermes_communications_api::COMMUNICATIONS_QUERY_SCHEMA_SHA256;
use hermes_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::provider_event_admission::{
    delivery_intent_mail_events_capability_v1, delivery_intent_telegram_events_capability_v1,
    delivery_intent_whatsapp_events_capability_v1, delivery_intent_zulip_events_capability_v1,
};

pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.storage.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.blob.v1";
pub const COMMUNICATION_DELIVERY_INTENT_COMMUNICATIONS_QUERY_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.communications_query.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_CUSTODY_SCOPE_ID_V1: &str =
    "communication_delivery_intent.body.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_QUOTA_BYTES_V1: u64 = 16 * 1024 * 1024;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;

#[must_use]
pub fn communication_delivery_intent_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
                connection_budget: COMMUNICATION_DELIVERY_INTENT_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: COMMUNICATION_DELIVERY_INTENT_STORAGE_TIMEOUT_MILLIS_V1,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communication_delivery_intent_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATION_DELIVERY_INTENT_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CUSTODY_SCOPE_ID_V1.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communication_delivery_intent_communications_query_capability_v1() -> CapabilityDescriptorV1
{
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_COMMUNICATIONS_QUERY_CAPABILITY_ID_V1
            .to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![ContractReferenceV1 {
            owner: "communications".to_owned(),
            name: "communications.query".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communication_delivery_intent_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communication_delivery_intent_settings_schema_bytes_v1() -> Vec<u8> {
    communication_delivery_intent_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communication_delivery_intent_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = communication_delivery_intent_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 2,
        module_id: COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            communication_delivery_intent_blob_capability_v1(),
            communication_delivery_intent_communications_query_capability_v1(),
            delivery_intent_mail_events_capability_v1(),
            communication_delivery_intent_storage_capability_v1(),
            delivery_intent_telegram_events_capability_v1(),
            delivery_intent_whatsapp_events_capability_v1(),
            delivery_intent_zulip_events_capability_v1(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: COMMUNICATION_DELIVERY_INTENT_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Communication Delivery Intent".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_admits_canonical_query_blob_storage_and_four_exact_provider_event_units() {
        let descriptor = communication_delivery_intent_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&communication_delivery_intent_settings_schema_v1())
            .expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.capabilities.len(), 7);
        assert_eq!(
            descriptor.capabilities[0].capability_id,
            COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[1].capability_id,
            COMMUNICATION_DELIVERY_INTENT_COMMUNICATIONS_QUERY_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[3].capability_id,
            COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1
        );
        assert!(descriptor.capabilities[0].provides.is_empty());
        assert_eq!(
            descriptor.capabilities[1].dependencies,
            vec![ContractReferenceV1 {
                owner: "communications".to_owned(),
                name: "communications.query".to_owned(),
                major: 1,
                revision: 1,
                schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
            }]
        );
    }
}
