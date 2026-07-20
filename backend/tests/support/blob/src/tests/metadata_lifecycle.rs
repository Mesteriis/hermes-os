use std::fs;
use std::os::unix::fs::PermissionsExt;

use hermes_blob_protocol::{BlobAccessFenceV1, BlobBackupClassV1, BlobQuotaGrantV1, BlobRefV1};
use hermes_blob_runtime::lease::BlobKeyLeaseV1;
use hermes_blob_runtime::metadata::{BlobMetadataError, BlobMetadataLedger};
use hermes_blob_runtime::storage::{
    BlobContentLifecycleStore, BlobLifecycleError, EncryptedBlobStore,
};
use zeroize::Zeroizing;

#[test]
fn durable_write_reservation_enforces_aggregate_capability_quota() {
    let directory = private_directory();
    let ledger = BlobMetadataLedger::open(directory.path()).expect("metadata ledger");
    let first = reference(1, 13);
    let second = reference(2, 13);
    let access = access();
    let quota = quota(20);

    let reservation = ledger
        .reserve_write(&first, &access, &quota)
        .expect("reserve first Blob write");
    ledger
        .commit_write(&reservation, &first, &access)
        .expect("commit first Blob write");
    assert_eq!(
        ledger.reserve_write(&second, &access, &quota),
        Err(BlobMetadataError::QuotaExceeded)
    );
}

#[test]
fn owner_marked_delete_waits_for_grace_then_removes_bytes_and_reservation() {
    let directory = private_directory();
    let store = EncryptedBlobStore::open(directory.path(), 1024).expect("encrypted store");
    let ledger = BlobMetadataLedger::open(directory.path()).expect("metadata ledger");
    let reference = reference(3, 13);
    let access = access();
    let key_lease = lease(&reference, access.clone());
    let write = ledger
        .reserve_write(&reference, &access, &quota(20))
        .expect("reserve Blob write");
    store
        .write_new(&reference, &access, &key_lease, b"private bytes", 2)
        .expect("write encrypted Blob");
    ledger
        .commit_write(&write, &reference, &access)
        .expect("commit Blob write");

    let deletion = ledger
        .reserve_deletion(&reference, &access, 10, 5)
        .expect("owner metadata marks deletion eligible");
    assert_eq!(
        ledger.deletion_is_due(&deletion, &reference, &access, 14),
        Err(BlobMetadataError::ReservationMismatch)
    );
    ledger
        .deletion_is_due(&deletion, &reference, &access, 15)
        .expect("deletion becomes due after grace period");
    store
        .delete(&reference, &access, &key_lease, 15)
        .expect("delete encrypted bytes");
    ledger
        .finalize_deletion(&deletion, &reference, &access, 15)
        .expect("remove technical deletion reservation");
    assert_eq!(ledger.reconcile_missing_deletions(|_| Ok(false)), Ok(0));
}

#[test]
fn deletion_finalization_has_one_metadata_consumer() {
    let directory = private_directory();
    let ledger = BlobMetadataLedger::open(directory.path()).expect("metadata ledger");
    let reference = reference(9, 13);
    let access = access();
    let write = ledger
        .reserve_write(&reference, &access, &quota(20))
        .expect("reserve Blob write");
    ledger
        .commit_write(&write, &reference, &access)
        .expect("commit Blob write");
    let deletion = ledger
        .reserve_deletion(&reference, &access, 10, 5)
        .expect("reserve deletion");

    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| ledger.finalize_deletion(&deletion, &reference, &access, 15));
        let second = scope.spawn(|| ledger.finalize_deletion(&deletion, &reference, &access, 15));
        (
            first.join().expect("first collector"),
            second.join().expect("second collector"),
        )
    });

    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert!(
        matches!(first, Err(BlobMetadataError::NotFound))
            || matches!(second, Err(BlobMetadataError::NotFound))
    );
}

#[test]
fn pending_reservations_survive_reopen_and_are_released_after_failed_write() {
    let directory = private_directory();
    let pending_reference = reference(4, 13);
    let access = access();
    let quota = quota(13);
    let reservation = BlobMetadataLedger::open(directory.path())
        .expect("metadata ledger")
        .reserve_write(&pending_reference, &access, &quota)
        .expect("reserve write");
    let reopened = BlobMetadataLedger::open(directory.path()).expect("reopen ledger");
    assert_eq!(
        reopened.reserve_write(&reference(5, 1), &access, &quota),
        Err(BlobMetadataError::QuotaExceeded)
    );
    reopened
        .abandon_write(&reservation, &pending_reference, &access)
        .expect("release failed write reservation");
    reopened
        .reserve_write(&reference(5, 1), &access, &quota)
        .expect("quota becomes available after abandoned write");
}

#[test]
fn lifecycle_reopen_discards_ciphertext_from_an_uncommitted_write() {
    let directory = private_directory();
    let reference = reference(8, 13);
    let access = access();
    let key_lease = lease(&reference, access.clone());
    let ledger = BlobMetadataLedger::open(directory.path()).expect("metadata ledger");
    let encrypted = EncryptedBlobStore::open(directory.path(), 1024).expect("encrypted store");
    ledger
        .reserve_write(&reference, &access, &quota(20))
        .expect("reserve write before simulated crash");
    encrypted
        .write_new(&reference, &access, &key_lease, b"private bytes", 2)
        .expect("write before simulated crash");

    let lifecycle = BlobContentLifecycleStore::open(directory.path(), 1024)
        .expect("recover only uncommitted encrypted bytes");
    lifecycle
        .write_new(
            &reference,
            &access,
            &quota(20),
            &key_lease,
            b"private bytes",
            2,
        )
        .expect("recovered reservation and ciphertext permit a retry");
}

#[test]
fn lifecycle_store_reserves_aggregate_quota_before_persisting_encrypted_content() {
    let directory = private_directory();
    let store = BlobContentLifecycleStore::open(directory.path(), 1024).expect("lifecycle store");
    let first = reference(6, 13);
    let second = reference(7, 13);
    let access = access();
    let first_lease = lease(&first, access.clone());
    let second_lease = lease(&second, access.clone());

    store
        .write_new(
            &first,
            &access,
            &quota(20),
            &first_lease,
            b"private bytes",
            2,
        )
        .expect("write within aggregate quota");
    assert!(matches!(
        store.write_new(
            &second,
            &access,
            &quota(20),
            &second_lease,
            b"private bytes",
            2
        ),
        Err(BlobLifecycleError::Metadata(
            BlobMetadataError::QuotaExceeded
        ))
    ));
}

#[test]
fn reopening_ledger_discards_only_a_private_staged_metadata_record_after_crash() {
    let directory = private_directory();
    let staged = directory
        .path()
        .join("metadata")
        .join("08080808080808080808080808080808.staged");
    BlobMetadataLedger::open(directory.path()).expect("create metadata root");
    fs::write(&staged, b"incomplete").expect("write staged record");
    fs::set_permissions(&staged, fs::Permissions::from_mode(0o600)).expect("private staged record");

    BlobMetadataLedger::open(directory.path()).expect("recover staged metadata record");
    assert!(!staged.exists());
}

fn reference(value: u8, declared_size: u64) -> BlobRefV1 {
    BlobRefV1::new(
        [value; 16],
        "owner_notes",
        declared_size,
        Some(2_000),
        BlobBackupClassV1::Required,
    )
    .expect("Blob reference")
}

fn access() -> BlobAccessFenceV1 {
    BlobAccessFenceV1::new(
        "owner_notes",
        "registration_notes",
        "attachments",
        "notes_runtime",
        3,
        4,
    )
    .expect("Blob access fence")
}

fn quota(max_bytes: u64) -> BlobQuotaGrantV1 {
    BlobQuotaGrantV1::new(
        "owner_notes",
        "registration_notes",
        "attachments",
        4,
        max_bytes,
    )
    .expect("Blob quota grant")
}

fn lease(reference: &BlobRefV1, access: BlobAccessFenceV1) -> BlobKeyLeaseV1 {
    BlobKeyLeaseV1::from_vault_response(reference, access, 1_500, 1, Zeroizing::new(vec![9; 64]))
        .expect("Blob key lease")
}

fn private_directory() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary root");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private temporary root");
    directory
}
