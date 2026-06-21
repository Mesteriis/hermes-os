use super::super::errors::ProjectStoreError;
use super::super::models::Project;
use super::super::rows::row_to_project;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_by_id(
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
}
