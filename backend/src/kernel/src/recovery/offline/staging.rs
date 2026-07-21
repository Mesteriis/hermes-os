use std::path::Path;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::infrastructure::filesystem::resolve_runtime_directory;
use crate::platform::macos::native_launch;
use crate::recovery::process_port::RecoveryComponentExecutables;

pub(crate) struct StagedRecoveryComponents {
    pub(crate) vault: StagedNativeArtifact,
    pub(crate) storage: StagedNativeArtifact,
    pub(crate) blob: Option<StagedNativeArtifact>,
    pub(crate) scheduler: Option<StagedNativeArtifact>,
}

impl StagedRecoveryComponents {
    pub(crate) fn prepare(data_dir: &Path, blob: bool, scheduler: bool) -> Result<Self, String> {
        let kernel = std::env::current_exe()
            .map_err(|_| "Kernel executable path is unavailable".to_owned())?;
        let runtime = resolve_runtime_directory(data_dir)?.join("offline-recovery");
        let vault = stage(&kernel, "platform.vault", &runtime.join("vault"))?;
        let storage = stage(&kernel, "platform.storage", &runtime.join("storage"))?;
        let blob = stage_optional(&kernel, blob, "platform.blob", &runtime.join("blob"))?;
        let scheduler = stage_optional(
            &kernel,
            scheduler,
            "platform.scheduler",
            &runtime.join("scheduler"),
        )?;
        Ok(Self {
            vault,
            storage,
            blob,
            scheduler,
        })
    }

    pub(crate) fn components(&self) -> RecoveryComponentExecutables<'_> {
        RecoveryComponentExecutables {
            vault: &self.vault,
            storage: &self.storage,
            blob: self.blob.as_ref(),
            scheduler: self.scheduler.as_ref(),
        }
    }

    pub(crate) fn remove(self) -> Result<(), String> {
        self.vault.remove()?;
        self.storage.remove()?;
        if let Some(blob) = self.blob {
            blob.remove()?;
        }
        if let Some(scheduler) = self.scheduler {
            scheduler.remove()?;
        }
        Ok(())
    }
}

fn stage(
    kernel: &Path,
    artifact_id: &str,
    destination: &Path,
) -> Result<StagedNativeArtifact, String> {
    native_launch::verify_selected_installed_release(
        kernel,
        "aarch64-apple-darwin",
        artifact_id,
        destination,
    )
}

fn stage_optional(
    kernel: &Path,
    enabled: bool,
    artifact_id: &str,
    destination: &Path,
) -> Result<Option<StagedNativeArtifact>, String> {
    enabled
        .then(|| stage(kernel, artifact_id, destination))
        .transpose()
}
