use crate::domains::projects::link_reviews::models::ProjectReviewedTarget;
use crate::domains::projects::link_reviews::store::ProjectLinkReviewStore;

use super::super::errors::ProjectStoreError;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn active_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_message_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }

    pub(in crate::domains::projects::core) async fn active_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_document_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }
}
