use sqlx::postgres::PgPool;

use super::constants::{DEFAULT_PROJECT_LIMIT, PROJECT_DETAIL_ITEM_LIMIT};
use super::errors::ProjectStoreError;
use super::ids::project_graph_node_id;
use super::models::{NewProject, Project, ProjectDetail, ProjectSummary};
use super::rows::row_to_project;
use super::validation::{validate_limit, validate_non_empty};

#[derive(Clone)]
pub struct ProjectStore {
    pub(super) pool: PgPool,
}

impl ProjectStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_project(&self, project: &NewProject) -> Result<Project, ProjectStoreError> {
        let project = project.validate()?;
        let mut transaction = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO projects (
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (project_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                kind = EXCLUDED.kind,
                status = EXCLUDED.status,
                description = EXCLUDED.description,
                owner_display_name = EXCLUDED.owner_display_name,
                progress_percent = EXCLUDED.progress_percent,
                start_date = EXCLUDED.start_date,
                target_date = EXCLUDED.target_date,
                updated_at = now()
            RETURNING
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
            "#,
        )
        .bind(&project.project_id)
        .bind(&project.name)
        .bind(&project.kind)
        .bind(&project.status)
        .bind(&project.description)
        .bind(&project.owner_display_name)
        .bind(project.progress_percent)
        .bind(project.start_date)
        .bind(project.target_date)
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query("DELETE FROM project_keywords WHERE project_id = $1")
            .bind(&project.project_id)
            .execute(&mut *transaction)
            .await?;

        for keyword in &project.keywords {
            sqlx::query(
                r#"
                INSERT INTO project_keywords (project_id, keyword)
                VALUES ($1, $2)
                ON CONFLICT (project_id, keyword) DO NOTHING
                "#,
            )
            .bind(&project.project_id)
            .bind(keyword)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        row_to_project(row)
    }

    pub async fn list_projects(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<ProjectSummary>, ProjectStoreError> {
        let limit = validate_limit(limit.unwrap_or(DEFAULT_PROJECT_LIMIT))?;
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
            ORDER BY updated_at DESC, name, project_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let projects = rows
            .into_iter()
            .map(row_to_project)
            .collect::<Result<Vec<_>, _>>()?;
        let mut summaries = Vec::with_capacity(projects.len());
        for project in projects {
            summaries.push(ProjectSummary {
                graph_node_id: project_graph_node_id(&project.project_id),
                stats: self.project_stats(&project.project_id).await?,
                project,
            });
        }

        Ok(summaries)
    }

    pub async fn project_detail(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectDetail>, ProjectStoreError> {
        let project_id = validate_non_empty("project_id", project_id)?;
        let Some(project) = self.project_by_id(&project_id).await? else {
            return Ok(None);
        };

        Ok(Some(ProjectDetail {
            graph_node_id: project_graph_node_id(&project.project_id),
            stats: self.project_stats(&project.project_id).await?,
            timeline: self
                .project_timeline(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            key_people: self
                .project_people(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            recent_messages: self
                .project_messages(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            documents: self
                .project_documents(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            project,
        }))
    }
}
