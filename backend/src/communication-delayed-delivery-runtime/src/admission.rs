use hermes_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_CANCEL_CONNECT_PATH_V1,
    COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1, COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1,
    COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
    COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
    COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
};
use hermes_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ClockTimerRequestV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1,
    contracts::{
        delayed_delivery_cancel_command_contract_v1, delayed_delivery_query_contract_v1,
        delayed_delivery_realtime_contract_v1, delayed_delivery_schedule_command_contract_v1,
        delivery_intent_command_contract_v1,
    },
};

pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1: &str =
    "communication.delayed_delivery.storage.v1";
pub const COMMUNICATION_DELAYED_DELIVERY_CLOCK_CAPABILITY_ID_V1: &str =
    "communication.delayed_delivery.clock.v1";
pub const COMMUNICATION_DELAYED_DELIVERY_DELIVERY_DEPENDENCY_CAPABILITY_ID_V1: &str =
    "communication.delayed_delivery.delivery_intent.v1";
pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const COMMUNICATION_DELAYED_DELIVERY_BLOB_QUOTA_BYTES_V1: u64 = 256 * 1024 * 1024;

#[must_use]
pub fn communication_delayed_delivery_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communication_delayed_delivery_settings_schema_bytes_v1() -> Vec<u8> {
    communication_delayed_delivery_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communication_delayed_delivery_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = communication_delayed_delivery_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            blob_capability(),
            clock_capability(),
            delivery_dependency_capability(),
            storage_capability(),
            client_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: COMMUNICATION_DELAYED_DELIVERY_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Communication Delayed Delivery".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_surface(
                delayed_delivery_schedule_command_contract_v1(),
                COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
            ),
            client_surface(
                delayed_delivery_cancel_command_contract_v1(),
                COMMUNICATION_DELAYED_DELIVERY_CANCEL_CONNECT_PATH_V1,
            ),
            client_surface(
                delayed_delivery_query_contract_v1(),
                COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(delayed_delivery_realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_surface(
    contract: hermes_runtime_protocol::v1::ContractReferenceV1,
    path: &str,
) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn delivery_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELAYED_DELIVERY_DELIVERY_DEPENDENCY_CAPABILITY_ID_V1
            .to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![delivery_intent_command_contract_v1()],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1.to_owned(),
                connection_budget: COMMUNICATION_DELAYED_DELIVERY_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATION_DELAYED_DELIVERY_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn clock_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELAYED_DELIVERY_CLOCK_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::ClockTimer(ClockTimerRequestV1 {
                requires_wall_clock: true,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::{
        v1::capability_request_v1::Request,
        validation::descriptor::{validate_descriptor_v1, validate_settings_schema_v1},
    };

    use super::*;

    #[test]
    fn descriptor_routes_each_client_method_and_requests_exact_local_resources() {
        let descriptor = communication_delayed_delivery_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&communication_delayed_delivery_settings_schema_v1())
            .expect("settings");

        assert_eq!(descriptor.capabilities.len(), 5);
        let client = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1
            })
            .expect("client capability");
        assert_eq!(client.provides.len(), 4);
        assert_ne!(client.provides[0].contract, client.provides[1].contract);
        let dependency = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id
                    == COMMUNICATION_DELAYED_DELIVERY_DELIVERY_DEPENDENCY_CAPABILITY_ID_V1
            })
            .expect("delivery dependency");
        assert_eq!(
            dependency.dependencies,
            vec![delivery_intent_command_contract_v1()]
        );
        let clock = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == COMMUNICATION_DELAYED_DELIVERY_CLOCK_CAPABILITY_ID_V1
            })
            .expect("clock capability");
        assert!(matches!(
            clock.requests[0].request,
            Some(Request::ClockTimer(ClockTimerRequestV1 {
                requires_wall_clock: true,
            }))
        ));
    }
}
