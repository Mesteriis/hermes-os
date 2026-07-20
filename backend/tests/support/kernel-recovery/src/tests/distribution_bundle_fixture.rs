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
fn signed_browser_bootstrap_artifact_is_read_only_after_manifest_verification() {
    let root = unique_target_root("hermes-browser-bootstrap-bundle");
    let bootstrap_path = root.join("browser/bootstrap.html");
    std::fs::create_dir_all(bootstrap_path.parent().expect("bootstrap parent"))
        .expect("create bootstrap directory");
    let bootstrap = b"<!doctype html><title>Hermes</title>".to_vec();
    std::fs::write(&bootstrap_path, &bootstrap).expect("write bootstrap");
    let signing_key = SigningKey::from_bytes((&[23_u8; 32]).into()).expect("test signing key");
    let manifest = DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "hermes-desktop".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "browser-bootstrap".to_owned(),
        target_triple: "aarch64-apple-darwin".to_owned(),
        generation: 1,
        artifacts: vec![DistributionManifestArtifactV1 {
            artifact_kind: DistributionArtifactKindV1::BrowserBootstrapBundle as i32,
            artifact_id: "browser.bootstrap".to_owned(),
            relative_path: "browser/bootstrap.html".to_owned(),
            size_bytes: bootstrap.len() as u64,
            sha256: Sha256::digest(&bootstrap).to_vec(),
            required: true,
            ..Default::default()
        }],
    };
    let (signed, trust_root) = sign_bundle_manifest(&manifest, &signing_key);
    let verified = distribution_bundle_verifier::verify(
        &root,
        &signed.encode_to_vec(),
        &trust_root,
        "aarch64-apple-darwin",
    )
    .expect("verify signed browser bootstrap");
    let artifact = verified
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == "browser.bootstrap")
        .expect("verified bootstrap artifact");
    assert_eq!(
        artifact
            .read_verified_bytes()
            .expect("re-read guarded bytes"),
        bootstrap
    );
    std::fs::write(&bootstrap_path, b"tampered").expect("tamper bootstrap");
    assert!(artifact.read_verified_bytes().is_err());
    std::fs::remove_dir_all(root).expect("remove fixture directory");
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

#[test]
fn storage_binding_requires_an_exact_signed_platform_descriptor() {
    let fixture = DistributionBundleFixture::storage("storage");
    let verified = fixture.verify();
    let store = SqliteControlStore::create(&fixture.root.join("control.sqlite"), "instance-1", 1)
        .expect("create control store");
    let binding = storage_binding::admit(&store, &verified).expect("admit storage release");
    assert_eq!(binding.process_id(), "storage");
    assert_eq!(binding.artifact_id(), "platform.storage");

    let wrong_owner = DistributionBundleFixture::storage("other-owner");
    let wrong_owner_bundle = wrong_owner.verify();
    assert_eq!(
        storage_binding::admit(&store, &wrong_owner_bundle)
            .expect_err("reject wrong storage owner"),
        "signed release must contain exactly one designated Storage artifact"
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

    fn storage(owner_id: &'static str) -> Self {
        Self::new_for(BundleIdentity::storage(owner_id))
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

    const fn storage(owner_id: &'static str) -> Self {
        Self {
            artifact_id: "platform.storage",
            artifact_relative_path: "bin/storage-control",
            descriptor_relative_path: "contracts/storage-descriptor.pb",
            settings_schema_relative_path: "contracts/storage-settings.pb",
            module_id: "storage",
            owner_id,
            module_kind: ModuleKindV1::Platform,
            capability_id: "control",
        }
    }
}

impl Drop for DistributionBundleFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

mod builders;

use builders::{
    bundle_descriptor, bundle_manifest, register_approved_module, sign_bundle_manifest,
};
