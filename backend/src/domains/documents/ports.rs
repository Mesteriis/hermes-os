use sqlx::{Postgres, Transaction};

use super::core::errors::DocumentImportError;
use super::core::models::{ImportedDocument, NewDocumentImport};
use super::core::store::DocumentImportStore;

#[derive(Clone)]
pub struct DocumentImportPort(DocumentImportStore);

impl DocumentImportPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(DocumentImportStore::new(pool))
    }

    pub async fn import_document(
        &self,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        self.0.import_document(document).await
    }

    pub(crate) async fn import_document_manual_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        source_ref: String,
        provenance: serde_json::Value,
        source_observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedDocument, DocumentImportError> {
        DocumentImportStore::import_document_manual_with_observation_in_transaction(
            transaction,
            document,
            source_ref,
            provenance,
            source_observation_id,
            relationship_kind,
            metadata,
        )
        .await
    }
}
