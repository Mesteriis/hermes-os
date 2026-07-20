//! Crash-aware composition of encrypted Blob bytes and technical quota metadata.

use std::path::Path;

use hermes_blob_protocol::{BlobAccessFenceV1, BlobQuotaGrantV1, BlobRangeV1, BlobRefV1};

use crate::lease::BlobKeyLeaseV1;
use crate::metadata::{BlobDeletionReservationV1, BlobMetadataError, BlobMetadataLedger};

use super::{BlobStorageError, EncryptedBlobStore};

/// Resolves one current deletion key or explicitly defers the removal.
pub trait BlobDeletionLeaseResolverV1 {
    fn resolve_deletion_lease(
        &mut self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        now_unix_ms: u64,
    ) -> Result<BlobKeyLeaseV1, BlobDeletionLeaseErrorV1>;
}

/// A revoked or unavailable key never authorizes an implicit Blob deletion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobDeletionLeaseErrorV1 {
    Revoked,
    Unavailable,
}

/// Sanitized result of one scheduled deletion pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlobGarbageCollectionReportV1 {
    deleted: u64,
    deferred: u64,
}

impl BlobGarbageCollectionReportV1 {
    #[must_use]
    pub const fn deleted(&self) -> u64 {
        self.deleted
    }

    #[must_use]
    pub const fn deferred(&self) -> u64 {
        self.deferred
    }
}

/// The only Blob write lifecycle that applies aggregate quota reservations.
pub struct BlobContentLifecycleStore {
    encrypted: EncryptedBlobStore,
    metadata: BlobMetadataLedger,
}

impl BlobContentLifecycleStore {
    pub fn open(data_dir: &Path, maximum_blob_bytes: u64) -> Result<Self, BlobLifecycleError> {
        let store = Self {
            encrypted: EncryptedBlobStore::open(data_dir, maximum_blob_bytes)
                .map_err(BlobLifecycleError::Storage)?,
            metadata: BlobMetadataLedger::open(data_dir).map_err(BlobLifecycleError::Metadata)?,
        };
        store.recover_uncommitted_writes()?;
        Ok(store)
    }

    pub fn write_new(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        quota: &BlobQuotaGrantV1,
        lease: &BlobKeyLeaseV1,
        plaintext: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), BlobLifecycleError> {
        let reservation = self
            .metadata
            .reserve_write(reference, access, quota)
            .map_err(BlobLifecycleError::Metadata)?;
        match self
            .encrypted
            .write_new(reference, access, lease, plaintext, now_unix_ms)
        {
            Ok(()) => self
                .metadata
                .commit_write(&reservation, reference, access)
                .map_err(BlobLifecycleError::Metadata),
            Err(error) => {
                let _ = self.metadata.abandon_write(&reservation, reference, access);
                Err(BlobLifecycleError::Storage(error))
            }
        }
    }

    pub fn read_range(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        lease: &BlobKeyLeaseV1,
        range: BlobRangeV1,
        now_unix_ms: u64,
    ) -> Result<Vec<u8>, BlobLifecycleError> {
        self.encrypted
            .read_range(reference, access, lease, range, now_unix_ms)
            .map_err(BlobLifecycleError::Storage)
    }

    pub fn reserve_deletion(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        now_unix_ms: u64,
        grace_period_ms: u64,
    ) -> Result<BlobDeletionReservationV1, BlobLifecycleError> {
        self.metadata
            .reserve_deletion(reference, access, now_unix_ms, grace_period_ms)
            .map_err(BlobLifecycleError::Metadata)
    }

    pub fn delete_due(
        &self,
        reservation: &BlobDeletionReservationV1,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        lease: &BlobKeyLeaseV1,
        now_unix_ms: u64,
    ) -> Result<(), BlobLifecycleError> {
        self.metadata
            .deletion_is_due(reservation, reference, access, now_unix_ms)
            .map_err(BlobLifecycleError::Metadata)?;
        self.encrypted
            .delete(reference, access, lease, now_unix_ms)
            .map_err(BlobLifecycleError::Storage)?;
        self.metadata
            .finalize_deletion(reservation, reference, access, now_unix_ms)
            .map_err(BlobLifecycleError::Metadata)
    }

    pub fn reconcile_missing_deletions(&self) -> Result<u64, BlobLifecycleError> {
        self.metadata
            .reconcile_missing_deletions(|reference| {
                self.encrypted
                    .exists(reference)
                    .map_err(|_| BlobMetadataError::Filesystem)
            })
            .map_err(BlobLifecycleError::Metadata)
    }

    pub fn collect_due_deletions<R>(
        &self,
        resolver: &mut R,
        now_unix_ms: u64,
    ) -> Result<BlobGarbageCollectionReportV1, BlobLifecycleError>
    where
        R: BlobDeletionLeaseResolverV1,
    {
        let mut report = BlobGarbageCollectionReportV1 {
            deleted: 0,
            deferred: 0,
        };
        for due in self
            .metadata
            .due_deletions(now_unix_ms)
            .map_err(BlobLifecycleError::Metadata)?
        {
            match resolver.resolve_deletion_lease(due.reference(), due.access(), now_unix_ms) {
                Ok(lease) => {
                    self.delete_due(
                        due.reservation(),
                        due.reference(),
                        due.access(),
                        &lease,
                        now_unix_ms,
                    )?;
                    report.deleted += 1;
                }
                Err(BlobDeletionLeaseErrorV1::Revoked | BlobDeletionLeaseErrorV1::Unavailable) => {
                    report.deferred += 1;
                }
            }
        }
        Ok(report)
    }

    fn recover_uncommitted_writes(&self) -> Result<(), BlobLifecycleError> {
        for pending in self
            .metadata
            .pending_writes()
            .map_err(BlobLifecycleError::Metadata)?
        {
            self.encrypted
                .discard_uncommitted(pending.reference())
                .map_err(BlobLifecycleError::Storage)?;
            self.metadata
                .discard_pending_write(&pending)
                .map_err(BlobLifecycleError::Metadata)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum BlobLifecycleError {
    Metadata(BlobMetadataError),
    Storage(BlobStorageError),
}
