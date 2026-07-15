use chrono::{DateTime, Utc};

use super::super::errors::ProjectStoreError;
use super::super::ids::project_graph_node_id;
use super::super::models::ProjectStats;
use super::super::review_targets::reviewed_target_ids;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_stats(
        &self,
        project_id: &str,
    ) -> Result<ProjectStats, ProjectStoreError> {
        let message_targets = self.active_project_messages(project_id).await?;
        let message_ids = reviewed_target_ids(&message_targets);
        let document_targets = self.active_project_documents(project_id).await?;
        let document_ids = reviewed_target_ids(&document_targets);

        let message_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM communication_messages message
            WHERE message_id = ANY($1)
            "#,
        )
        .bind(&message_ids)
        .fetch_one(&self.pool)
        .await?;

        let document_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM documents document
            WHERE document_id = ANY($1)
            "#,
        )
        .bind(&document_ids)
        .fetch_one(&self.pool)
        .await?;

        let persona_count = sqlx::query_scalar::<_, i64>(
            r#"
            WITH project_messages AS (
                SELECT sender, recipients
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            participants AS (
                SELECT lower(trim(sender)) AS email_address
                FROM project_messages
                UNION ALL
                SELECT lower(trim(recipient.value)) AS email_address
                FROM project_messages message,
                     jsonb_array_elements_text(message.recipients) AS recipient(value)
            )
            SELECT count(DISTINCT email_address)
            FROM participants
            WHERE email_address <> ''
            "#,
        )
        .bind(&message_ids)
        .fetch_one(&self.pool)
        .await?;

        let graph_node_id = project_graph_node_id(project_id);
        let graph_connection_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM graph_edges
            WHERE valid_to IS NULL
              AND (source_node_id = $1 OR target_node_id = $1)
            "#,
        )
        .bind(&graph_node_id)
        .fetch_one(&self.pool)
        .await?;

        let latest_activity_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            WITH project_message_activity AS (
                SELECT COALESCE(occurred_at, projected_at) AS occurred_at
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            project_document_activity AS (
                SELECT imported_at AS occurred_at
                FROM documents document
                WHERE document_id = ANY($2)
            )
            SELECT max(occurred_at)
            FROM (
                SELECT occurred_at FROM project_message_activity
                UNION ALL
                SELECT occurred_at FROM project_document_activity
            ) activity
            "#,
        )
        .bind(&message_ids)
        .bind(&document_ids)
        .fetch_one(&self.pool)
        .await?;

        Ok(ProjectStats {
            message_count,
            document_count,
            persona_count,
            #[allow(deprecated)]
            people_count: persona_count,
            graph_connection_count,
            latest_activity_at,
        })
    }
}
