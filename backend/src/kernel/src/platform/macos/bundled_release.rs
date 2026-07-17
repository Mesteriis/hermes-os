//! Binds an approved registration to one verified module artifact in this macOS app release.

use std::path::Path;

use hermes_kernel_control_store::BundledManagedLaunchBinding;
use hermes_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::bundled_launch;
use crate::platform::macos::native_launch;

const MACOS_AARCH64_TARGET: &str = "aarch64-apple-darwin";

pub fn bind_current_installed_release(
    store: &SqliteControlStore,
    registration_id: &str,
    artifact_id: &str,
) -> Result<BundledManagedLaunchBinding, String> {
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    bind_installed_release(store, registration_id, artifact_id, &kernel_executable)
}

pub fn bind_installed_release(
    store: &SqliteControlStore,
    registration_id: &str,
    artifact_id: &str,
    kernel_executable: &Path,
) -> Result<BundledManagedLaunchBinding, String> {
    let bundle =
        native_launch::verify_selected_installed_bundle(kernel_executable, MACOS_AARCH64_TARGET)?;
    bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    bundled_launch::admit(store, registration_id, &bundle, artifact_id)
}
