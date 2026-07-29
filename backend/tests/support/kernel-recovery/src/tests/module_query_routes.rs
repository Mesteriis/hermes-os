use hermes_kernel_control_store::{InitialOwnerIdentity, ModuleQueryContractV1};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, ContractReferenceV1, ModuleDescriptorV1,
    ModuleKindV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
};
use prost::Message;

use crate::modules::registration::registry;

use super::common::unique_target_root;

const OWNER: &str = "owner_notes";
const PROVIDER_CAPABILITY: &str = "notes.query";
const CALLER_CAPABILITY: &str = "notes.compose";

#[test]
fn query_providers_are_approval_gated_and_dependencies_remain_capability_scoped() {
    let root = unique_target_root("hermes-module-query-route");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(OWNER, "device_notes", [4; 65]))
        .expect("claim initial owner");

    let registration = registry::register(&store, &descriptor(OWNER).encode_to_vec())
        .expect("register query contracts");
    assert!(
        store
            .approved_module_query_rpc_routes()
            .expect("read pending routes")
            .is_empty()
    );
    assert_eq!(
        store
            .module_contract_dependencies(registration.registration_id(), CALLER_CAPABILITY)
            .expect("read caller dependency"),
        vec![contract_record(
            registration.registration_id(),
            CALLER_CAPABILITY
        )]
    );

    store
        .approve_module_registration(
            registration.registration_id(),
            &[CALLER_CAPABILITY.to_owned(), PROVIDER_CAPABILITY.to_owned()],
        )
        .expect("approve query capabilities");
    assert_eq!(
        store
            .approved_module_query_rpc_routes()
            .expect("read approved route"),
        vec![contract_record(
            registration.registration_id(),
            PROVIDER_CAPABILITY
        )]
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn query_provider_contract_must_be_owned_by_the_registered_module_owner() {
    let root = unique_target_root("hermes-module-query-foreign-owner");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(OWNER, "device_notes", [4; 65]))
        .expect("claim initial owner");

    assert!(
        registry::register(&store, &descriptor("owner_other").encode_to_vec()).is_err(),
        "provider route cannot claim another owner contract",
    );
    assert!(
        store
            .approved_module_query_rpc_routes()
            .expect("read routes")
            .is_empty()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn descriptor(provider_owner: &str) -> ModuleDescriptorV1 {
    let contract = ContractReferenceV1 {
        owner: provider_owner.to_owned(),
        name: "notes.canonical.query".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: vec![7; 32],
    };
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: OWNER.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![
            CapabilityDescriptorV1 {
                capability_id: CALLER_CAPABILITY.to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                dependencies: vec![contract.clone()],
                ..Default::default()
            },
            CapabilityDescriptorV1 {
                capability_id: PROVIDER_CAPABILITY.to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                provides: vec![ProvidedSurfaceV1 {
                    kind: ProvidedSurfaceKindV1::QueryRpc as i32,
                    contract: Some(contract.clone()),
                    client_rpc_route: None,
                    client_blob_route: None,
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn contract_record(registration_id: &str, capability_id: &str) -> ModuleQueryContractV1 {
    ModuleQueryContractV1::new(
        registration_id,
        capability_id,
        OWNER,
        "notes.canonical.query",
        1,
        1,
        [7; 32],
    )
}
