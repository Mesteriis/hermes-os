use std::ops::Deref;
use std::path::Path;

use sqlx::postgres::PgPool;

use super::blob_store::{LocalCommunicationBlob, LocalCommunicationBlobStore};
use super::errors::CommunicationStorageError;
use super::models::{
    ImportedCommunicationAttachment, NewCommunicationAttachment, NewCommunicationBlob,
    StoredCommunicationAttachment, StoredCommunicationBlob,
};
use super::store::CommunicationStorageStore;

/// Workflow-facing metadata boundary for communication blobs and attachments.
#[derive(Clone)]
pub struct CommunicationAttachmentPort {
    store: CommunicationStorageStore,
}

impl CommunicationAttachmentPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            store: CommunicationStorageStore::new(pool),
        }
    }

    pub async fn upsert_blob(
        &self,
        blob: &NewCommunicationBlob,
    ) -> Result<StoredCommunicationBlob, CommunicationStorageError> {
        self.store.upsert_blob(blob).await
    }

    pub async fn upsert_attachment(
        &self,
        attachment: &NewCommunicationAttachment,
    ) -> Result<StoredCommunicationAttachment, CommunicationStorageError> {
        self.store.upsert_attachment(attachment).await
    }

    pub async fn imported_attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        self.store.imported_attachment_by_id(attachment_id).await
    }
}

/// Workflow-facing local blob boundary. It deliberately exposes blob data
/// operations, not arbitrary filesystem access.
#[derive(Clone, Debug)]
pub struct LocalBlobPort {
    store: LocalCommunicationBlobStore,
}

impl LocalBlobPort {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            store: LocalCommunicationBlobStore::new(root),
        }
    }

    pub fn root(&self) -> &Path {
        self.store.root()
    }

    pub async fn put_blob(
        &self,
        bytes: &[u8],
    ) -> Result<LocalCommunicationBlob, CommunicationStorageError> {
        self.store.put_blob(bytes).await
    }
}

impl Deref for LocalBlobPort {
    type Target = LocalCommunicationBlobStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
