use hermes_kernel_control_store::{
    InitialOwnerIdentity, ModuleBlobQuotaRequestV1, ModuleRegistration, ModuleRegistrationState,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::v1::{
    BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
    ModuleDescriptorV1, ModuleKindV1, capability_request_v1::Request,
};
use prost::Message;

use crate::modules::registration::registry;
use crate::platform::blob::catalog;

use super::common::unique_target_root;

#[test]
fn blob_quotas_become_visible_only_after_exact_capability_approval() {
    let root = unique_target_root("hermes-blob-quota-request");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let request = blob_request("blob.content", 16 * 1024 * 1024);

    store
        .create_pending_registration_with_requests(
            &registration(),
            &["blob.content".to_owned(), "events.publish".to_owned()],
            &[],
            &[],
            std::slice::from_ref(&request),
        )
        .expect("persist pending registration and Blob request together");
    assert_eq!(
        store
            .module_blob_quota_request("registration_notes", "blob.content")
            .expect("read retained Blob request"),
        Some(request.clone())
    );
    assert!(
        catalog::resolve(&store)
            .expect("pending registration has no Blob catalog")
            .is_empty()
    );

    store
        .approve_module_registration("registration_notes", &["blob.content".to_owned()])
        .expect("approve Blob capability");
    let entries = catalog::resolve(&store).expect("resolve approved Blob catalog");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].registration_id(), "registration_notes");
    assert_eq!(entries[0].module_id(), "module_notes");
    assert_eq!(entries[0].grant_epoch(), 2);
    assert_eq!(entries[0].capability_id(), "blob.content");
    assert_eq!(entries[0].request(), &request);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_invalid_or_unrequested_blob_quotas_atomically() {
    let root = unique_target_root("hermes-blob-quota-request-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");

    for request in [
        blob_request("unrequested", 1),
        blob_request("blob.content", 0),
        blob_request("blob.content", (1 << 40) + 1),
    ] {
        assert!(
            store
                .create_pending_registration_with_requests(
                    &registration(),
                    &["blob.content".to_owned()],
                    &[],
                    &[],
                    &[request],
                )
                .is_err()
        );
    }
    assert!(
        store
            .module_registration("registration_notes")
            .expect("registration remains absent after rejected request")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn module_registration_retains_a_descriptor_declared_blob_quota() {
    let root = unique_target_root("hermes-blob-descriptor-registration");
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

    let registration = registry::register(&store, &descriptor(64 * 1024 * 1024).encode_to_vec())
        .expect("register Blob descriptor");
    assert_eq!(
        store
            .module_blob_quota_request(registration.registration_id(), "blob.content")
            .expect("read descriptor-declared Blob quota"),
        Some(ModuleBlobQuotaRequestV1::new(
            registration.registration_id(),
            "blob.content",
            "owner_notes",
            64 * 1024 * 1024,
        ))
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

fn blob_request(capability_id: &str, max_bytes: u64) -> ModuleBlobQuotaRequestV1 {
    ModuleBlobQuotaRequestV1::new(
        "registration_notes",
        capability_id,
        "owner_notes",
        max_bytes,
    )
}

fn descriptor(max_bytes: u64) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: "owner_notes".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "blob.content".to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            requests: vec![CapabilityRequestV1 {
                request: Some(Request::BlobQuota(BlobQuotaRequestV1 { max_bytes })),
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
