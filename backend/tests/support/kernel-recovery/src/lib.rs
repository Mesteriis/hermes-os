#[cfg(test)]
mod tests {
    use hermes_events_protocol::{envelope_validation::decode_envelope_v1, v1::{ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, SourceRefV1}};
    use hermes_gateway_protocol::v1::{GetRecoveryStatusRequestV1, RecoveryControlRequestV1};
    use hermes_kernel_control_store::{ControlStore, ExternalRuntimeAttestation, InitialOwnerIdentity, ModuleRegistration, ModuleRegistrationState, SettingsApplyState, SettingsDesiredSnapshot, SettingsSchemaBinding, StoreHealth};
    use hermes_kernel_control_store_sqlite::SqliteControlStore;
    use hermes_runtime_protocol::v1::KernelStateV1;
    use hermes_runtime_protocol::{descriptor_validation::{decode_descriptor_v1, decode_settings_schema_v1, decode_settings_snapshot_v1, validate_initial_owner_enrollment, validate_settings_snapshot_against_schema_v1}, v1::{CapabilityCriticalityV1, CapabilityDescriptorV1, InitialOwnerEnrollmentChallengeV1, InitialOwnerEnrollmentV1, ModuleDescriptorV1, ModuleKindV1, SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1, SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1, SettingsValueEntryV1, SettingValueV1}};
    use prost::Message;

    #[test]
    fn a_new_store_exposes_a_trustworthy_health_snapshot() {
        let store = ControlStore::new("instance-1", 1);

        assert_eq!(store.health(), StoreHealth::Trustworthy);
        assert_eq!(store.instance_id(), "instance-1");
        assert_eq!(store.generation(), 1);
    }

    #[test]
    fn creates_and_reopens_a_trustworthy_control_store() {
        let path = std::env::temp_dir().join(format!(
            "hermes-control-store-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);

        let created = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
        assert_eq!(created.snapshot().health(), StoreHealth::Trustworthy);

        let reopened = SqliteControlStore::open(&path).expect("open store");
        assert_eq!(reopened.snapshot().instance_id(), "instance-1");
        assert_eq!(reopened.snapshot().generation(), 1);

        std::fs::remove_file(path).expect("remove temporary store");
    }

    #[test]
    fn recovery_fences_advance_monotonically() {
        let path = std::env::temp_dir().join(format!(
            "hermes-control-store-fences-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");

        let advanced = store.advance_recovery_fences().expect("advance fences");
        assert_eq!(advanced.generation(), 2);
        assert_eq!(advanced.identity_epoch(), 2);
        assert_eq!(advanced.grant_epoch(), 2);

        let reopened = SqliteControlStore::open(&path).expect("open store");
        assert_eq!(reopened.snapshot().generation(), 2);
        assert_eq!(reopened.snapshot().identity_epoch(), 2);
        assert_eq!(reopened.snapshot().grant_epoch(), 2);
        std::fs::remove_file(path).expect("remove temporary store");
    }

    #[test]
    fn initial_owner_claim_is_atomic_and_keeps_only_the_public_key() {
        let path = std::env::temp_dir().join(format!("hermes-control-store-owner-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
        let public_key = [
            0x04, 0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6,
            0xe5, 0x63, 0xa4, 0x40, 0xf2, 0x77, 0x03, 0x7d, 0x81, 0x2d, 0xeb, 0x33,
            0xa0, 0xf4, 0xa1, 0x39, 0x45, 0xd8, 0x98, 0xc2, 0x96, 0x4f, 0xe3, 0x42,
            0xe2, 0xfe, 0x1a, 0x7f, 0x9b, 0x8e, 0xe7, 0xeb, 0x4a, 0x7c, 0x0f, 0x9e,
            0x16, 0x2b, 0xce, 0x33, 0x57, 0x6b, 0x31, 0x5e, 0xce, 0xcb, 0xb6, 0x40,
            0x68, 0x37, 0xbf, 0x51, 0xf5,
        ];
        let first = InitialOwnerIdentity::new("owner-1", "device-1", public_key);
        store.claim_initial_owner(&first).expect("claim first owner");
        assert_eq!(store.initial_owner_identity().expect("read owner"), Some(first.clone()));
        assert!(store.claim_initial_owner(&InitialOwnerIdentity::new("owner-2", "device-2", public_key)).is_err());
        std::fs::remove_file(path).expect("remove temporary store");
    }

    #[test]
    fn module_registration_has_explicit_state_transitions_and_grant_epoch_fencing() {
        let path = std::env::temp_dir().join(format!("hermes-control-store-registration-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
        let registration = ModuleRegistration::new("registration-1", "module-1", "owner-1", [7; 32], ModuleRegistrationState::Pending, 1);
        store.create_pending_registration(&registration, &["capability.read".to_owned(), "capability.write".to_owned()]).expect("create pending registration");
        assert_eq!(store.module_registration("registration-1").expect("read registration"), Some(registration));
        let approved_grants = store.approve_module_registration("registration-1", &["capability.read".to_owned()]).expect("approve capability subset");
        assert_eq!(approved_grants.grant_epoch(), 2);
        assert_eq!(approved_grants.capability_ids(), ["capability.read"]);
        let approved = store.module_registration("registration-1").expect("read approved registration").expect("registration exists");
        assert_eq!(approved.grant_epoch(), 2);
        assert_eq!(store.effective_grant_set("registration-1").expect("effective grants").expect("approved grants").capability_ids(), ["capability.read"]);
        let attestation = ExternalRuntimeAttestation::new("registration-1", "compose-runtime-1", 1, 2, [9; 32]);
        store.attest_external_runtime(&attestation).expect("attest external runtime");
        assert_eq!(store.effective_external_runtime_attestation("registration-1").expect("effective external runtime"), Some(attestation.clone()));
        assert!(store.attest_external_runtime(&attestation).is_err());
        let schema = SettingsSchemaBinding::new("registration-1", 1, 1, [8; 32], 0, 0, SettingsApplyState::Current, None);
        store.register_settings_schema(&schema).expect("bind settings schema");
        assert_eq!(store.settings_schema_binding("registration-1").expect("read schema binding"), Some(schema));
        let revision = store.commit_desired_settings_snapshot(&SettingsDesiredSnapshot { registration_id: "registration-1".into(), expected_revision: 0, snapshot_bytes: vec![1, 2, 3] }).expect("commit desired settings");
        assert_eq!(revision, 1);
        assert_eq!(store.desired_settings_snapshot("registration-1").expect("read desired snapshot"), Some((1, vec![1, 2, 3])));
        assert_eq!(store.settings_schema_binding("registration-1").expect("read updated binding").expect("binding").desired_revision(), 1);
        assert_eq!(store.settings_schema_binding("registration-1").expect("read pending binding").expect("binding").apply_state(), SettingsApplyState::PendingValidation);
        assert!(store.transition_settings_apply_state("registration-1", 1, SettingsApplyState::Applying, None).is_err());
        store.transition_settings_apply_state("registration-1", 1, SettingsApplyState::PendingApply, None).expect("validation accepted");
        store.transition_settings_apply_state("registration-1", 1, SettingsApplyState::Applying, None).expect("start apply");
        store.confirm_effective_settings_revision("registration-1", 1).expect("confirm applied settings");
        let current_settings = store.settings_schema_binding("registration-1").expect("read effective binding").expect("binding");
        assert_eq!(current_settings.effective_revision(), 1);
        assert_eq!(current_settings.apply_state(), SettingsApplyState::Current);
        let blocked_revision = store.commit_desired_settings_snapshot(&SettingsDesiredSnapshot { registration_id: "registration-1".into(), expected_revision: 1, snapshot_bytes: vec![4] }).expect("commit correction");
        assert!(store.transition_settings_apply_state("registration-1", blocked_revision, SettingsApplyState::BlockedConfig, Some("invalid reason with spaces")).is_err());
        store.transition_settings_apply_state("registration-1", blocked_revision, SettingsApplyState::BlockedConfig, Some("validation.invalid_interval")).expect("record sanitized validation failure");
        let blocked_settings = store.settings_schema_binding("registration-1").expect("read blocked settings").expect("binding");
        assert_eq!(blocked_settings.apply_state(), SettingsApplyState::BlockedConfig);
        assert_eq!(blocked_settings.sanitized_reason_code(), Some("validation.invalid_interval"));
        assert!(store.confirm_effective_settings_revision("registration-1", blocked_revision).is_err());
        assert!(store.commit_desired_settings_snapshot(&SettingsDesiredSnapshot { registration_id: "registration-1".into(), expected_revision: 0, snapshot_bytes: vec![4] }).is_err());
        assert!(store.approve_module_registration("registration-1", &["capability.unrequested".to_owned()]).is_err());
        let suspended = store.transition_module_registration("registration-1", ModuleRegistrationState::Suspended).expect("suspend registration");
        assert_eq!(suspended.grant_epoch(), 3);
        let reapproved = store.approve_module_registration("registration-1", &["capability.write".to_owned()]).expect("replace approved capability subset");
        assert_eq!(reapproved.grant_epoch(), 4);
        assert_eq!(reapproved.capability_ids(), ["capability.write"]);
        assert!(store.effective_external_runtime_attestation("registration-1").expect("stale external runtime").is_none());
        let suspended = store.transition_module_registration("registration-1", ModuleRegistrationState::Suspended).expect("suspend reapproved registration");
        assert_eq!(suspended.grant_epoch(), 5);
        assert!(store.transition_module_registration("registration-1", ModuleRegistrationState::Pending).is_err());
        let revoked = store.transition_module_registration("registration-1", ModuleRegistrationState::Revoked).expect("revoke registration");
        assert_eq!(revoked.grant_epoch(), 6);
        assert!(store.transition_module_registration("registration-1", ModuleRegistrationState::Approved).is_err());
        std::fs::remove_file(path).expect("remove temporary store");
    }


    #[test]
    fn exports_a_consistent_control_store_with_a_checksum() {
        let directory = std::env::temp_dir().join(format!(
            "hermes-control-store-export-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir(&directory).expect("create temporary directory");
        let source = directory.join("source.sqlite");
        let destination = directory.join("export.sqlite");

        let store = SqliteControlStore::create(&source, "instance-1", 1).expect("create store");
        let export = store.export_to(&destination).expect("export store");

        assert_eq!(export.instance_id(), "instance-1");
        assert_eq!(export.generation(), 1);
        assert_eq!(export.sha256_hex().len(), 64);
        assert!(SqliteControlStore::open(&destination).is_ok());

        std::fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn v1_envelope_preserves_the_major_version_field_number() {
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            ..Default::default()
        };

        assert_eq!(envelope.encode_to_vec(), vec![0x08, 0x01]);
    }

    #[test]
    fn envelope_validation_requires_a_complete_typed_header_and_kind() {
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1, envelope_revision: 1, message_id: vec![1; 16], correlation_id: vec![1; 16],
            contract: Some(ContractRefV1 { owner: "communications".into(), name: "message".into(), major: 1, revision: 1, schema_sha256: vec![2; 32] }),
            source: Some(SourceRefV1 { module_id: "mail".into(), runtime_instance_id: vec![3; 16], runtime_generation: 1 }),
            actor: Some(ActorRefV1 { kind: ActorKindV1::System as i32, actor_id: vec![4] }),
            recorded_at: Some(prost_types::Timestamp { seconds: 1, nanos: 0 }),
            semantics: Some(hermes_events_protocol::v1::durable_envelope_v1::Semantics::Event(EventMetadataV1 { occurred_at: None })), ..Default::default()
        };
        assert!(decode_envelope_v1(&envelope.encode_to_vec()).is_ok());
        let mut missing_kind = envelope; missing_kind.semantics = None;
        assert!(decode_envelope_v1(&missing_kind.encode_to_vec()).is_err());
    }

    #[test]
    fn recovery_only_has_a_stable_runtime_state_discriminant() {
        assert_eq!(KernelStateV1::RecoveryOnly as i32, 3);
    }

    #[test]
    fn recovery_status_request_is_an_empty_typed_message() {
        assert!(
            GetRecoveryStatusRequestV1::default()
                .encode_to_vec()
                .is_empty()
        );
    }

    #[test]
    fn recovery_control_envelope_preserves_the_status_operation_variant() {
        let request = RecoveryControlRequestV1 {
            operation: Some(
                hermes_gateway_protocol::v1::recovery_control_request_v1::Operation::GetRecoveryStatus(
                    GetRecoveryStatusRequestV1 {},
                ),
            ),
        };

        let decoded = RecoveryControlRequestV1::decode(request.encode_to_vec().as_slice())
            .expect("decode recovery request");
        assert!(matches!(
            decoded.operation,
            Some(hermes_gateway_protocol::v1::recovery_control_request_v1::Operation::GetRecoveryStatus(_))
        ));
    }

    #[test]
    fn descriptor_validation_rejects_unsorted_or_untyped_capabilities() {
        let mut descriptor = ModuleDescriptorV1 { descriptor_major: 1, descriptor_revision: 1, module_id: "mail".into(), owner_id: "communications".into(), module_kind: ModuleKindV1::Integration as i32, module_version: "1".into(), build_id: "build".into(), ..Default::default() };
        descriptor.capabilities = vec![
            CapabilityDescriptorV1 { capability_id: "z".into(), capability_revision: 1, criticality: CapabilityCriticalityV1::Required as i32, ..Default::default() },
            CapabilityDescriptorV1 { capability_id: "a".into(), capability_revision: 1, criticality: CapabilityCriticalityV1::Required as i32, ..Default::default() },
        ];
        assert!(decode_descriptor_v1(&descriptor.encode_to_vec()).is_err());
    }

    #[test]
    fn descriptor_validation_accepts_a_canonical_typed_descriptor() {
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
        let encoded = descriptor.encode_to_vec();
        assert_eq!(decode_descriptor_v1(&encoded).expect("canonical descriptor"), descriptor);
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
        let snapshot = SettingsSnapshotV1 { target_id: "registration-1".into(), revision: 1, values: vec![SettingsValueEntryV1 { setting_id: "sync.interval".into(), value: Some(SettingValueV1 { value: Some(hermes_runtime_protocol::v1::setting_value_v1::Value::DurationMillis(1000)) }) }] };
        assert!(decode_settings_snapshot_v1(&snapshot.encode_to_vec()).is_ok());
        let schema = SettingsSchemaV1 { major: 1, revision: 1, definitions: vec![SettingDefinitionV1 { setting_id: "sync.interval".into(), capability_id: String::new(), value_type: SettingValueTypeV1::Duration as i32, mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32, target_scope: SettingTargetScopeV1::ModuleRegistration as i32, apply_mode: SettingApplyModeV1::HotReload as i32, client_visibility: SettingClientVisibilityV1::Editable as i32, fresh_owner_proof_required: false, kernel_controller_id: String::new(), display_name: "Sync interval".into() }] };
        assert!(validate_settings_snapshot_against_schema_v1(&schema, &snapshot).is_ok());
        let mut wrong_type = snapshot.clone();
        wrong_type.values[0].value = Some(SettingValueV1 { value: Some(hermes_runtime_protocol::v1::setting_value_v1::Value::StringValue("not a duration".into())) });
        assert!(validate_settings_snapshot_against_schema_v1(&schema, &wrong_type).is_err());
        let mut unknown_setting = snapshot.clone();
        unknown_setting.values[0].setting_id = "unknown.setting".into();
        assert!(validate_settings_snapshot_against_schema_v1(&schema, &unknown_setting).is_err());
        let mut oversized = snapshot.clone();
        oversized.values[0].value = Some(SettingValueV1 { value: Some(hermes_runtime_protocol::v1::setting_value_v1::Value::StringValue("x".repeat(8193))) });
        let string_schema = SettingsSchemaV1 { definitions: vec![SettingDefinitionV1 { value_type: SettingValueTypeV1::String as i32, ..schema.definitions[0].clone() }], ..schema.clone() };
        assert!(validate_settings_snapshot_against_schema_v1(&string_schema, &oversized).is_err());
        let mut missing = snapshot;
        missing.values[0].value = None;
        assert!(decode_settings_snapshot_v1(&missing.encode_to_vec()).is_err());
    }

    #[test]
    fn initial_enrollment_contract_is_fixed_to_p256_and_a_single_pristine_generation() {
        let challenge = InitialOwnerEnrollmentChallengeV1 { protocol_major: 1, instance_id: vec![1; 32], nonce: vec![2; 32], kernel_generation: 1 };
        let proof = InitialOwnerEnrollmentV1 { protocol_major: 1, device_public_key_sec1: [vec![0x04], vec![3; 64]].concat(), challenge_signature_raw: vec![4; 64], owner_id: "owner-1".to_owned(), device_id: "device-1".to_owned() };
        assert!(validate_initial_owner_enrollment(&challenge, &proof));
        assert!(!validate_initial_owner_enrollment(&InitialOwnerEnrollmentChallengeV1 { kernel_generation: 2, ..challenge }, &proof));
    }
}
