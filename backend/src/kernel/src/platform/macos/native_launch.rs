//! Complete macOS native-artifact verification immediately before managed spawn.

use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use hermes_kernel_control_store::{BundledManagedLaunchBinding, PlatformManagedProcessBinding};
use hermes_runtime_protocol::v1::{ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1};

use crate::distribution::bundle_verifier::{self, VerifiedDistributionBundle};
use crate::distribution::runtime_dependencies;
use crate::distribution::staged_artifact::{self, StagedNativeArtifact};
use crate::distribution::trust_root::ReleaseTrustRoot;
use crate::platform::macos::release_resources;

const MAX_SIGNED_MANIFEST_BYTES: u64 = 513 * 1024;

pub struct PreparedPlatformManagedProcess {
    staged_executable: StagedNativeArtifact,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Option<Vec<u8>>,
}

/// Verified bundled module bytes and its descriptor contracts for a managed launch.
pub struct PreparedBundledManagedRuntime {
    staged_executable: StagedNativeArtifact,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Option<Vec<u8>>,
}

pub struct PreparedBundledManagedIntegrationRuntime {
    runtime: PreparedBundledManagedRuntime,
    runtime_artifact_bindings: Vec<ManagedRuntimeArtifactBindingV1>,
    staged_runtime_artifacts: Vec<StagedNativeArtifact>,
    state_layout_revision: Option<u32>,
}

impl PreparedBundledManagedIntegrationRuntime {
    #[must_use]
    pub fn runtime_artifact_bindings(&self) -> &[ManagedRuntimeArtifactBindingV1] {
        &self.runtime_artifact_bindings
    }

    #[must_use]
    pub fn state_layout_revision(&self) -> Option<u32> {
        self.state_layout_revision
    }

    pub fn into_launch_parts(self) -> (PreparedBundledManagedRuntime, Vec<StagedNativeArtifact>) {
        (self.runtime, self.staged_runtime_artifacts)
    }

    pub fn remove(self) {
        let (runtime, artifacts) = self.into_launch_parts();
        let _ = runtime.remove();
        for artifact in artifacts {
            let _ = artifact.remove();
        }
    }
}

impl PreparedBundledManagedRuntime {
    #[must_use]
    pub fn descriptor_bytes(&self) -> &[u8] {
        &self.descriptor_bytes
    }

    #[must_use]
    pub fn settings_schema_bytes(&self) -> Option<&[u8]> {
        self.settings_schema_bytes.as_deref()
    }

    pub fn into_staged_executable(self) -> StagedNativeArtifact {
        self.staged_executable
    }

    pub fn remove(self) -> Result<(), String> {
        self.staged_executable.remove()
    }
}

impl PreparedPlatformManagedProcess {
    #[must_use]
    pub fn descriptor_bytes(&self) -> &[u8] {
        &self.descriptor_bytes
    }

    #[must_use]
    pub fn settings_schema_bytes(&self) -> Option<&[u8]> {
        self.settings_schema_bytes.as_deref()
    }

    pub fn into_staged_executable(self) -> StagedNativeArtifact {
        self.staged_executable
    }

    pub fn remove(self) -> Result<(), String> {
        self.staged_executable.remove()
    }
}

pub fn verify_selected_installed_release(
    kernel_executable: &Path,
    target_triple: &str,
    artifact_id: &str,
    launch_directory: &Path,
) -> Result<StagedNativeArtifact, String> {
    let bundle = verify_selected_installed_bundle(kernel_executable, target_triple)?;
    stage_verified_artifact(&bundle, artifact_id, launch_directory)
}

pub fn stage_bound_installed_release(
    kernel_executable: &Path,
    binding: &BundledManagedLaunchBinding,
    launch_directory: &Path,
) -> Result<StagedNativeArtifact, String> {
    let bundle = verify_selected_installed_bundle(kernel_executable, "aarch64-apple-darwin")?;
    let artifact = bound_artifact(&bundle, binding)?;
    stage_artifact(artifact, launch_directory)
}

pub fn prepare_bound_managed_runtime(
    kernel_executable: &Path,
    binding: &BundledManagedLaunchBinding,
    launch_directory: &Path,
) -> Result<PreparedBundledManagedRuntime, String> {
    let bundle = verify_selected_installed_bundle(kernel_executable, "aarch64-apple-darwin")?;
    let artifact = bound_artifact(&bundle, binding)?;
    let descriptor_bytes = artifact
        .module_descriptor_bytes()
        .ok_or_else(|| "managed launch artifact lacks a module descriptor".to_owned())?
        .to_vec();
    Ok(PreparedBundledManagedRuntime {
        staged_executable: stage_artifact(artifact, launch_directory)?,
        descriptor_bytes,
        settings_schema_bytes: artifact.settings_schema_bytes().map(ToOwned::to_owned),
    })
}

pub fn prepare_bound_managed_integration_runtime(
    kernel_executable: &Path,
    binding: &BundledManagedLaunchBinding,
    launch_directory: &Path,
    granted_capability_ids: &[String],
) -> Result<PreparedBundledManagedIntegrationRuntime, String> {
    let bundle = verify_selected_installed_bundle(kernel_executable, "aarch64-apple-darwin")?;
    let executable = bound_artifact(&bundle, binding)?;
    let descriptor = executable
        .module_descriptor()
        .ok_or_else(|| "managed launch artifact lacks a module descriptor".to_owned())?;
    let requirements =
        runtime_dependencies::select(descriptor, granted_capability_ids, bundle.manifest())?;
    let runtime = PreparedBundledManagedRuntime {
        staged_executable: stage_artifact(executable, launch_directory)?,
        descriptor_bytes: executable
            .module_descriptor_bytes()
            .ok_or_else(|| "managed launch artifact lacks a module descriptor".to_owned())?
            .to_vec(),
        settings_schema_bytes: executable.settings_schema_bytes().map(ToOwned::to_owned),
    };
    let result = stage_runtime_dependencies(
        &bundle,
        requirements.runtime_artifacts(),
        &launch_directory.join("runtime-artifacts"),
    );
    let (runtime_artifact_bindings, staged_runtime_artifacts) = match result {
        Ok(staged) => staged,
        Err(error) => {
            let _ = runtime.remove();
            return Err(error);
        }
    };
    Ok(PreparedBundledManagedIntegrationRuntime {
        runtime,
        runtime_artifact_bindings,
        staged_runtime_artifacts,
        state_layout_revision: requirements.state_layout_revision(),
    })
}

pub fn prepare_bound_platform_process(
    kernel_executable: &Path,
    binding: &PlatformManagedProcessBinding,
    launch_directory: &Path,
) -> Result<PreparedPlatformManagedProcess, String> {
    let bundle = verify_selected_installed_bundle(kernel_executable, "aarch64-apple-darwin")?;
    let artifact = platform_bound_artifact(&bundle, binding)?;
    let descriptor_bytes = artifact
        .module_descriptor_bytes()
        .ok_or_else(|| "platform process release artifact lacks a module descriptor".to_owned())?
        .to_vec();
    Ok(PreparedPlatformManagedProcess {
        staged_executable: stage_artifact(artifact, launch_directory)?,
        descriptor_bytes,
        settings_schema_bytes: artifact.settings_schema_bytes().map(ToOwned::to_owned),
    })
}

pub fn verify_selected_installed_bundle(
    kernel_executable: &Path,
    target_triple: &str,
) -> Result<VerifiedDistributionBundle, String> {
    let resources = release_resources::discover_from_executable(kernel_executable)?;
    let signed_manifest_bytes = read_stable_manifest(resources.signed_manifest_path())?;
    let trust_root = ReleaseTrustRoot::load(resources.trust_root_path())?;
    bundle_verifier::verify(
        resources.distribution_root(),
        &signed_manifest_bytes,
        &trust_root,
        target_triple,
    )
}

pub fn verify_and_stage(
    bundle_root: &Path,
    signed_manifest_bytes: &[u8],
    trust_root_path: &Path,
    target_triple: &str,
    artifact_id: &str,
    launch_directory: &Path,
) -> Result<StagedNativeArtifact, String> {
    let trust_root = ReleaseTrustRoot::load(trust_root_path)?;
    let bundle = bundle_verifier::verify(
        bundle_root,
        signed_manifest_bytes,
        &trust_root,
        target_triple,
    )?;
    stage_verified_artifact(&bundle, artifact_id, launch_directory)
}

fn stage_verified_artifact(
    bundle: &VerifiedDistributionBundle,
    artifact_id: &str,
    launch_directory: &Path,
) -> Result<StagedNativeArtifact, String> {
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    stage_artifact(artifact, launch_directory)
}

fn stage_runtime_dependencies(
    bundle: &VerifiedDistributionBundle,
    requested: &[hermes_runtime_protocol::v1::DistributionManifestArtifactV1],
    launch_directory: &Path,
) -> Result<
    (
        Vec<ManagedRuntimeArtifactBindingV1>,
        Vec<StagedNativeArtifact>,
    ),
    String,
> {
    let mut bindings = Vec::with_capacity(requested.len());
    let mut staged_artifacts = Vec::with_capacity(requested.len());
    for request in requested {
        let artifact = match bundle
            .artifacts()
            .binary_search_by(|candidate| candidate.artifact_id().cmp(&request.artifact_id))
            .ok()
            .map(|index| &bundle.artifacts()[index])
        {
            Some(artifact) => artifact,
            None => {
                cleanup_staged_artifacts(staged_artifacts);
                return Err(
                    "verified managed integration runtime artifact is unavailable".to_owned(),
                );
            }
        };
        if artifact.size_bytes() != request.size_bytes
            || artifact.expected_sha256().as_slice() != request.sha256.as_slice()
        {
            cleanup_staged_artifacts(staged_artifacts);
            return Err(
                "verified managed integration runtime artifact does not match manifest".to_owned(),
            );
        }
        let staged = match stage_artifact(artifact, launch_directory) {
            Ok(staged) => staged,
            Err(error) => {
                cleanup_staged_artifacts(staged_artifacts);
                return Err(error);
            }
        };
        let Some(staged_path) = staged
            .path()
            .to_str()
            .filter(|value| !value.contains(['\0', '\n', '\r']))
            .map(ToOwned::to_owned)
        else {
            cleanup_staged_artifacts(staged_artifacts);
            let _ = staged.remove();
            return Err("managed integration staged artifact path is invalid".to_owned());
        };
        bindings.push(ManagedRuntimeArtifactBindingV1 {
            artifact_id: request.artifact_id.clone(),
            r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
            staged_path,
            size_bytes: request.size_bytes,
            sha256: request.sha256.clone(),
        });
        staged_artifacts.push(staged);
    }
    Ok((bindings, staged_artifacts))
}

fn cleanup_staged_artifacts(artifacts: Vec<StagedNativeArtifact>) {
    for artifact in artifacts {
        let _ = artifact.remove();
    }
}

fn bound_artifact<'a>(
    bundle: &'a VerifiedDistributionBundle,
    binding: &BundledManagedLaunchBinding,
) -> Result<&'a crate::distribution::bundle_verifier::VerifiedDistributionArtifact, String> {
    if bundle.manifest().distribution_id != binding.distribution_id() {
        return Err(
            "installed distribution identity does not match managed launch binding".to_owned(),
        );
    }
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == binding.artifact_id())
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    if artifact.expected_sha256() != binding.executable_sha256()
        || artifact.descriptor_sha256() != Some(binding.descriptor_sha256())
        || artifact.settings_schema_sha256() != binding.settings_schema_sha256()
    {
        return Err(
            "installed managed launch artifact does not match its durable binding".to_owned(),
        );
    }
    Ok(artifact)
}

fn platform_bound_artifact<'a>(
    bundle: &'a VerifiedDistributionBundle,
    binding: &PlatformManagedProcessBinding,
) -> Result<&'a crate::distribution::bundle_verifier::VerifiedDistributionArtifact, String> {
    if bundle.manifest().distribution_id != binding.distribution_id() {
        return Err(
            "installed distribution identity does not match platform process binding".to_owned(),
        );
    }
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == binding.artifact_id())
        .ok_or_else(|| {
            "platform process artifact is absent from distribution manifest".to_owned()
        })?;
    if artifact.expected_sha256() != binding.executable_sha256()
        || artifact.descriptor_sha256() != Some(binding.descriptor_sha256())
        || artifact.settings_schema_sha256() != binding.settings_schema_sha256()
    {
        return Err(
            "installed platform process artifact does not match its durable binding".to_owned(),
        );
    }
    Ok(artifact)
}

fn stage_artifact(
    artifact: &crate::distribution::bundle_verifier::VerifiedDistributionArtifact,
    launch_directory: &Path,
) -> Result<StagedNativeArtifact, String> {
    staged_artifact::stage(
        artifact.canonical_path(),
        launch_directory,
        &staged_artifact_name(artifact.expected_sha256()),
        artifact.expected_sha256(),
    )
}

fn staged_artifact_name(digest: &[u8; 32]) -> String {
    let mut name = String::from("artifact-");
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(name, "{byte:02x}");
    }
    name
}

fn read_stable_manifest(path: &Path) -> Result<Vec<u8>, String> {
    let before = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if before.file_type().is_symlink()
        || !before.is_file()
        || before.len() > MAX_SIGNED_MANIFEST_BYTES
    {
        return Err("signed distribution manifest is not a bounded regular file".to_owned());
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let opened = file.metadata().map_err(|error| error.to_string())?;
    if !same_file(&before, &opened) {
        return Err("signed distribution manifest changed while it was opened".to_owned());
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(opened.len())
            .map_err(|_| "signed distribution manifest is too large".to_owned())?,
    );
    file.read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let after = file.metadata().map_err(|error| error.to_string())?;
    let path_after = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !same_file(&opened, &after) || !same_file(&opened, &path_after) {
        return Err("signed distribution manifest changed while it was read".to_owned());
    }
    Ok(bytes)
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}
