//! Builds an isolated signed macOS release bundle for managed-process conformance.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use hermes_runtime_protocol::v1::{
    DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
    ModuleDescriptorV1, ReleaseTrustRootKeyV1, ReleaseTrustRootV1, SignedDistributionManifestV1,
};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use prost::Message;
use sha2::{Digest, Sha256};

const TARGET_TRIPLE: &str = "aarch64-apple-darwin";
const SIGNING_KEY_ID: &str = "managed-runtime-test-key";

pub(crate) struct SignedRuntimeArtifact {
    artifact_id: &'static str,
    binary: PathBuf,
    descriptor: Vec<u8>,
    settings_schema: Option<Vec<u8>>,
}

impl SignedRuntimeArtifact {
    pub(crate) fn new(artifact_id: &'static str, binary: PathBuf, descriptor: Vec<u8>) -> Self {
        Self {
            artifact_id,
            binary,
            descriptor,
            settings_schema: None,
        }
    }

    pub(crate) fn with_settings_schema(mut self, settings_schema: Vec<u8>) -> Self {
        self.settings_schema = Some(settings_schema);
        self
    }
}

pub(crate) struct InstalledSignedBundle {
    kernel: PathBuf,
}

impl InstalledSignedBundle {
    pub(crate) fn install(
        root: &Path,
        artifacts: &[SignedRuntimeArtifact],
    ) -> Result<Self, String> {
        if artifacts.is_empty() {
            return Err("signed release must contain managed artifacts".to_owned());
        }
        let kernel = root.join("Hermes.app/Contents/MacOS/hermes-kernel");
        let resources = root.join("Hermes.app/Contents/Resources/hermes-kernel-release");
        let distribution = resources.join("distribution");
        std::fs::create_dir_all(
            kernel
                .parent()
                .ok_or_else(|| "signed release Kernel path is invalid".to_owned())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&distribution).map_err(|error| error.to_string())?;
        std::fs::write(&kernel, b"test-kernel").map_err(|error| error.to_string())?;
        let manifest = install_artifacts(&distribution, artifacts)?;
        write_release_signature(&resources, &manifest)?;
        Ok(Self { kernel })
    }

    #[must_use]
    pub(crate) fn kernel(&self) -> &Path {
        &self.kernel
    }
}

fn install_artifacts(
    distribution: &Path,
    artifacts: &[SignedRuntimeArtifact],
) -> Result<DistributionManifestV1, String> {
    let mut manifest_artifacts = artifacts
        .iter()
        .map(|artifact| install_artifact(distribution, artifact))
        .collect::<Result<Vec<_>, _>>()?;
    manifest_artifacts.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    Ok(DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "hermes-managed-runtime-conformance".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "managed-runtime-conformance".to_owned(),
        target_triple: TARGET_TRIPLE.to_owned(),
        generation: 1,
        artifacts: manifest_artifacts,
    })
}

fn install_artifact(
    distribution: &Path,
    artifact: &SignedRuntimeArtifact,
) -> Result<DistributionManifestArtifactV1, String> {
    let binary_relative_path = format!("bin/{}", artifact.artifact_id);
    let binary_path = distribution.join(&binary_relative_path);
    std::fs::create_dir_all(
        binary_path
            .parent()
            .ok_or_else(|| "signed release artifact path is invalid".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::copy(&artifact.binary, &binary_path).map_err(|error| error.to_string())?;
    std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let descriptor_relative_path = format!("contracts/{}.descriptor.pb", artifact.artifact_id);
    let descriptor_path = distribution.join(&descriptor_relative_path);
    std::fs::create_dir_all(
        descriptor_path
            .parent()
            .ok_or_else(|| "signed release descriptor path is invalid".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(&descriptor_path, &artifact.descriptor).map_err(|error| error.to_string())?;
    let binary_bytes = std::fs::read(&binary_path).map_err(|error| error.to_string())?;
    let descriptor = ModuleDescriptorV1::decode(artifact.descriptor.as_slice())
        .map_err(|_| "signed release descriptor is invalid".to_owned())?;
    if descriptor.module_id.is_empty() || descriptor.owner_id.is_empty() {
        return Err("signed release descriptor has no platform identity".to_owned());
    }
    let (settings_schema_relative_path, settings_schema_size_bytes, settings_schema_sha256) =
        install_settings_schema(distribution, artifact)?;
    Ok(DistributionManifestArtifactV1 {
        artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
        artifact_id: artifact.artifact_id.to_owned(),
        relative_path: binary_relative_path,
        size_bytes: binary_bytes.len() as u64,
        sha256: Sha256::digest(&binary_bytes).to_vec(),
        descriptor_sha256: Sha256::digest(&artifact.descriptor).to_vec(),
        settings_schema_sha256,
        required: true,
        descriptor_relative_path,
        descriptor_size_bytes: artifact.descriptor.len() as u64,
        settings_schema_relative_path,
        settings_schema_size_bytes,
    })
}

fn install_settings_schema(
    distribution: &Path,
    artifact: &SignedRuntimeArtifact,
) -> Result<(String, u64, Vec<u8>), String> {
    let Some(schema) = &artifact.settings_schema else {
        return Ok((String::new(), 0, Vec::new()));
    };
    let relative_path = format!("contracts/{}.settings.pb", artifact.artifact_id);
    let path = distribution.join(&relative_path);
    std::fs::write(&path, schema).map_err(|error| error.to_string())?;
    Ok((
        relative_path,
        schema.len() as u64,
        Sha256::digest(schema).to_vec(),
    ))
}

fn write_release_signature(
    resources: &Path,
    manifest: &DistributionManifestV1,
) -> Result<(), String> {
    let signing_key = SigningKey::from_bytes((&[43_u8; 32]).into())
        .map_err(|_| "test signing key is invalid".to_owned())?;
    let raw_manifest_bytes = manifest.encode_to_vec();
    let signature: Signature = signing_key.sign(&raw_manifest_bytes);
    let signed = SignedDistributionManifestV1 {
        verification_key_id: SIGNING_KEY_ID.to_owned(),
        raw_manifest_bytes,
        signature_raw: signature.to_bytes().to_vec(),
    };
    let trust_root = ReleaseTrustRootV1 {
        major: 1,
        revision: 1,
        verification_keys: vec![ReleaseTrustRootKeyV1 {
            key_id: SIGNING_KEY_ID.to_owned(),
            public_key_sec1: signing_key
                .verifying_key()
                .to_sec1_point(false)
                .as_bytes()
                .to_vec(),
        }],
    };
    std::fs::write(
        resources.join("hermes-signed-distribution-manifest.pb"),
        signed.encode_to_vec(),
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        resources.join("hermes-release-trust-root.pb"),
        trust_root.encode_to_vec(),
    )
    .map_err(|error| error.to_string())
}
