use chrono::{DateTime, Utc};

use crate::domains::projects::link_reviews::{ProjectLinkReviewStore, ProjectReviewedTarget};
use crate::engines::timeline::TimelineEngine;

use super::errors::ProjectStoreError;
use super::ids::project_graph_node_id;
use super::models::{
    Project, ProjectDocumentSummary, ProjectMessageSummary, ProjectPersonSummary, ProjectStats,
    ProjectTimelineItem,
};
use super::projection::reviewed_target_ids;
use super::rows::{
    row_to_project, row_to_project_document, row_to_project_message, row_to_project_person,
    row_to_timeline_item,
};
use super::store::ProjectStore;

impl ProjectStore {
    pub(super) async fn project_by_id(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date,
                created_at,
                updated_at
            FROM projects
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_project).transpose()
    }

    pub(super) async fn project_keywords(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, ProjectStoreError> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT keyword
            FROM project_keywords
            WHERE project_id = $1
            ORDER BY keyword
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub(super) async fn project_stats(
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

        let people_count = sqlx::query_scalar::<_, i64>(
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
            people_count,
            graph_connection_count,
            latest_activity_at,
        })
    }

    pub(super) async fn project_messages(
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

    pub(super) async fn project_documents(
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

    pub(super) async fn project_people(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectPersonSummary>, ProjectStoreError> {
        let message_ids = reviewed_target_ids(&self.active_project_messages(project_id).await?);
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            WITH project_messages AS (
                SELECT sender, recipients, COALESCE(occurred_at, projected_at) AS occurred_at
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            participants AS (
                SELECT lower(trim(sender)) AS email_address, occurred_at
                FROM project_messages
                UNION ALL
                SELECT lower(trim(recipient.value)) AS email_address, message.occurred_at
                FROM project_messages message,
                     jsonb_array_elements_text(message.recipients) AS recipient(value)
            )
            SELECT
                COALESCE(person.display_name, participants.email_address) AS display_name,
                participants.email_address,
                count(*)::BIGINT AS interaction_count,
                max(participants.occurred_at) AS last_interaction_at
            FROM participants
            LEFT JOIN persons person ON person.email_address = participants.email_address
            WHERE participants.email_address <> ''
            GROUP BY participants.email_address, person.display_name
            ORDER BY interaction_count DESC, last_interaction_at DESC NULLS LAST, display_name
            LIMIT $2
            "#,
        )
        .bind(&message_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_project_person).collect()
    }

    pub(super) async fn project_timeline(
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

    pub(super) async fn active_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_message_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }

    pub(super) async fn active_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_document_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }
}
