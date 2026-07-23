use hermes_kernel_control_store::{
    InitialOwnerIdentity, ModuleClientRpcRouteV1, ModuleRegistration, ModuleRegistrationState,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, ClientRpcRouteV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
};
use prost::Message;

use crate::modules::registration::registry;

use super::common::unique_target_root;

#[test]
fn control_store_exposes_only_approved_owner_client_rpc_routes() {
    let root = unique_target_root("hermes-client-rpc-route");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            "owner_notes",
            "device_notes",
            [4; 65],
        ))
        .expect("claim initial owner");

    let registration = registry::register(&store, &descriptor().encode_to_vec())
        .expect("register descriptor-declared client route");
    assert!(
        store
            .approved_module_client_rpc_routes()
            .expect("read pending routes")
            .is_empty()
    );

    store
        .approve_module_registration(registration.registration_id(), &["notes.query".to_owned()])
        .expect("approve client route capability");
    assert_eq!(
        store
            .approved_module_client_rpc_routes()
            .expect("read approved routes"),
        vec![ModuleClientRpcRouteV1::new(
            registration.registration_id(),
            "notes.query",
            "owner_notes",
            "notes.query",
            hermes_kernel_control_store::ModuleClientRpcContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [7; 32],
            "/hermes.notes.v1.NotesQueryService/Query",
        )],
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_foreign_or_duplicate_client_rpc_routes_atomically() {
    let root = unique_target_root("hermes-client-rpc-route-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let valid = client_route("owner_notes", "/hermes.notes.v1.NotesQueryService/Query");
    let foreign = client_route("owner_other", "/hermes.notes.v1.NotesQueryService/Query");

    for routes in [vec![foreign], vec![valid.clone(), valid]] {
        assert!(
            store
                .create_pending_registration_with_all_descriptor_requests(
                    &registration(),
                    &["notes.query".to_owned()],
                    hermes_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                        storage: &[],
                        events: &[],
                        blobs: &[],
                        scheduler: &[],
                        vault_purposes: &[],
                        client_rpc_routes: &routes,
                    },
                )
                .is_err()
        );
    }
    assert!(
        store
            .module_registration("registration_notes")
            .expect("read registration")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn registration() -> ModuleRegistration {
    ModuleRegistration::new(
        "registration_notes",
        "module_notes",
        "owner_notes",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    )
}

fn client_route(owner: &str, path: &str) -> ModuleClientRpcRouteV1 {
    ModuleClientRpcRouteV1::new(
        "registration_notes",
        "notes.query",
        owner,
        "notes.query",
        hermes_kernel_control_store::ModuleClientRpcContractVersionV1 {
            major: 1,
            revision: 1,
        },
        [7; 32],
        path,
    )
}

fn descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: "owner_notes".to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "notes.query".to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            provides: vec![ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(ContractReferenceV1 {
                    owner: "owner_notes".to_owned(),
                    name: "notes.query".to_owned(),
                    major: 1,
                    revision: 1,
                    schema_sha256: vec![7; 32],
                }),
                client_rpc_route: Some(ClientRpcRouteV1 {
                    path: "/hermes.notes.v1.NotesQueryService/Query".to_owned(),
                }),
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
