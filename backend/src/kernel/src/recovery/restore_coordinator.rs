//! Ordered, fail-closed orchestration for a verified whole-instance restore.

use std::path::Path;

use super::media::{SignedRecoveryMediaManifestV1, verify_signed_inventory};

#[derive(Clone, Copy)]
pub(crate) struct RestorePlanV1 {
    pub(crate) blob_enabled: bool,
    pub(crate) scheduler_enabled: bool,
}

pub(crate) trait WholeInstanceRestorePort {
    fn verify_empty_target(&mut self) -> Result<(), String>;
    fn restore_control_store(&mut self) -> Result<(), String>;
    fn restore_vault(&mut self) -> Result<(), String>;
    fn restore_storage(&mut self) -> Result<(), String>;
    fn restore_blob(&mut self) -> Result<(), String>;
    fn recreate_event_topology(&mut self) -> Result<(), String>;
    fn replay_outbox_inbox(&mut self) -> Result<(), String>;
    fn restore_scheduler(&mut self) -> Result<(), String>;
    fn invalidate_stale_runtime_state(&mut self) -> Result<(), String>;
}

/// Verifies all media before calling any component and preserves the only safe
/// restore order. Component implementations own their own empty-target checks.
pub(crate) fn restore_verified_instance(
    media_root: &Path,
    signed_manifest: &SignedRecoveryMediaManifestV1,
    key_id: &str,
    public_key_sec1: &[u8],
    plan: RestorePlanV1,
    port: &mut impl WholeInstanceRestorePort,
) -> Result<(), String> {
    verify_signed_inventory(media_root, signed_manifest, key_id, public_key_sec1)?;
    port.verify_empty_target()?;
    port.restore_control_store()?;
    port.restore_vault()?;
    port.restore_storage()?;
    if plan.blob_enabled {
        port.restore_blob()?;
    }
    port.recreate_event_topology()?;
    port.replay_outbox_inbox()?;
    if plan.scheduler_enabled {
        port.restore_scheduler()?;
    }
    port.invalidate_stale_runtime_state()
}
