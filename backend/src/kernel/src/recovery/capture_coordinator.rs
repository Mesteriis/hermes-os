//! Ordered whole-instance capture through component-owned offline ports.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::media::encryption::RecoveryMediaEncryptionKey;
use super::media::format::{RecoveryMediaInventoryV1, RecoveryMediaProvenanceV1};
use super::media::publish::{RecoveryMediaPublisher, RecoveryMediaSigner};

pub(crate) trait WholeInstanceCapturePort {
    fn verify_quiesced(&mut self) -> Result<(), String>;
    fn capture_control_store(&mut self, destination: &Path) -> Result<(), String>;
    fn capture_vault(&mut self, destination: &Path) -> Result<(), String>;
    fn capture_storage(&mut self, destination: &Path) -> Result<(), String>;
    fn capture_blob(&mut self, destination: &Path) -> Result<(), String>;
    fn capture_event_topology(&mut self, destination: &Path) -> Result<(), String>;
    fn capture_scheduler(&mut self, destination: &Path) -> Result<(), String>;
}

pub(crate) fn capture_verified_instance(
    destination: &Path,
    provenance: RecoveryMediaProvenanceV1,
    inventory: RecoveryMediaInventoryV1,
    signer: &impl RecoveryMediaSigner,
    encryption_key: &RecoveryMediaEncryptionKey,
    port: &mut impl WholeInstanceCapturePort,
) -> Result<PathBuf, String> {
    port.verify_quiesced()?;
    let publisher = RecoveryMediaPublisher::create(destination)?;
    let paths = CapturePaths::create(publisher.payload_root(), inventory)?;
    port.capture_control_store(&paths.control_store)?;
    port.capture_vault(&paths.vault)?;
    port.capture_storage(&paths.storage)?;
    if let Some(blob) = paths.blob.as_deref() {
        port.capture_blob(blob)?;
    }
    port.capture_event_topology(&paths.event_hub)?;
    if let Some(scheduler) = paths.scheduler.as_deref() {
        port.capture_scheduler(scheduler)?;
    }
    publisher.publish(provenance, inventory, signer, encryption_key)
}

struct CapturePaths {
    control_store: PathBuf,
    vault: PathBuf,
    storage: PathBuf,
    blob: Option<PathBuf>,
    event_hub: PathBuf,
    scheduler: Option<PathBuf>,
}

impl CapturePaths {
    fn create(root: &Path, inventory: RecoveryMediaInventoryV1) -> Result<Self, String> {
        Ok(Self {
            control_store: component_directory(root, "control-store")?,
            vault: component_directory(root, "vault")?,
            storage: component_directory(root, "storage")?,
            blob: inventory
                .blob_enabled()
                .then(|| component_directory(root, "blob"))
                .transpose()?,
            event_hub: component_directory(root, "event-hub")?,
            scheduler: inventory
                .scheduler_enabled()
                .then(|| component_directory(root, "scheduler"))
                .transpose()?,
        })
    }
}

fn component_directory(root: &Path, component: &str) -> Result<PathBuf, String> {
    let directory = root.join(component);
    std::fs::create_dir(&directory).map_err(|error| error.to_string())?;
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    Ok(directory)
}
