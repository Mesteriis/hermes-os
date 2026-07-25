//! Exact, still-unadmitted descriptor for the Zulip integration runtime.
//!
//! Client ports and platform dependencies remain separate capability units.
//! This descriptor does not register Zulip or grant any capability.

use hermes_communications_ingress::admission::communication_observed_publish_request_v1;
use hermes_runtime_protocol::v1::{
    BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
    ClientRpcRouteV1, ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    StorageNamespaceRequestV1, VaultActionV1, VaultPurposeRequestV1, VaultSecretClassV1,
    VaultTargetScopeV1, capability_request_v1::Request,
};
use hermes_zulip_api::client_contract::{
    ZULIP_CLIENT_CONTRACT_MAJOR, ZULIP_CLIENT_CONTRACT_REVISION, ZULIP_CLIENT_DESCRIPTOR_SET_V1,
    ZulipClientContractV1,
};
pub use hermes_zulip_api::client_contract::{ZULIP_MODULE_ID, ZULIP_OWNER_ID};
use hermes_zulip_core::ZULIP_API_KEY_PURPOSE_ID;
use sha2::{Digest, Sha256};

use crate::settings::{
    ZULIP_SETTINGS_SCHEMA_MAJOR_V1, ZULIP_SETTINGS_SCHEMA_REVISION_V1,
    zulip_settings_schema_bytes_v1,
};

pub const ZULIP_BLOB_CAPABILITY_ID: &str = "zulip.blob.v1";
pub const ZULIP_CREDENTIALS_CAPABILITY_ID: &str = "zulip.credentials.v1";
pub const ZULIP_EVENTS_CAPABILITY_ID: &str = "zulip.events.v1";
pub const ZULIP_STORAGE_CAPABILITY_ID: &str = "zulip.storage.v1";
pub const ZULIP_BLOB_QUOTA_BYTES: u64 = 64 * 1024 * 1024;
pub const ZULIP_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const ZULIP_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const ZULIP_CREDENTIAL_LEASE_TTL_SECONDS: u32 = 60;

#[must_use]
pub fn zulip_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        zulip_blob_capability_v1(),
        zulip_client_capability_v1(ZulipClientContractV1::Command),
        zulip_credentials_capability_v1(),
        zulip_events_capability_v1(),
        zulip_client_capability_v1(ZulipClientContractV1::Query),
        zulip_storage_capability_v1(),
    ]
}

fn zulip_client_capability_v1(contract: ZulipClientContractV1) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: contract.capability_id().to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(zulip_client_contract_reference_v1(contract)),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: contract.connect_path().to_owned(),
            }),
        }],
        ..Default::default()
    }
}

fn zulip_client_contract_reference_v1(contract: ZulipClientContractV1) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ZULIP_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: ZULIP_CLIENT_CONTRACT_MAJOR,
        revision: ZULIP_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(ZULIP_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn zulip_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZULIP_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: ZULIP_BLOB_QUOTA_BYTES,
            })),
        }],
        ..Default::default()
    }
}

fn zulip_credentials_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZULIP_CREDENTIALS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: ZULIP_API_KEY_PURPOSE_ID.to_owned(),
                requested_lease_ttl_seconds: ZULIP_CREDENTIAL_LEASE_TTL_SECONDS,
                allowed_secret_classes: vec![VaultSecretClassV1::ProviderCredential as i32],
                actions: vec![VaultActionV1::Resolve as i32],
                target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
                key_schema_revision: 0,
            })),
        }],
        ..Default::default()
    }
}

fn zulip_events_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZULIP_EVENTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![communication_observed_publish_request_v1()],
        ..Default::default()
    }
}

fn zulip_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZULIP_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ZULIP_OWNER_ID.to_owned(),
                connection_budget: ZULIP_STORAGE_CONNECTION_BUDGET,
                timeout_millis: ZULIP_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn zulip_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = zulip_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ZULIP_MODULE_ID.to_owned(),
        owner_id: ZULIP_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: zulip_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: ZULIP_SETTINGS_SCHEMA_MAJOR_V1,
            revision: ZULIP_SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: ZULIP_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 256 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Zulip".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::{
        v1::{CapabilityCriticalityV1, ModuleKindV1, ProvidedSurfaceKindV1},
        validation::descriptor::validate_descriptor_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_valid_and_keeps_client_and_platform_capabilities_separate() {
        let descriptor = zulip_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                ZULIP_BLOB_CAPABILITY_ID,
                ZulipClientContractV1::Command.capability_id(),
                ZULIP_CREDENTIALS_CAPABILITY_ID,
                ZULIP_EVENTS_CAPABILITY_ID,
                ZulipClientContractV1::Query.capability_id(),
                ZULIP_STORAGE_CAPABILITY_ID,
            ]
        );

        for contract in ZulipClientContractV1::ALL {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == contract.capability_id())
                .expect("Zulip client capability");
            assert_eq!(
                capability.criticality,
                CapabilityCriticalityV1::Optional as i32
            );
            assert_eq!(capability.provides.len(), 1);
            assert_eq!(
                capability.provides[0].kind,
                ProvidedSurfaceKindV1::ClientRpc as i32
            );
            assert_eq!(
                capability.provides[0]
                    .client_rpc_route
                    .as_ref()
                    .expect("Zulip client route")
                    .path,
                contract.connect_path()
            );
        }

        let credentials = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == ZULIP_CREDENTIALS_CAPABILITY_ID)
            .expect("Zulip credential capability");
        assert!(matches!(
            credentials.requests[0].request.as_ref(),
            Some(Request::VaultPurpose(request))
                if request.purpose_id == ZULIP_API_KEY_PURPOSE_ID
        ));

        let events = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == ZULIP_EVENTS_CAPABILITY_ID)
            .expect("Zulip events capability");
        assert_eq!(events.provides, []);
        assert!(matches!(
            events.requests[0].request,
            Some(Request::EventRoute(_))
        ));
    }
}
