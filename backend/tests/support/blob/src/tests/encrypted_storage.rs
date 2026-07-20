use std::fs;
use std::os::unix::fs::PermissionsExt;

use hermes_blob_protocol::{BlobAccessFenceV1, BlobBackupClassV1, BlobRangeV1, BlobRefV1};
use hermes_blob_runtime::lease::BlobKeyLeaseV1;
use hermes_blob_runtime::storage::{BlobStorageError, EncryptedBlobStore};
use zeroize::Zeroizing;

fn reference() -> BlobRefV1 {
    BlobRefV1::new(
        [7; 16],
        "owner_notes",
        13,
        Some(2_000),
        BlobBackupClassV1::Required,
    )
    .expect("Blob reference")
}

fn fence() -> BlobAccessFenceV1 {
    BlobAccessFenceV1::new(
        "owner_notes",
        "registration",
        "attachments",
        "runtime",
        3,
        4,
    )
    .expect("fence")
}

fn lease(reference: &BlobRefV1, fence: BlobAccessFenceV1) -> BlobKeyLeaseV1 {
    BlobKeyLeaseV1::from_vault_response(reference, fence, 1_500, 1, Zeroizing::new(vec![9; 64]))
        .expect("Vault-routed key lease")
}

fn private_directory() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary root");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private temporary root");
    directory
}

#[test]
fn blob_refs_remain_opaque_and_ranges_stay_within_declared_size() {
    let reference = reference();
    assert_eq!(reference.owner_id(), "owner_notes");
    assert!(BlobRangeV1::new(0, 12, reference.declared_size()).is_ok());
    assert!(BlobRangeV1::new(12, 12, reference.declared_size()).is_err());
    assert!(BlobRangeV1::new(0, 14, reference.declared_size()).is_err());
    assert!(BlobRefV1::new([0; 16], "owner_notes", 1, None, BlobBackupClassV1::Required,).is_err());
}

#[test]
fn encrypted_store_writes_no_plaintext_and_returns_only_bounded_ranges() {
    let directory = private_directory();
    let store = EncryptedBlobStore::open(directory.path(), 1024).expect("store");
    let reference = reference();
    let fence = fence();
    let lease = lease(&reference, fence.clone());
    let plaintext = b"private bytes";

    store
        .write_new(&reference, &fence, &lease, plaintext, 2)
        .expect("encrypted write");

    let filename = "07070707070707070707070707070707.blob";
    let stored = fs::read(directory.path().join("content").join(filename)).expect("ciphertext");
    assert!(
        !stored
            .windows(plaintext.len())
            .any(|window| window == plaintext)
    );
    assert_eq!(
        store
            .read_range(
                &reference,
                &fence,
                &lease,
                BlobRangeV1::new(8, 13, reference.declared_size()).expect("range"),
                3,
            )
            .expect("bounded read"),
        b"bytes",
    );
}

#[test]
fn lease_fence_expiry_and_symlink_attacks_fail_closed() {
    let directory = private_directory();
    let store = EncryptedBlobStore::open(directory.path(), 1024).expect("store");
    let reference = reference();
    let fence = fence();
    let lease = lease(&reference, fence.clone());

    assert_eq!(
        store.write_new(&reference, &fence, &lease, b"private bytes", 1_500),
        Err(BlobStorageError::Lease(
            hermes_blob_runtime::lease::BlobLeaseError::Expired
        )),
    );

    let path = directory
        .path()
        .join("content")
        .join("07070707070707070707070707070707.blob");
    std::os::unix::fs::symlink("/tmp", &path).expect("test symlink");
    assert_eq!(
        store.write_new(&reference, &fence, &lease, b"private bytes", 2),
        Err(BlobStorageError::AlreadyExists),
    );
}

#[test]
fn expired_reference_cannot_be_written_or_read_with_a_current_key_lease() {
    let directory = private_directory();
    let store = EncryptedBlobStore::open(directory.path(), 1024).expect("store");
    let reference = reference();
    let fence = fence();
    let lease = lease(&reference, fence.clone());

    assert_eq!(
        store.write_new(&reference, &fence, &lease, b"private bytes", 2_000),
        Err(BlobStorageError::Expired),
    );
    store
        .write_new(&reference, &fence, &lease, b"private bytes", 2)
        .expect("write before reference expiry");
    assert_eq!(
        store.read_range(
            &reference,
            &fence,
            &lease,
            BlobRangeV1::new(0, 1, reference.declared_size()).expect("range"),
            2_000,
        ),
        Err(BlobStorageError::Expired),
    );
}

#[test]
fn owner_fenced_delete_removes_the_blob() {
    let directory = private_directory();
    let store = EncryptedBlobStore::open(directory.path(), 1024).expect("store");
    let reference = reference();
    let fence = fence();
    let lease = lease(&reference, fence.clone());
    store
        .write_new(&reference, &fence, &lease, b"private bytes", 2)
        .expect("encrypted write");

    store
        .delete(&reference, &fence, &lease, 3)
        .expect("owner-fenced delete");
    assert!(
        !directory
            .path()
            .join("content/07070707070707070707070707070707.blob")
            .exists()
    );
}

#[test]
fn ciphertext_is_bound_to_the_complete_reference_and_private_root() {
    let directory = private_directory();
    let store = EncryptedBlobStore::open(directory.path(), 1024).expect("store");
    let reference = reference();
    let fence = fence();
    let key_lease = lease(&reference, fence.clone());
    store
        .write_new(&reference, &fence, &key_lease, b"private bytes", 2)
        .expect("encrypted write");

    let altered = BlobRefV1::new(
        [7; 16],
        "owner_notes",
        13,
        Some(2_000),
        BlobBackupClassV1::Rebuildable,
    )
    .expect("altered reference");
    let altered_lease = lease(&altered, fence.clone());
    assert_eq!(
        store.read_range(
            &altered,
            &fence,
            &altered_lease,
            BlobRangeV1::new(0, 1, altered.declared_size()).expect("range"),
            3,
        ),
        Err(BlobStorageError::AuthenticationFailed),
    );

    let symlink_root = directory.path().join("alias");
    std::os::unix::fs::symlink(directory.path(), &symlink_root).expect("test root symlink");
    assert!(matches!(
        EncryptedBlobStore::open(&symlink_root, 1024),
        Err(BlobStorageError::UnsafePath)
    ));
}
