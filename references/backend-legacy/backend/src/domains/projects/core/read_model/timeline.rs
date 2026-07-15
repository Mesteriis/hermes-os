use crate::engines::timeline::TimelineEngine;

use super::super::errors::ProjectStoreError;
use super::super::models::ProjectTimelineItem;
use super::super::review_targets::reviewed_target_ids;
use super::super::rows::row_to_timeline_item;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_timeline(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectTimelineItem>, ProjectStoreError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let message_ids = reviewed_target_ids(&self.active_project_messages(project_id).await?);
        let document_ids = reviewed_target_ids(&self.active_project_documents(project_id).await?);

        let rows = sqlx::query(
            r#"
            WITH project_messages AS (
                SELECT
                    'message' AS item_kind,
                    message_id AS item_id,
                    subject AS title,
                    sender AS subtitle,
                    COALESCE(occurred_at, projected_at) AS occurred_at
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            project_documents AS (
                SELECT
                    'document' AS item_kind,
                    document_id AS item_id,
                    title,
                    document_kind AS subtitle,
                    imported_at AS occurred_at
                FROM documents document
                WHERE document_id = ANY($2)
            )
            SELECT item_kind, item_id, title, subtitle, occurred_at
            FROM (
                SELECT * FROM project_messages
                UNION ALL
                SELECT * FROM project_documents
            ) timeline
            ORDER BY occurred_at DESC, item_kind, item_id
            LIMIT $3
            "#,
        )
        .bind(&message_ids)
        .bind(&document_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_timeline_item).collect()
    }
}
