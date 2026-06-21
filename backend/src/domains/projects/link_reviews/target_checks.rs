use sqlx::{Postgres, Transaction};

use super::errors::ProjectLinkReviewError;
use super::models::ProjectLinkTargetKind;
use super::store::ProjectLinkReviewStore;

impl ProjectLinkReviewStore {
    pub(crate) async fn ensure_project_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_id: &str,
    ) -> Result<(), ProjectLinkReviewError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM projects WHERE project_id = $1)",
        )
        .bind(project_id)
        .fetch_one(&mut **transaction)
        .await?;

        if !exists {
            return Err(ProjectLinkReviewError::ProjectNotFound);
        }

        Ok(())
    }

    pub(crate) async fn ensure_target_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<(), ProjectLinkReviewError> {
        let exists =
            match target_kind {
                ProjectLinkTargetKind::Message => sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM communication_messages WHERE message_id = $1)",
                )
                .bind(target_id)
                .fetch_one(&mut **transaction)
                .await?,
                ProjectLinkTargetKind::Document => {
                    sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS (SELECT 1 FROM documents WHERE document_id = $1)",
                    )
                    .bind(target_id)
                    .fetch_one(&mut **transaction)
                    .await?
                }
            };

        if !exists {
            return Err(ProjectLinkReviewError::TargetNotFound);
        }

        Ok(())
    }
}
