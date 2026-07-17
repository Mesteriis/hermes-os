use super::common::*;

#[test]
fn distribution_bundle_verifier_requires_the_signed_target_and_exact_artifact_bytes() {
    let fixture = DistributionBundleFixture::new();
    let verified = fixture.verify();
    fixture.assert_verified_contract(&verified);
    fixture.assert_launch_binding(&verified);
    fixture.assert_registration_identity_rejected(&verified);
    fixture.assert_path_and_target_rejected();
    fixture.assert_replaced_bytes_rejected();
    fixture.assert_descriptor_schema_mismatch_rejected();
}

#[test]
fn telemetry_binding_requires_an_exact_signed_platform_descriptor() {
    let fixture = DistributionBundleFixture::telemetry();
    let verified = fixture.verify();
    let store = SqliteControlStore::create(&fixture.root.join("control.sqlite"), "instance-1", 1)
        .expect("create control store");
    let binding = telemetry_binding::admit(&store, &verified).expect("admit telemetry release");
    assert_eq!(binding.process_id(), "telemetry");
    assert_eq!(binding.binding_revision(), 1);
    assert_eq!(binding.artifact_id(), "platform.telemetry");
    assert_eq!(binding.executable_sha256(), &fixture.artifact_digest);
    assert_eq!(binding.descriptor_sha256(), &fixture.descriptor_digest);

    let wrong_owner = DistributionBundleFixture::telemetry_for_owner("other-owner");
    let wrong_owner_bundle = wrong_owner.verify();
    let wrong_owner_store =
        SqliteControlStore::create(&wrong_owner.root.join("control.sqlite"), "instance-2", 1)
            .expect("create wrong-owner store");
    assert_eq!(
        telemetry_binding::admit(&wrong_owner_store, &wrong_owner_bundle)
            .expect_err("reject wrong telemetry owner"),
        "signed release must contain exactly one designated Telemetry artifact"
    );
}

struct DistributionBundleFixture {
    root: std::path::PathBuf,
    artifact_path: std::path::PathBuf,
    descriptor_path: std::path::PathBuf,
    settings_schema_path: std::path::PathBuf,
    artifact_digest: [u8; 32],
    descriptor_digest: [u8; 32],
    settings_schema_digest: [u8; 32],
    descriptor: ModuleDescriptorV1,
    settings_schema: SettingsSchemaV1,
    settings_schema_bytes: Vec<u8>,
    manifest: DistributionManifestV1,
    signed: SignedDistributionManifestV1,
    signing_key: SigningKey,
    trust_root: ReleaseTrustRoot,
}

impl DistributionBundleFixture {
    fn new() -> Self {
        Self::new_for(BundleIdentity::mail())
    }

    fn telemetry() -> Self {
        Self::new_for(BundleIdentity::telemetry("telemetry"))
    }

    fn telemetry_for_owner(owner_id: &'static str) -> Self {
        Self::new_for(BundleIdentity::telemetry(owner_id))
    }

    fn new_for(identity: BundleIdentity) -> Self {
        let root = unique_target_root("hermes-distribution-bundle");
        let artifact_path = root.join(identity.artifact_relative_path);
        let descriptor_path = root.join(identity.descriptor_relative_path);
        let settings_schema_path = root.join(identity.settings_schema_relative_path);
        std::fs::create_dir_all(artifact_path.parent().expect("artifact parent"))
            .expect("create artifact directory");
        std::fs::create_dir_all(descriptor_path.parent().expect("descriptor parent"))
            .expect("create descriptor directory");
        std::fs::write(&artifact_path, Self::artifact_bytes()).expect("write artifact");
        let artifact_digest = Sha256::digest(Self::artifact_bytes()).into();
        let settings_schema = SettingsSchemaV1 {
            major: 1,
            revision: 1,
            ..Default::default()
        };
        let settings_schema_bytes = settings_schema.encode_to_vec();
        std::fs::write(&settings_schema_path, &settings_schema_bytes)
            .expect("write settings schema");
        let settings_schema_digest = Sha256::digest(&settings_schema_bytes).into();
        let descriptor = bundle_descriptor(
            &settings_schema,
            settings_schema_digest,
            settings_schema_bytes.len(),
            identity,
        );
        let descriptor_bytes = descriptor.encode_to_vec();
        std::fs::write(&descriptor_path, &descriptor_bytes).expect("write descriptor");
        let descriptor_digest = Sha256::digest(&descriptor_bytes).into();
        let manifest = bundle_manifest(
            artifact_digest,
            descriptor_digest,
            settings_schema_digest,
            descriptor_bytes.len(),
            settings_schema_bytes.len(),
            identity,
        );
        let signing_key = SigningKey::from_bytes((&[11_u8; 32]).into()).expect("test key");
        let (signed, trust_root) = sign_bundle_manifest(&manifest, &signing_key);
        Self {
            root,
            artifact_path,
            descriptor_path,
            settings_schema_path,
            artifact_digest,
            descriptor_digest,
            settings_schema_digest,
            descriptor,
            settings_schema,
            settings_schema_bytes,
            manifest,
            signed,
            signing_key,
            trust_root,
        }
    }

    fn verify(&self) -> distribution_bundle_verifier::VerifiedDistributionBundle {
        distribution_bundle_verifier::verify(
            &self.root,
            &self.signed.encode_to_vec(),
            &self.trust_root,
            "aarch64-apple-darwin",
        )
        .expect("verified distribution bundle")
    }

    fn assert_verified_contract(
        &self,
        verified: &distribution_bundle_verifier::VerifiedDistributionBundle,
    ) {
        assert_eq!(verified.manifest(), &self.manifest);
        assert_eq!(verified.artifacts().len(), 1);
        let artifact = &verified.artifacts()[0];
        assert_eq!(artifact.artifact_id(), "runtime.mail");
        assert_eq!(artifact.canonical_path(), self.artifact_path);
        assert_eq!(artifact.expected_sha256(), &self.artifact_digest);
        assert_eq!(
            artifact
                .module_descriptor()
                .expect("verified descriptor")
                .module_id,
            "mail"
        );
        assert_eq!(
            artifact
                .settings_schema()
                .expect("verified settings schema"),
            &self.settings_schema
        );
    }

    fn assert_launch_binding(
        &self,
        verified: &distribution_bundle_verifier::VerifiedDistributionBundle,
    ) {
        let store_path = self.root.join("control.sqlite");
        let store =
            SqliteControlStore::create(&store_path, "instance-1", 1).expect("create control store");
        register_approved_module(
            &store,
            "registration-mail",
            "mail",
            "communications",
            self.descriptor_digest,
        );
        let binding = bundled_managed_launch_binding::admit(
            &store,
            "registration-mail",
            verified,
            "runtime.mail",
        )
        .expect("admit signed release binding");
        assert_eq!(binding.binding_revision(), 1);
        assert_eq!(binding.executable_sha256(), &self.artifact_digest);
        assert_eq!(binding.descriptor_sha256(), &self.descriptor_digest);
        assert_eq!(
            binding.settings_schema_sha256(),
            Some(&self.settings_schema_digest)
        );
    }

    fn assert_registration_identity_rejected(
        &self,
        verified: &distribution_bundle_verifier::VerifiedDistributionBundle,
    ) {
        let store = SqliteControlStore::open(&self.root.join("control.sqlite"))
            .expect("open control store");
        register_approved_module(
            &store,
            "registration-other",
            "calendar",
            "calendar",
            self.descriptor_digest,
        );
        assert_eq!(
            bundled_managed_launch_binding::admit(
                &store,
                "registration-other",
                verified,
                "runtime.mail",
            )
            .expect_err("reject mismatched module identity"),
            "managed launch artifact does not match its approved registration"
        );
    }

    fn assert_path_and_target_rejected(&self) {
        let linked_root = self.root.with_extension("link");
        std::os::unix::fs::symlink(&self.root, &linked_root).expect("create linked root");
        assert_eq!(
            distribution_bundle_verifier::verify(
                &linked_root,
                &self.signed.encode_to_vec(),
                &self.trust_root,
                "aarch64-apple-darwin",
            )
            .err()
            .expect("linked root"),
            "distribution bundle root must not traverse a symlink"
        );
        std::fs::remove_file(linked_root).expect("remove linked root");
        assert_eq!(
            distribution_bundle_verifier::verify(
                &self.root,
                &self.signed.encode_to_vec(),
                &self.trust_root,
                "x86_64-unknown-linux-gnu",
            )
            .err()
            .expect("wrong target"),
            "distribution manifest target triple does not match this Kernel"
        );
    }

    fn assert_replaced_bytes_rejected(&self) {
        std::fs::write(&self.settings_schema_path, b"replaced").expect("replace settings schema");
        assert_eq!(
            self.verify_error(),
            "distribution settings schema size does not match manifest"
        );
        std::fs::write(&self.settings_schema_path, &self.settings_schema_bytes)
            .expect("restore settings schema");
        std::fs::write(&self.artifact_path, b"replaced bytes").expect("replace artifact");
        assert_eq!(
            self.verify_error(),
            "distribution artifact size does not match manifest"
        );
        std::fs::write(&self.artifact_path, Self::artifact_bytes()).expect("restore artifact");
    }

    fn assert_descriptor_schema_mismatch_rejected(&self) {
        let mut descriptor = self.descriptor.clone();
        descriptor
            .settings_schema_ref
            .as_mut()
            .expect("settings schema reference")
            .revision = 2;
        let descriptor_bytes = descriptor.encode_to_vec();
        std::fs::write(&self.descriptor_path, &descriptor_bytes)
            .expect("write mismatched descriptor");
        let mut manifest = self.manifest.clone();
        manifest.artifacts[0].descriptor_sha256 = Sha256::digest(&descriptor_bytes).to_vec();
        manifest.artifacts[0].descriptor_size_bytes =
            u64::try_from(descriptor_bytes.len()).expect("descriptor length");
        let (signed, _) = sign_bundle_manifest(&manifest, &self.signing_key);
        assert_eq!(
            distribution_bundle_verifier::verify(
                &self.root,
                &signed.encode_to_vec(),
                &self.trust_root,
                "aarch64-apple-darwin",
            )
            .err()
            .expect("reject descriptor/schema mismatch"),
            "distribution settings schema does not match module descriptor"
        );
    }

    fn verify_error(&self) -> String {
        distribution_bundle_verifier::verify(
            &self.root,
            &self.signed.encode_to_vec(),
            &self.trust_root,
            "aarch64-apple-darwin",
        )
        .err()
        .expect("bundle verification must fail")
    }

    fn artifact_bytes() -> &'static [u8] {
        b"Hermes signed module runtime"
    }
}

#[derive(Clone, Copy)]
struct BundleIdentity {
    artifact_id: &'static str,
    artifact_relative_path: &'static str,
    descriptor_relative_path: &'static str,
    settings_schema_relative_path: &'static str,
    module_id: &'static str,
    owner_id: &'static str,
    module_kind: ModuleKindV1,
    capability_id: &'static str,
}

impl BundleIdentity {
    const fn mail() -> Self {
        Self {
            artifact_id: "runtime.mail",
            artifact_relative_path: "bin/module-runtime",
            descriptor_relative_path: "contracts/mail-descriptor.pb",
            settings_schema_relative_path: "contracts/mail-settings.pb",
            module_id: "mail",
            owner_id: "communications",
            module_kind: ModuleKindV1::Integration,
            capability_id: "read",
        }
    }

    const fn telemetry(owner_id: &'static str) -> Self {
        Self {
            artifact_id: "platform.telemetry",
            artifact_relative_path: "bin/telemetry-collector",
            descriptor_relative_path: "contracts/telemetry-descriptor.pb",
            settings_schema_relative_path: "contracts/telemetry-settings.pb",
            module_id: "telemetry",
            owner_id,
            module_kind: ModuleKindV1::Platform,
            capability_id: "collect",
        }
    }
}

impl Drop for DistributionBundleFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn bundle_descriptor(
    schema: &SettingsSchemaV1,
    settings_schema_digest: [u8; 32],
    settings_schema_size: usize,
    identity: BundleIdentity,
) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: identity.module_id.to_owned(),
        owner_id: identity.owner_id.to_owned(),
        module_kind: identity.module_kind as i32,
        module_version: "1".to_owned(),
        build_id: "build-1".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: identity.capability_id.to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            ..Default::default()
        }],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: schema.major,
            revision: schema.revision,
            artifact_size_bytes: settings_schema_size as u64,
            sha256: settings_schema_digest.to_vec(),
        }),
        ..Default::default()
    }
}

fn bundle_manifest(
    artifact_digest: [u8; 32],
    descriptor_digest: [u8; 32],
    settings_schema_digest: [u8; 32],
    descriptor_size: usize,
    settings_schema_size: usize,
    identity: BundleIdentity,
) -> DistributionManifestV1 {
    DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "hermes-desktop".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "build-1".to_owned(),
        target_triple: "aarch64-apple-darwin".to_owned(),
        generation: 1,
        artifacts: vec![DistributionManifestArtifactV1 {
            artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
            artifact_id: identity.artifact_id.to_owned(),
            relative_path: identity.artifact_relative_path.to_owned(),
            size_bytes: DistributionBundleFixture::artifact_bytes().len() as u64,
            sha256: artifact_digest.to_vec(),
            descriptor_sha256: descriptor_digest.to_vec(),
            settings_schema_sha256: settings_schema_digest.to_vec(),
            required: true,
            descriptor_relative_path: identity.descriptor_relative_path.to_owned(),
            descriptor_size_bytes: descriptor_size as u64,
            settings_schema_relative_path: identity.settings_schema_relative_path.to_owned(),
            settings_schema_size_bytes: settings_schema_size as u64,
        }],
    }
}

fn sign_bundle_manifest(
    manifest: &DistributionManifestV1,
    signing_key: &SigningKey,
) -> (SignedDistributionManifestV1, ReleaseTrustRoot) {
    let raw_manifest_bytes = manifest.encode_to_vec();
    let signature: Signature = signing_key.sign(&raw_manifest_bytes);
    let signed = SignedDistributionManifestV1 {
        verification_key_id: "release-2026".to_owned(),
        raw_manifest_bytes,
        signature_raw: signature.to_bytes().to_vec(),
    };
    let public_key_sec1: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed P-256 key");
    (
        signed,
        release_trust_root(&[("release-2026", public_key_sec1)]),
    )
}

fn register_approved_module(
    store: &SqliteControlStore,
    registration_id: &str,
    module_id: &str,
    owner_id: &str,
    descriptor_digest: [u8; 32],
) {
    let registration = ModuleRegistration::new(
        registration_id,
        module_id,
        owner_id,
        descriptor_digest,
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &["read".to_owned()])
        .expect("create registration");
    store
        .approve_module_registration(registration_id, &["read".to_owned()])
        .expect("approve registration");
}
