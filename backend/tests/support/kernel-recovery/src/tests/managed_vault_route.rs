use super::common::*;
use hermes_runtime_protocol::v1::{ManagedRuntimeVaultRouteRequestV1, VaultCiphertextResponseV1};
use hermes_runtime_protocol::validation::vault::STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1;
use hermes_vault_protocol::VaultTransportCommandV1;
use std::io::{Read, Write};

use crate::runtime::lifecycle::control::{
    ManagedRuntimeVaultRouteHandler, relay_with_vault_routes,
};

#[test]
fn runtime_protocol_keeps_the_revoking_storage_route_fence_exact() {
    assert_eq!(
        STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1,
        VaultTransportCommandV1::RevokeAudience.operation_digest()
    );
}

#[test]
fn managed_runtime_routes_vault_ciphertext_only_after_descriptor_handshake() {
    let (root, staged, expectation) = route_child_fixture();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicU64::new(0));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    supervisor
        .configure_vault_route_handler(Arc::new(RecordingRouteHandler {
            calls: Arc::clone(&calls),
        }))
        .expect("configure handler before runtime launch");
    supervisor
        .start(
            "storage-control".to_owned(),
            staged,
            expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed runtime");

    assert!(
        wait_for_route(&calls),
        "typed route reaches the Kernel handler"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    std::fs::remove_dir_all(root).expect("remove route fixture");
}

#[test]
fn relay_completes_a_request_after_a_nested_vault_route() {
    let (mut kernel, mut child) = UnixStream::pair().expect("relay channel");
    let (root, _staged, expectation) = route_child_fixture();
    let calls = Arc::new(AtomicU64::new(0));
    let handler = RecordingRouteHandler {
        calls: Arc::clone(&calls),
    };
    let route = valid_route();
    let worker = std::thread::spawn(move || {
        assert_eq!(read_frame(&mut child), b"revoke");
        write_frame(
            &mut child,
            &ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
        );
        let _ = read_frame(&mut child);
        write_bytes(&mut child, b"revoked");
    });

    assert_eq!(
        relay_with_vault_routes(&mut kernel, b"revoke", &expectation, Some(&handler))
            .expect("relay completes"),
        b"revoked"
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);
    worker.join().expect("relay worker");
    std::fs::remove_dir_all(root).expect("remove relay fixture");
}

struct RecordingRouteHandler {
    calls: Arc<AtomicU64>,
}

impl ManagedRuntimeVaultRouteHandler for RecordingRouteHandler {
    fn route_vault_ciphertext(
        &self,
        expectation: &ManagedRuntimeExpectation,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, String> {
        if expectation.registration_id() != "storage-control"
            || route.registration_id != "storage-control"
            || route.caller_runtime_generation != expectation.runtime_generation()
            || route.grant_epoch != expectation.grant_epoch()
        {
            return Err("route fence mismatch".to_owned());
        }
        self.calls.fetch_add(1, Ordering::Release);
        Ok(VaultCiphertextResponseV1 {
            major: 1,
            vault_runtime_generation: route.vault_runtime_generation,
            caller_runtime_generation: route.caller_runtime_generation,
            request_id: route.request_id,
            operation_digest_sha256: route.operation_digest_sha256,
            direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
            hpke_encapped_key: vec![1; 32],
            ciphertext: vec![2],
            hpke_authentication_tag: vec![3; 16],
        })
    }
}

fn route_child_fixture() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    ManagedRuntimeExpectation,
) {
    let root = unique_target_root("hermes-managed-vault-route");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "storage".into(),
        owner_id: "platform".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "storage-control",
        "storage-runtime",
        "storage",
        5,
        7,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let staged = stage_route_child(&root, &descriptor_bytes);
    (root, staged, expectation)
}

fn stage_route_child(
    root: &std::path::Path,
    descriptor_bytes: &[u8],
) -> staged_native_artifact::StagedNativeArtifact {
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(
            hermes_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes: descriptor_bytes.to_vec(),
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let route = ManagedRuntimeVaultRouteRequestV1 {
        route: Some(valid_route()),
    };
    let payload = route_child_payload(describe, route);
    let source = root.join("managed-route-child.sh");
    std::fs::create_dir_all(root).expect("create route fixture");
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\nsleep 30\n",
            shell_binary_literal(&payload)
        ),
    )
    .expect("write route child");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&source).expect("read route child")).into();
    staged_native_artifact::stage(&source, &root.join("launch"), "route-child", &digest)
        .expect("stage route child")
}

fn valid_route() -> VaultCiphertextRouteV1 {
    VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "storage-control".into(),
        runtime_instance_id: "storage-runtime".into(),
        caller_runtime_generation: 5,
        vault_runtime_generation: 3,
        grant_epoch: 7,
        request_id: vec![1; 16],
        operation_digest_sha256: vec![2; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![3; 32],
        ciphertext: vec![4],
        hpke_authentication_tag: vec![5; 16],
        response_recipient_hpke_public_key_x25519: vec![6; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    }
}

fn route_child_payload(
    describe: ManagedRuntimeControlRequestV1,
    route: ManagedRuntimeVaultRouteRequestV1,
) -> Vec<u8> {
    [
        frame(&describe.encode_to_vec()),
        frame(&route.encode_to_vec()),
    ]
    .concat()
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len() + 5);
    let mut length = u32::try_from(bytes.len()).expect("bounded route frame");
    while length >= 0x80 {
        result.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    result.push(length as u8);
    result.extend_from_slice(bytes);
    result
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_usize;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).expect("frame prefix");
        length |= usize::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![0; length];
            stream.read_exact(&mut bytes).expect("frame body");
            return bytes;
        }
    }
    panic!("frame prefix exceeds bound");
}

fn write_frame(stream: &mut UnixStream, message: &impl Message) {
    write_bytes(stream, &message.encode_to_vec());
}

fn write_bytes(stream: &mut UnixStream, bytes: &[u8]) {
    stream.write_all(&frame(bytes)).expect("write frame");
    stream.flush().expect("flush frame");
}

fn wait_for_route(calls: &AtomicU64) -> bool {
    // Process spawn and the inherited-FD handshake are intentionally external to
    // this test process. Keep the assertion bounded, but do not turn ordinary
    // scheduler contention from the parallel recovery suite into a route failure.
    for _ in 0..200 {
        if calls.load(Ordering::Acquire) == 1 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}
