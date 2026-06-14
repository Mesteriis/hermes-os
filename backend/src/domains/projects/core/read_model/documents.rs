use super::super::errors::ProjectStoreError;
use super::super::models::ProjectDocumentSummary;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_project_document;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_documents(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectDocumentSummary>, ProjectStoreError> {
        let document_ids = reviewed_target_ids(&self.active_project_documents(project_id).await?);
        if document_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, imported_at
            FROM documents document
            WHERE document_id = ANY($1)
            ORDER BY imported_at DESC, document_id
            LIMIT $2
            "#,
        )
        .bind(&document_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_project_document).collect()
    }
}
