use std::collections::HashMap;

use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectReviewedTarget};

use super::errors::ProjectStoreError;
use super::models::{ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource};
use super::rows::{row_to_matched_document, row_to_matched_message};
use super::store::ProjectStore;

impl ProjectStore {
    pub(crate) async fn graph_projection_projects(
        &self,
    ) -> Result<Vec<ProjectProjectionSource>, ProjectStoreError> {
        let rows = sqlx::query(
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
            ORDER BY project_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut projects = Vec::new();
        for row in rows {
            let project = super::rows::row_to_project(row)?;
            projects.push(ProjectProjectionSource {
                keywords: self.project_keywords(&project.project_id).await?,
                project,
            });
        }

        Ok(projects)
    }

    pub(crate) async fn matching_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMatchedMessage>, ProjectStoreError> {
        let reviewed = self.active_project_messages(project_id).await?;
        if reviewed.is_empty() {
            return Ok(Vec::new());
        }
        let (message_ids, reviewed_by_id) = reviewed_targets_and_map(reviewed);

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                occurred_at,
                projected_at
            FROM communication_messages message
            WHERE message_id = ANY($1)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            "#,
        )
        .bind(&message_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            let mut message = row_to_matched_message(row)?;
            message.review_state = reviewed_by_id
                .get(&message.message_id)
                .copied()
                .unwrap_or(ProjectLinkReviewState::Suggested);
            messages.push(message);
        }

        Ok(messages)
    }

    pub(crate) async fn matching_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMatchedDocument>, ProjectStoreError> {
        let reviewed = self.active_project_documents(project_id).await?;
        if reviewed.is_empty() {
            return Ok(Vec::new());
        }
        let (document_ids, reviewed_by_id) = reviewed_targets_and_map(reviewed);

        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, source_fingerprint, imported_at
            FROM documents document
            WHERE document_id = ANY($1)
            ORDER BY imported_at DESC, document_id
            "#,
        )
        .bind(&document_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::with_capacity(rows.len());
        for row in rows {
            let mut document = row_to_matched_document(row)?;
            document.review_state = reviewed_by_id
                .get(&document.document_id)
                .copied()
                .unwrap_or(ProjectLinkReviewState::Suggested);
            documents.push(document);
        }

        Ok(documents)
    }
}

pub(super) fn reviewed_targets_and_map(
    targets: Vec<ProjectReviewedTarget>,
) -> (Vec<String>, HashMap<String, ProjectLinkReviewState>) {
    let mut ids = Vec::with_capacity(targets.len());
    let mut map = HashMap::with_capacity(targets.len());
    for target in targets {
        map.insert(target.target_id.clone(), target.review_state);
        ids.push(target.target_id);
    }

    (ids, map)
}

pub(super) fn reviewed_target_ids(targets: &[ProjectReviewedTarget]) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.target_id.clone())
        .collect()
}
