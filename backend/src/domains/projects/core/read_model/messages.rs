use super::super::errors::ProjectStoreError;
use super::super::models::ProjectMessageSummary;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_project_message;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_messages(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectMessageSummary>, ProjectStoreError> {
        let message_ids = reviewed_target_ids(&self.active_project_messages(project_id).await?);
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                subject,
                sender,
                COALESCE(occurred_at, projected_at) AS occurred_at
            FROM communication_messages message
            WHERE message_id = ANY($1)
            ORDER BY occurred_at DESC, message_id
            LIMIT $2
            "#,
        )
        .bind(&message_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_project_message).collect()
    }
}
