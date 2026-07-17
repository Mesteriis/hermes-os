//! Owner-authorized Telemetry Collector admission from a signed distribution.

use std::path::Path;

use hermes_kernel_control_store::PlatformManagedProcessBinding;
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::v1::ModuleKindV1;

use crate::distribution::bundle_verifier::{
    VerifiedDistributionArtifact, VerifiedDistributionBundle,
};
use crate::platform::macos::native_launch;

pub const TELEMETRY_PROCESS_ID: &str = "telemetry";
const TELEMETRY_MODULE_ID: &str = "telemetry";
const TELEMETRY_OWNER_ID: &str = "telemetry";
const MACOS_AARCH64_TARGET: &str = "aarch64-apple-darwin";

pub fn bind_current_installed_release(
    store: &SqliteControlStore,
) -> Result<PlatformManagedProcessBinding, String> {
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    bind_installed_release(store, &kernel)
}

pub fn bind_installed_release(
    store: &SqliteControlStore,
    kernel: &Path,
) -> Result<PlatformManagedProcessBinding, String> {
    let bundle = native_launch::verify_selected_installed_bundle(kernel, MACOS_AARCH64_TARGET)?;
    admit(store, &bundle)
}

pub fn admit(
    store: &SqliteControlStore,
    bundle: &VerifiedDistributionBundle,
) -> Result<PlatformManagedProcessBinding, String> {
    let artifact = designated_artifact(bundle)?;
    let binding = PlatformManagedProcessBinding::new(
        TELEMETRY_PROCESS_ID,
        next_binding_revision(store)?,
        &bundle.manifest().distribution_id,
        artifact.artifact_id(),
        *artifact.expected_sha256(),
        *artifact
            .descriptor_sha256()
            .ok_or_else(|| "Telemetry release artifact lacks a module descriptor".to_owned())?,
        artifact.settings_schema_sha256().copied(),
    );
    store
        .record_platform_managed_process_binding(&binding)
        .map_err(|error| format!("{error:?}"))?;
    Ok(binding)
}

fn designated_artifact(
    bundle: &VerifiedDistributionBundle,
) -> Result<&VerifiedDistributionArtifact, String> {
    let artifacts = bundle
        .artifacts()
        .iter()
        .filter(|artifact| is_designated(artifact))
        .collect::<Vec<_>>();
    match artifacts.as_slice() {
        [artifact] => Ok(*artifact),
        _ => {
            Err("signed release must contain exactly one designated Telemetry artifact".to_owned())
        }
    }
}

fn is_designated(artifact: &VerifiedDistributionArtifact) -> bool {
    let Some(descriptor) = artifact.module_descriptor() else {
        return false;
    };
    descriptor.module_id == TELEMETRY_MODULE_ID
        && descriptor.owner_id == TELEMETRY_OWNER_ID
        && descriptor.module_kind == ModuleKindV1::Platform as i32
}

fn next_binding_revision(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_binding(TELEMETRY_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |binding| {
            binding
                .binding_revision()
                .checked_add(1)
                .ok_or_else(|| "Telemetry launch binding revision overflowed".to_owned())
        })
}
