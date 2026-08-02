use hermes_contacts_command_api::{
    CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1, CONTACTS_MODULE_ID_V1, CONTACTS_OWNER_ID_V1,
    contact_upsert_rejected_contract_reference_v1, contact_upsert_rejected_publish_request_v1,
    contact_upserted_contract_reference_v1, contact_upserted_publish_request_v1,
    upsert_contact_command_consume_request_v1, upsert_contact_command_contract_reference_v1,
};
use hermes_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const CONTACTS_STORAGE_CAPABILITY_ID_V1: &str = "contacts.storage.v1";
const CONTACTS_UPSERTED_PUBLISH_CAPABILITY_ID_V1: &str =
    "contacts.mail-identity.upserted.publisher.v1";
const CONTACTS_REJECTED_PUBLISH_CAPABILITY_ID_V1: &str =
    "contacts.mail-identity.rejected.publisher.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn contacts_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn contacts_settings_schema_bytes_v1() -> Vec<u8> {
    contacts_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn contacts_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = contacts_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: CONTACTS_MODULE_ID_V1.to_owned(),
        owner_id: CONTACTS_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            event_capability(
                CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurableConsumer,
                upsert_contact_command_contract_reference_v1(),
                upsert_contact_command_consume_request_v1(),
            ),
            event_capability(
                CONTACTS_REJECTED_PUBLISH_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                contact_upsert_rejected_contract_reference_v1(),
                contact_upsert_rejected_publish_request_v1(),
            ),
            event_capability(
                CONTACTS_UPSERTED_PUBLISH_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                contact_upserted_contract_reference_v1(),
                contact_upserted_publish_request_v1(),
            ),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Contacts".to_owned(),
    }
}

fn event_capability(
    capability_id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: kind as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![request],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CONTACTS_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: CONTACTS_OWNER_ID_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_contacts_owned_and_has_only_event_and_storage_authority() {
        let descriptor = contacts_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&contacts_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.owner_id, CONTACTS_OWNER_ID_V1);
        assert_eq!(descriptor.capabilities.len(), 4);
        assert!(
            descriptor
                .capabilities
                .iter()
                .all(|capability| capability.capability_id.starts_with("contacts."))
        );
    }
}
