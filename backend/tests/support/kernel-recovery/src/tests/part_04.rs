use super::common::*;
use hermes_runtime_protocol::v1::{
    CapabilityRequestV1, VaultPurposeRequestV1, capability_request_v1,
};

#[test]
fn descriptor_validation_requires_a_bounded_typed_vault_purpose() {
    let vault_purpose = VaultPurposeRequestV1 {
        purpose_id: "provider.oauth".into(),
        requested_lease_ttl_seconds: 600,
        allowed_secret_classes: vec![
            VaultSecretClassV1::ProviderCredential as i32,
            VaultSecretClassV1::OauthRefreshCredential as i32,
        ],
        actions: vec![VaultActionV1::Resolve as i32],
        target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
    };
    let descriptor = descriptor_with_vault_purpose(vault_purpose.clone());
    assert!(decode_descriptor_v1(&descriptor.encode_to_vec()).is_ok());
    let mut invalid = descriptor.clone();
    let Some(capability_request_v1::Request::VaultPurpose(purpose)) =
        invalid.capabilities[0].requests[0].request.as_mut()
    else {
        panic!("vault purpose fixture");
    };
    purpose.actions = vec![VaultActionV1::Resolve as i32, VaultActionV1::Resolve as i32];
    assert!(decode_descriptor_v1(&invalid.encode_to_vec()).is_err());
    assert_invalid_vault_purposes(&descriptor, vault_purpose);
}

fn descriptor_with_vault_purpose(vault_purpose: VaultPurposeRequestV1) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "credentials".into(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            requests: vec![CapabilityRequestV1 {
                request: Some(capability_request_v1::Request::VaultPurpose(vault_purpose)),
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn assert_invalid_vault_purposes(
    descriptor: &ModuleDescriptorV1,
    vault_purpose: VaultPurposeRequestV1,
) {
    let invalid_purposes = [
        VaultPurposeRequestV1 {
            purpose_id: "".into(),
            ..vault_purpose.clone()
        },
        VaultPurposeRequestV1 {
            requested_lease_ttl_seconds: 3_601,
            ..vault_purpose.clone()
        },
        VaultPurposeRequestV1 {
            allowed_secret_classes: vec![99],
            ..vault_purpose.clone()
        },
        VaultPurposeRequestV1 {
            actions: vec![],
            ..vault_purpose.clone()
        },
        VaultPurposeRequestV1 {
            target_scope: VaultTargetScopeV1::Unspecified as i32,
            ..vault_purpose
        },
    ];
    for purpose in invalid_purposes {
        let mut invalid = descriptor.clone();
        invalid.capabilities[0].requests[0] = CapabilityRequestV1 {
            request: Some(capability_request_v1::Request::VaultPurpose(purpose)),
        };
        assert!(decode_descriptor_v1(&invalid.encode_to_vec()).is_err());
    }
}

#[test]
fn inherited_managed_runtime_control_accepts_only_the_bound_descriptor() {
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "read".into(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            ..Default::default()
        }],
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "registration-1",
        "mail",
        3,
        7,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let (server_stream, mut child_stream) = UnixStream::pair().expect("create inherited pair");
    let server = std::thread::spawn(move || {
        managed_runtime_control::establish_channel(server_stream, &expectation)
    });
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(
            hermes_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    write_test_frame(&mut child_stream, &request.encode_to_vec());
    let response =
        ManagedRuntimeControlResponseV1::decode(read_test_frame(&mut child_stream).as_slice())
            .expect("decode control response");
    assert!(response.error_code.is_empty());
    assert!(matches!(
        response.result,
        Some(hermes_runtime_protocol::v1::managed_runtime_control_response_v1::Result::Describe(ref describe))
            if describe.registration_id == "registration-1" && describe.runtime_generation == 3 && describe.grant_epoch == 7
    ));
    server
        .join()
        .expect("server thread")
        .expect("describe accepted");
}

#[test]
fn inherited_managed_runtime_control_rejects_a_replaced_descriptor() {
    let expected_descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let expectation = ManagedRuntimeExpectation::new(
        "registration-1",
        "mail",
        1,
        2,
        Sha256::digest(expected_descriptor.encode_to_vec()).into(),
        None,
    );
    let (server_stream, mut child_stream) = UnixStream::pair().expect("create inherited pair");
    let server = std::thread::spawn(move || {
        managed_runtime_control::establish_channel(server_stream, &expectation)
    });
    let replaced = ModuleDescriptorV1 {
        module_id: "telegram".into(),
        ..expected_descriptor
    };
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(
            hermes_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes: replaced.encode_to_vec(),
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    write_test_frame(&mut child_stream, &request.encode_to_vec());
    let response =
        ManagedRuntimeControlResponseV1::decode(read_test_frame(&mut child_stream).as_slice())
            .expect("decode rejection response");
    assert_eq!(response.error_code, "managed_runtime_describe_rejected");
    assert!(
        server
            .join()
            .expect("server thread")
            .expect_err("replaced descriptor")
            .contains("digest does not match")
    );
}

#[test]
fn inherited_managed_runtime_control_relays_only_bounded_opaque_frames_after_describe() {
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "registration-1",
        "mail",
        3,
        7,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let (server_stream, mut child_stream) = UnixStream::pair().expect("create inherited pair");
    let server = std::thread::spawn(move || {
        let mut channel = managed_runtime_control::establish_channel(server_stream, &expectation)?;
        managed_runtime_control::relay(&mut channel, b"opaque-request")
    });
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(
            hermes_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    write_test_frame(&mut child_stream, &describe.encode_to_vec());
    let _ = read_test_frame(&mut child_stream);
    assert_eq!(read_test_frame(&mut child_stream), b"opaque-request");
    write_test_frame(&mut child_stream, b"opaque-response");
    assert_eq!(
        server.join().expect("server thread").expect("opaque relay"),
        b"opaque-response"
    );
}

#[test]
fn managed_runtime_expectation_rejects_a_stale_persisted_launch_fence() {
    let registration = ModuleRegistration::new(
        "registration-1",
        "mail",
        "communications",
        [7; 32],
        ModuleRegistrationState::Approved,
        2,
    );
    let binding = BundledManagedLaunchBinding::new(
        "registration-1",
        1,
        "hermes-desktop",
        "runtime.mail",
        [8; 32],
        [7; 32],
        None,
    );
    let stale_record = ManagedLaunchRecord::new("registration-1", 1, 1, 1, 3);

    assert_eq!(
        ManagedRuntimeExpectation::from_fenced_launch(&registration, &binding, &stale_record)
            .expect_err("stale grant epoch"),
        "managed launch fence does not match its approved registration"
    );
}

#[test]
fn macos_release_resource_locator_requires_the_signed_bundle_resource_layout() {
    let fixture_name = format!(
        "hermes-macos-release-resources-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let root = std::env::current_dir()
        .expect("current directory")
        .join("target")
        .join(fixture_name);
    let executable = root.join("Hermes.app/Contents/MacOS/hermes-kernel");
    let resource_root = root.join("Hermes.app/Contents/Resources/hermes-kernel-release");
    std::fs::create_dir_all(resource_root.join("distribution/bin"))
        .expect("create resource layout");
    std::fs::create_dir_all(executable.parent().expect("executable parent"))
        .expect("create executable directory");
    std::fs::write(&executable, b"kernel").expect("write executable");
    std::fs::write(
        resource_root.join("hermes-release-trust-root.pb"),
        b"trust-root",
    )
    .expect("write trust root");
    std::fs::write(
        resource_root.join("hermes-signed-distribution-manifest.pb"),
        b"manifest",
    )
    .expect("write signed manifest");

    let resources = macos_release_resources::discover_from_executable(&executable)
        .expect("discover release resources");
    assert_eq!(
        resources.distribution_root(),
        resource_root.join("distribution")
    );
    assert_eq!(
        resources.signed_manifest_path(),
        resource_root.join("hermes-signed-distribution-manifest.pb")
    );
    assert_eq!(
        resources.trust_root_path(),
        resource_root.join("hermes-release-trust-root.pb")
    );
    std::fs::remove_dir_all(root).expect("remove release resource fixture");
}

#[test]
fn macos_release_resource_locator_rejects_a_symlinked_app_bundle() {
    let fixture_name = format!(
        "hermes-macos-release-symlink-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let root = std::env::current_dir()
        .expect("current directory")
        .join("target")
        .join(fixture_name);
    let real_bundle = root.join("real/Hermes.app");
    let executable = real_bundle.join("Contents/MacOS/hermes-kernel");
    std::fs::create_dir_all(executable.parent().expect("executable parent"))
        .expect("create executable directory");
    std::fs::write(&executable, b"kernel").expect("write executable");
    let linked_bundle = root.join("linked.app");
    std::os::unix::fs::symlink(&real_bundle, &linked_bundle).expect("link app bundle");

    let linked_executable = linked_bundle.join("Contents/MacOS/hermes-kernel");
    assert!(macos_release_resources::discover_from_executable(&linked_executable).is_err());
    std::fs::remove_dir_all(root).expect("remove symlink fixture");
}

#[test]
fn macos_release_resource_locator_rejects_a_symlinked_resources_directory() {
    let fixture_name = format!(
        "hermes-macos-release-resource-directory-symlink-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let root = std::env::current_dir()
        .expect("current directory")
        .join("target")
        .join(fixture_name);
    let executable = root.join("Hermes.app/Contents/MacOS/hermes-kernel");
    let external_resources = root.join("external-resources");
    let external_resource_root = external_resources.join("hermes-kernel-release");
    std::fs::create_dir_all(executable.parent().expect("executable parent"))
        .expect("create executable directory");
    std::fs::write(&executable, b"kernel").expect("write executable");
    std::fs::create_dir_all(external_resource_root.join("distribution"))
        .expect("create external resource layout");
    std::fs::write(
        external_resource_root.join("hermes-release-trust-root.pb"),
        b"trust-root",
    )
    .expect("write external trust root");
    std::fs::write(
        external_resource_root.join("hermes-signed-distribution-manifest.pb"),
        b"manifest",
    )
    .expect("write external signed manifest");
    std::os::unix::fs::symlink(
        &external_resources,
        root.join("Hermes.app/Contents/Resources"),
    )
    .expect("link resources directory");

    assert!(macos_release_resources::discover_from_executable(&executable).is_err());
    std::fs::remove_dir_all(root).expect("remove resource symlink fixture");
}

#[test]
fn managed_child_supervisor_requires_describe_on_the_inherited_fd() {
    let root = unique_target_root("hermes-managed-child-supervisor");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(
            hermes_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes: descriptor_bytes.clone(),
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let request_bytes = request.encode_to_vec();
    assert!(
        request_bytes.len() < 128,
        "test request uses one-byte frame length"
    );
    std::fs::create_dir_all(&root).expect("create fixture root");
    let source = root.join("managed-child.sh");
    let payload = [vec![request_bytes.len() as u8], request_bytes].concat();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\nIFS= read -r ignored || true\n",
            shell_binary_literal(&payload)
        ),
    )
    .expect("write child script");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read child script")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "managed-child", &digest)
            .expect("stage child script");
    let expectation = ManagedRuntimeExpectation::new(
        "registration-1",
        "mail",
        1,
        2,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(2)).expect("execution policy");

    let result = managed_child_supervisor::run(&staged, &[], &expectation, &policy)
        .expect("managed child describe");
    assert_managed_child_completed(result);
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

fn assert_managed_child_completed(
    result: bounded_managed_child_execution::ManagedChildExecutionResult,
) {
    assert_eq!(result.attempts(), 1);
    assert_eq!(result.exit_code(), 0);
}
