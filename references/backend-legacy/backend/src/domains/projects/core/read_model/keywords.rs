use super::super::errors::ProjectStoreError;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_keywords(
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
}
