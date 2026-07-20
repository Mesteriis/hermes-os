use std::fs;

use sha2::{Digest, Sha256};

use crate::recovery::media::{
    RecoveryMediaEntryV1, RecoveryMediaManifestV1, SignedRecoveryMediaManifestV1,
};
use crate::recovery::restore_coordinator::{
    RestorePlanV1, WholeInstanceRestorePort, restore_verified_instance,
};
use crate::tests::common::{Signer, SigningKey, unique_target_root};

#[test]
fn coordinator_restores_verified_components_in_canonical_order() {
    let (root, signed, public_key) = signed_media();
    let mut port = RecordingPort::default();
    restore_verified_instance(
        &root,
        &signed,
        "recovery-key",
        &public_key,
        RestorePlanV1 {
            blob_enabled: true,
            scheduler_enabled: true,
        },
        &mut port,
    )
    .expect("restore");
    assert_eq!(
        port.calls,
        [
            "empty",
            "control",
            "vault",
            "storage",
            "blob",
            "events",
            "scheduler",
            "fence"
        ]
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn coordinator_stops_before_later_components_after_failure() {
    let (root, signed, public_key) = signed_media();
    let mut port = RecordingPort {
        fail_at: Some("storage"),
        ..Default::default()
    };
    assert!(
        restore_verified_instance(
            &root,
            &signed,
            "recovery-key",
            &public_key,
            RestorePlanV1 {
                blob_enabled: true,
                scheduler_enabled: true
            },
            &mut port,
        )
        .is_err()
    );
    assert_eq!(port.calls, ["empty", "control", "vault", "storage"]);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn coordinator_requires_an_empty_target_before_control_store_restore() {
    let (root, signed, public_key) = signed_media();
    let mut port = RecordingPort {
        fail_at: Some("empty"),
        ..Default::default()
    };
    assert!(
        restore_verified_instance(
            &root,
            &signed,
            "recovery-key",
            &public_key,
            RestorePlanV1 {
                blob_enabled: false,
                scheduler_enabled: false
            },
            &mut port,
        )
        .is_err()
    );
    assert_eq!(port.calls, ["empty"]);
    fs::remove_dir_all(root).expect("cleanup");
}

#[derive(Default)]
struct RecordingPort {
    calls: Vec<&'static str>,
    fail_at: Option<&'static str>,
}

impl RecordingPort {
    fn call(&mut self, name: &'static str) -> Result<(), String> {
        self.calls.push(name);
        if self.fail_at == Some(name) {
            return Err("component failed".to_owned());
        }
        Ok(())
    }
}

impl WholeInstanceRestorePort for RecordingPort {
    fn verify_empty_target(&mut self) -> Result<(), String> {
        self.call("empty")
    }
    fn restore_control_store(&mut self) -> Result<(), String> {
        self.call("control")
    }
    fn restore_vault(&mut self) -> Result<(), String> {
        self.call("vault")
    }
    fn restore_storage(&mut self) -> Result<(), String> {
        self.call("storage")
    }
    fn restore_blob(&mut self) -> Result<(), String> {
        self.call("blob")
    }
    fn restore_event_topology(&mut self) -> Result<(), String> {
        self.call("events")
    }
    fn restore_scheduler(&mut self) -> Result<(), String> {
        self.call("scheduler")
    }
    fn invalidate_stale_runtime_state(&mut self) -> Result<(), String> {
        self.call("fence")
    }
}

fn signed_media() -> (std::path::PathBuf, SignedRecoveryMediaManifestV1, [u8; 65]) {
    let root = unique_target_root("hermes-whole-restore");
    fs::create_dir_all(&root).expect("media root");
    let bytes = b"whole-instance-media";
    fs::write(root.join("control.bin"), bytes).expect("media");
    let entry = RecoveryMediaEntryV1::new(
        "control.bin".to_owned(),
        bytes.len() as u64,
        Sha256::digest(bytes).into(),
    )
    .expect("entry");
    let raw = RecoveryMediaManifestV1::encode(vec![entry]).expect("manifest");
    let key = SigningKey::from_bytes((&[11_u8; 32]).into()).expect("key");
    let signature: p256::ecdsa::Signature = key.sign(&raw);
    let signed = SignedRecoveryMediaManifestV1::new(
        "recovery-key".to_owned(),
        raw,
        signature.to_bytes().into(),
    )
    .expect("signed manifest");
    let public_key = key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("key");
    (root, signed, public_key)
}
