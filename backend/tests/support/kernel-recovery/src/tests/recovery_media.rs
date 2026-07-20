use std::fs;

use sha2::{Digest, Sha256};

use crate::recovery::media::{
    RecoveryMediaEntryV1, SignedRecoveryMediaManifestV1, verify_inventory,
};
use crate::tests::common::{Signer, SigningKey, unique_target_root};

#[test]
fn recovery_media_requires_an_exact_regular_file_inventory() {
    let root = unique_target_root("hermes-recovery-media");
    fs::create_dir_all(root.join("vault")).expect("media root");
    let bytes = b"verified-vault-snapshot";
    fs::write(root.join("vault/snapshot.bin"), bytes).expect("snapshot");
    let entry = RecoveryMediaEntryV1::new(
        "vault/snapshot.bin".to_owned(),
        bytes.len() as u64,
        Sha256::digest(bytes).into(),
    )
    .expect("entry");
    assert!(verify_inventory(&root, std::slice::from_ref(&entry)).is_ok());

    fs::write(root.join("extra.bin"), b"unexpected").expect("extra file");
    assert!(verify_inventory(&root, &[entry]).is_err());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn recovery_media_rejects_path_escape_and_digest_drift() {
    assert!(RecoveryMediaEntryV1::new("../escape".to_owned(), 1, [0; 32]).is_err());
    let root = unique_target_root("hermes-recovery-media-digest");
    fs::create_dir_all(&root).expect("media root");
    fs::write(root.join("control.bin"), b"changed").expect("control store");
    let entry = RecoveryMediaEntryV1::new("control.bin".to_owned(), 7, [0; 32]).expect("entry");
    assert!(verify_inventory(&root, &[entry]).is_err());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn recovery_media_rejects_symlinked_manifest_entry() {
    let root = unique_target_root("hermes-recovery-media-symlink");
    fs::create_dir_all(&root).expect("media root");
    let external = unique_target_root("hermes-recovery-media-external");
    fs::write(&external, b"external").expect("external file");
    std::os::unix::fs::symlink(&external, root.join("vault.bin")).expect("media symlink");
    let entry = RecoveryMediaEntryV1::new(
        "vault.bin".to_owned(),
        8,
        Sha256::digest(b"external").into(),
    )
    .expect("entry");
    assert!(verify_inventory(&root, &[entry]).is_err());
    fs::remove_dir_all(root).expect("cleanup media");
    fs::remove_file(external).expect("cleanup external");
}

#[test]
fn recovery_media_requires_the_pinned_manifest_signature() {
    let key = SigningKey::from_bytes((&[7_u8; 32]).into()).expect("signing key");
    let raw = b"canonical recovery manifest".to_vec();
    let signature: p256::ecdsa::Signature = key.sign(&raw);
    let manifest = SignedRecoveryMediaManifestV1::new(
        "recovery-media-2026".to_owned(),
        raw,
        signature.to_bytes().into(),
    )
    .expect("signed manifest");
    let public_key = key.verifying_key().to_sec1_point(false);
    assert!(
        manifest
            .verify("recovery-media-2026", public_key.as_bytes())
            .is_ok()
    );
    assert!(
        manifest
            .verify("different-key", public_key.as_bytes())
            .is_err()
    );
}
