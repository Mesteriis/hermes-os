use super::common::*;
use hermes_runtime_protocol::v1::{
    DeploymentProfileV1, DeviceProofV1, DistributionArtifactV1, FileDeviceProofV1,
    InitialOwnerEnrollmentTransportV1, NativeExecutableDigestV1, OciImageDigestV1,
    RemotePairingEnrollmentV1, RuntimeLifecycleV1, VaultCiphertextRouteDirectionV1,
    VaultCiphertextRouteV1, device_proof_v1, distribution_artifact_v1,
    initial_owner_enrollment_transport_v1,
};
use hermes_runtime_protocol::validation::deployment::{
    DeploymentValidationError, MACOS_TAURI_TARGET, validate_deployment_binding,
    validate_initial_owner_enrollment_transport,
};
use hermes_runtime_protocol::validation::vault::{
    VaultCiphertextRouteValidationError, validate_vault_ciphertext_route_v1,
};

#[test]
fn vault_ciphertext_route_requires_a_complete_current_transport_binding() {
    let mut route = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "registration-1".to_owned(),
        runtime_instance_id: "runtime-1".to_owned(),
        vault_runtime_generation: 1,
        grant_epoch: 1,
        request_id: vec![1; 16],
        operation_digest_sha256: vec![2; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![3; 32],
        ciphertext: vec![4],
        hpke_authentication_tag: vec![5; 16],
        response_recipient_hpke_public_key_x25519: vec![6; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
    };
    assert_eq!(validate_vault_ciphertext_route_v1(&route), Ok(()));
    route.response_recipient_hpke_public_key_x25519.pop();
    assert_eq!(
        validate_vault_ciphertext_route_v1(&route),
        Err(VaultCiphertextRouteValidationError::InvalidBinding)
    );
}

#[test]
fn vault_ciphertext_response_must_match_the_current_request_fence() {
    let request = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "registration-1".to_owned(),
        runtime_instance_id: "runtime-1".to_owned(),
        vault_runtime_generation: 7,
        grant_epoch: 3,
        request_id: vec![1; 16],
        operation_digest_sha256: vec![2; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![3; 32],
        ciphertext: vec![4],
        hpke_authentication_tag: vec![5; 16],
        response_recipient_hpke_public_key_x25519: vec![6; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
    };
    let response = hermes_runtime_protocol::v1::VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: 7,
        request_id: request.request_id.clone(),
        operation_digest_sha256: request.operation_digest_sha256.clone(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: vec![7; 32],
        ciphertext: vec![8],
        hpke_authentication_tag: vec![9; 16],
    };
    assert!(vault_ciphertext_route::validate_response(&request, response.clone()).is_ok());
    assert!(
        vault_ciphertext_route::validate_response(
            &request,
            hermes_runtime_protocol::v1::VaultCiphertextResponseV1 {
                vault_runtime_generation: 8,
                ..response
            },
        )
        .is_err()
    );
}

#[test]
fn managed_runtime_supervisor_prevents_duplicates_and_stops_on_kernel_shutdown() {
    let (root, staged, duplicate, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "registration-1".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");
    assert!(
        supervisor
            .is_active("registration-1")
            .expect("active runtime")
    );
    let duplicate_policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    assert_eq!(
        supervisor
            .start(
                "registration-1".to_owned(),
                duplicate,
                runtime_expectation(2, descriptor_digest),
                duplicate_policy,
            )
            .expect_err("duplicate runtime"),
        "managed runtime is already active for this registration"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    assert!(
        !supervisor
            .is_active("registration-1")
            .expect("reaped runtime")
    );
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_relays_bounded_opaque_frames_after_admission() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "registration-1".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");
    assert_eq!(
        supervisor
            .relay("registration-1", b"opaque-request".to_vec())
            .expect("relay opaque frame"),
        b"opaque-response"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_passes_explicit_arguments_to_the_staged_child() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    let contracts =
        StagedRuntimeContracts::stage(&root.join("contracts"), b"exact-verified-contract", None)
            .expect("stage runtime contracts");
    let descriptor_path = contracts.descriptor_path().to_owned();
    supervisor
        .start_with_arguments_and_contracts(
            "registration-1".to_owned(),
            staged,
            vec!["serve-inherited".to_owned()],
            runtime_expectation(1, descriptor_digest),
            policy,
            contracts,
        )
        .expect("start managed runtime");
    assert_eq!(
        supervisor
            .relay("registration-1", b"opaque-request".to_vec())
            .expect("relay opaque frame"),
        b"opaque-response-with-arguments"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    assert!(!descriptor_path.exists());
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_stops_one_rebound_process_without_global_shutdown() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "vault".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");
    supervisor.stop("vault").expect("stop one runtime");
    assert!(!shutdown_requested.load(Ordering::Acquire));
    assert!(!supervisor.is_active("vault").expect("reap runtime"));
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

fn prepare_supervisor_artifacts() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    staged_native_artifact::StagedNativeArtifact,
    [u8; 32],
) {
    let root = unique_target_root("hermes-managed-runtime-supervisor");
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
    let descriptor_digest = Sha256::digest(&descriptor_bytes).into();
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
    std::fs::create_dir_all(&root).expect("create fixture root");
    let source = root.join("managed-child.sh");
    let payload = [vec![request_bytes.len() as u8], request_bytes].concat();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count=15 of=/dev/null 2>/dev/null\nif [ \"$1\" = serve-inherited ]; then printf '\\036opaque-response-with-arguments' >&0; else printf '\\017opaque-response' >&0; fi\nsleep 30\n",
            shell_binary_literal(&payload)
        ),
    )
    .expect("write child script");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read child script")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "managed-child", &digest)
            .expect("stage child script");
    let duplicate = staged_native_artifact::stage(
        &source,
        &root.join("launch"),
        "managed-child-duplicate",
        &digest,
    )
    .expect("stage duplicate child script");
    (root, staged, duplicate, descriptor_digest)
}

fn runtime_expectation(generation: u64, descriptor_digest: [u8; 32]) -> ManagedRuntimeExpectation {
    ManagedRuntimeExpectation::new(
        "registration-1",
        "mail",
        generation,
        2,
        descriptor_digest,
        None,
    )
}

#[test]
fn settings_schema_requires_ordered_typed_non_secret_definitions() {
    let schema = SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![SettingDefinitionV1 {
            setting_id: "sync.interval".into(),
            capability_id: "capability.read".into(),
            value_type: SettingValueTypeV1::Duration as i32,
            mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
            target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
            apply_mode: SettingApplyModeV1::HotReload as i32,
            client_visibility: SettingClientVisibilityV1::Editable as i32,
            fresh_owner_proof_required: true,
            kernel_controller_id: String::new(),
            display_name: "Sync interval".into(),
        }],
    };
    assert!(decode_settings_schema_v1(&schema.encode_to_vec()).is_ok());
    let mut invalid = schema;
    invalid.definitions[0].mutation_authority = SettingMutationAuthorityV1::KernelManaged as i32;
    invalid.definitions[0].client_visibility = SettingClientVisibilityV1::Editable as i32;
    invalid.definitions[0].kernel_controller_id = "controller".into();
    assert!(decode_settings_schema_v1(&invalid.encode_to_vec()).is_err());
}

#[test]
fn settings_snapshot_requires_a_canonical_typed_value_set() {
    let snapshot = duration_settings_snapshot();
    assert!(decode_settings_snapshot_v1(&snapshot.encode_to_vec()).is_ok());
    let schema = duration_settings_schema();
    assert!(validate_settings_snapshot_against_schema_v1(&schema, &snapshot).is_ok());
    let mut wrong_type = snapshot.clone();
    wrong_type.values[0].value = Some(SettingValueV1 {
        value: Some(
            hermes_runtime_protocol::v1::setting_value_v1::Value::StringValue(
                "not a duration".into(),
            ),
        ),
    });
    assert!(validate_settings_snapshot_against_schema_v1(&schema, &wrong_type).is_err());
    let mut unknown_setting = snapshot.clone();
    unknown_setting.values[0].setting_id = "unknown.setting".into();
    assert!(validate_settings_snapshot_against_schema_v1(&schema, &unknown_setting).is_err());
    assert_oversized_setting_is_rejected(&schema, &snapshot);
    let mut missing = snapshot;
    missing.values[0].value = None;
    assert!(decode_settings_snapshot_v1(&missing.encode_to_vec()).is_err());
}

fn duration_settings_snapshot() -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: "registration-1".into(),
        revision: 1,
        values: vec![SettingsValueEntryV1 {
            setting_id: "sync.interval".into(),
            value: Some(SettingValueV1 {
                value: Some(
                    hermes_runtime_protocol::v1::setting_value_v1::Value::DurationMillis(1000),
                ),
            }),
        }],
    }
}

fn duration_settings_schema() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![SettingDefinitionV1 {
            setting_id: "sync.interval".into(),
            capability_id: String::new(),
            value_type: SettingValueTypeV1::Duration as i32,
            mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
            target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
            apply_mode: SettingApplyModeV1::HotReload as i32,
            client_visibility: SettingClientVisibilityV1::Editable as i32,
            fresh_owner_proof_required: false,
            kernel_controller_id: String::new(),
            display_name: "Sync interval".into(),
        }],
    }
}

fn assert_oversized_setting_is_rejected(schema: &SettingsSchemaV1, snapshot: &SettingsSnapshotV1) {
    let mut oversized = snapshot.clone();
    oversized.values[0].value = Some(SettingValueV1 {
        value: Some(
            hermes_runtime_protocol::v1::setting_value_v1::Value::StringValue("x".repeat(8193)),
        ),
    });
    let string_schema = SettingsSchemaV1 {
        definitions: vec![SettingDefinitionV1 {
            value_type: SettingValueTypeV1::String as i32,
            ..schema.definitions[0].clone()
        }],
        ..schema.clone()
    };
    assert!(validate_settings_snapshot_against_schema_v1(&string_schema, &oversized).is_err());
}

#[test]
fn initial_enrollment_contract_is_fixed_to_p256_and_a_single_pristine_generation() {
    let challenge = InitialOwnerEnrollmentChallengeV1 {
        protocol_major: 1,
        instance_id: vec![1; 32],
        nonce: vec![2; 32],
        kernel_generation: 1,
    };
    let proof = InitialOwnerEnrollmentV1 {
        protocol_major: 1,
        device_public_key_sec1: [vec![0x04], vec![3; 64]].concat(),
        challenge_signature_raw: vec![4; 64],
        owner_id: "owner-1".to_owned(),
        device_id: "device-1".to_owned(),
    };
    assert!(validate_initial_owner_enrollment(&challenge, &proof));
    assert!(!validate_initial_owner_enrollment(
        &InitialOwnerEnrollmentChallengeV1 {
            kernel_generation: 2,
            ..challenge
        },
        &proof
    ));
}

#[test]
fn deployment_contracts_bind_profile_lifecycle_artifact_and_file_proof() {
    let proof = DeviceProofV1 {
        proof: Some(device_proof_v1::Proof::FileEs256(FileDeviceProofV1 {
            public_key_sec1: [vec![0x04], vec![7; 64]].concat(),
            signature_raw: vec![8; 64],
        })),
    };
    let artifact = DistributionArtifactV1 {
        artifact: Some(distribution_artifact_v1::Artifact::NativeExecutable(
            NativeExecutableDigestV1 {
                target_triple: MACOS_TAURI_TARGET.to_owned(),
                sha256: vec![9; 32],
            },
        )),
    };
    assert!(
        validate_deployment_binding(
            DeploymentProfileV1::MacosTauriEmbedded as i32,
            RuntimeLifecycleV1::ManagedChild as i32,
            &artifact,
        )
        .is_ok()
    );
    assert_eq!(
        validate_deployment_binding(
            DeploymentProfileV1::MacosTauriEmbedded as i32,
            RuntimeLifecycleV1::ExternalCompose as i32,
            &artifact,
        ),
        Err(DeploymentValidationError::InvalidLifecycle)
    );
    let oci_artifact = DistributionArtifactV1 {
        artifact: Some(distribution_artifact_v1::Artifact::OciImage(
            OciImageDigestV1 {
                repository: "registry.example/hermes/kernel".to_owned(),
                sha256: vec![12; 32],
            },
        )),
    };
    assert!(
        validate_deployment_binding(
            DeploymentProfileV1::LinuxDockerServer as i32,
            RuntimeLifecycleV1::ExternalCompose as i32,
            &oci_artifact,
        )
        .is_ok()
    );
    assert_remote_pairing_transport(proof);
}

fn assert_remote_pairing_transport(proof: DeviceProofV1) {
    let remote = InitialOwnerEnrollmentTransportV1 {
        transport: Some(
            initial_owner_enrollment_transport_v1::Transport::RemotePairing(
                RemotePairingEnrollmentV1 {
                    endpoint: "https://127.0.0.1:9443".to_owned(),
                    tls_certificate_sha256: vec![10; 32],
                    one_time_token: vec![11; 32],
                    device_proof: Some(proof),
                },
            ),
        ),
    };
    assert!(validate_initial_owner_enrollment_transport(&remote).is_ok());
    let invalid_proof = InitialOwnerEnrollmentTransportV1 {
        transport: Some(
            initial_owner_enrollment_transport_v1::Transport::RemotePairing(
                RemotePairingEnrollmentV1 {
                    endpoint: "https://127.0.0.1:9443".to_owned(),
                    tls_certificate_sha256: vec![10; 32],
                    one_time_token: vec![11; 32],
                    device_proof: Some(DeviceProofV1::default()),
                },
            ),
        ),
    };
    assert_eq!(
        validate_initial_owner_enrollment_transport(&invalid_proof),
        Err(DeploymentValidationError::InvalidDeviceProof)
    );
}
