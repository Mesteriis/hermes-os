//! Ordered, fail-closed orchestration for a verified whole-instance restore.

use std::path::Path;

use super::media::encryption::RecoveryMediaEncryptionKey;
use super::media::verification::open_verified_recovery_media;

pub(crate) trait WholeInstanceRestorePort {
    fn verify_empty_target(&mut self) -> Result<(), String>;
    fn restore_control_store(&mut self, source: &Path) -> Result<(), String>;
    fn restore_vault(&mut self, source: &Path) -> Result<(), String>;
    fn restore_storage(&mut self, source: &Path) -> Result<(), String>;
    fn restore_blob(&mut self, source: &Path) -> Result<(), String>;
    fn recreate_event_topology(
        &mut self,
        source: &Path,
        control_store_source: &Path,
    ) -> Result<(), String>;
    fn prepare_outbox_inbox_replay(
        &mut self,
        scheduler_source: Option<&Path>,
    ) -> Result<(), String>;
    fn invalidate_stale_runtime_state(&mut self) -> Result<(), String>;
}

/// Verifies all media before calling any component and preserves the only safe
/// restore order. Component implementations own their own empty-target checks.
pub(crate) fn restore_verified_instance(
    media_root: &Path,
    key_id: &str,
    public_key_sec1: &[u8],
    decryption_workspace: &Path,
    encryption_key: &RecoveryMediaEncryptionKey,
    port: &mut impl WholeInstanceRestorePort,
) -> Result<(), String> {
    let (manifest, payload) = open_verified_recovery_media(
        media_root,
        key_id,
        public_key_sec1,
        decryption_workspace,
        encryption_key,
    )?;
    let inventory = manifest.inventory();
    let root = payload.root();
    port.verify_empty_target()?;
    port.restore_control_store(&root.join("control-store"))?;
    port.restore_vault(&root.join("vault"))?;
    port.restore_storage(&root.join("storage"))?;
    if inventory.blob_enabled() {
        port.restore_blob(&root.join("blob"))?;
    }
    port.recreate_event_topology(&root.join("event-hub"), &root.join("control-store"))?;
    port.prepare_outbox_inbox_replay(
        inventory
            .scheduler_enabled()
            .then(|| root.join("scheduler"))
            .as_deref(),
    )?;
    port.invalidate_stale_runtime_state()
}
