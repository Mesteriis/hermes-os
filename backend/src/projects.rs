use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::graph::{GraphNodeKind, node_id};
use crate::project_link_reviews::{
    ProjectLinkReviewState, ProjectLinkReviewStore, ProjectReviewedTarget,
};

const DEFAULT_PROJECT_LIMIT: i64 = 25;
const MAX_PROJECT_LIMIT: i64 = 100;
const PROJECT_DETAIL_ITEM_LIMIT: i64 = 8;

#[derive(Clone)]
pub struct ProjectStore {
    pool: PgPool,
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
            let project = row_to_project(row)?;
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

    async fn project_by_id(&self, project_id: &str) -> Result<Option<Project>, ProjectStoreError> {
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

    async fn project_keywords(&self, project_id: &str) -> Result<Vec<String>, ProjectStoreError> {
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

    async fn project_stats(&self, project_id: &str) -> Result<ProjectStats, ProjectStoreError> {
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

    async fn project_messages(
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

    async fn project_documents(
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

    async fn project_people(
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

    async fn project_timeline(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectTimelineItem>, ProjectStoreError> {
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

    async fn active_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_message_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }

    async fn active_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_document_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProject {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub owner_display_name: String,
    pub progress_percent: i32,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub keywords: Vec<String>,
}

impl NewProject {
    pub fn active(
        project_id: impl Into<String>,
        name: impl Into<String>,
        kind: impl Into<String>,
        description: impl Into<String>,
        owner_display_name: impl Into<String>,
        keywords: Vec<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            name: name.into(),
            kind: kind.into(),
            status: "active".to_owned(),
            description: description.into(),
            owner_display_name: owner_display_name.into(),
            progress_percent: 0,
            start_date: None,
            target_date: None,
            keywords,
        }
    }

    pub fn progress(mut self, progress_percent: i32) -> Self {
        self.progress_percent = progress_percent;
        self
    }

    fn validate(&self) -> Result<ValidatedProject, ProjectStoreError> {
        let project_id = validate_non_empty("project_id", &self.project_id)?;
        let name = validate_non_empty("name", &self.name)?;
        let kind = validate_non_empty("kind", &self.kind)?;
        let status = validate_non_empty("status", &self.status)?;
        let description = validate_non_empty("description", &self.description)?;
        let owner_display_name =
            validate_non_empty("owner_display_name", &self.owner_display_name)?;
        if !(0..=100).contains(&self.progress_percent) {
            return Err(ProjectStoreError::InvalidProgress(self.progress_percent));
        }

        let mut seen = BTreeSet::new();
        let mut keywords = Vec::new();
        for keyword in &self.keywords {
            let keyword = validate_non_empty("keyword", keyword)?;
            if seen.insert(keyword.to_ascii_lowercase()) {
                keywords.push(keyword);
            }
        }
        if keywords.is_empty() {
            return Err(ProjectStoreError::NoKeywords);
        }

        Ok(ValidatedProject {
            project_id,
            name,
            kind,
            status,
            description,
            owner_display_name,
            progress_percent: self.progress_percent,
            start_date: self.start_date,
            target_date: self.target_date,
            keywords,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedProject {
    project_id: String,
    name: String,
    kind: String,
    status: String,
    description: String,
    owner_display_name: String,
    progress_percent: i32,
    start_date: Option<NaiveDate>,
    target_date: Option<NaiveDate>,
    keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub owner_display_name: String,
    pub progress_percent: i32,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectStats {
    pub message_count: i64,
    pub document_count: i64,
    pub people_count: i64,
    pub graph_connection_count: i64,
    pub latest_activity_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectSummary {
    pub project: Project,
    pub stats: ProjectStats,
    pub graph_node_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectDetail {
    pub project: Project,
    pub stats: ProjectStats,
    pub graph_node_id: String,
    pub timeline: Vec<ProjectTimelineItem>,
    pub key_people: Vec<ProjectPersonSummary>,
    pub recent_messages: Vec<ProjectMessageSummary>,
    pub documents: Vec<ProjectDocumentSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectTimelineItem {
    pub item_kind: String,
    pub item_id: String,
    pub title: String,
    pub subtitle: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectPersonSummary {
    pub display_name: String,
    pub email_address: String,
    pub interaction_count: i64,
    pub last_interaction_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectMessageSummary {
    pub message_id: String,
    pub subject: String,
    pub sender: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectDocumentSummary {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectListResponse {
    pub items: Vec<ProjectSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectProjectionSource {
    pub project: Project,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMatchedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMatchedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub source_fingerprint: String,
    pub imported_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}

#[derive(Debug, Error)]
pub enum ProjectStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("project progress_percent must be between 0 and 100: {0}")]
    InvalidProgress(i32),

    #[error("project must have at least one keyword")]
    NoKeywords,

    #[error(transparent)]
    ProjectLinkReview(#[from] crate::project_link_reviews::ProjectLinkReviewError),

    #[error("project limit must be positive")]
    InvalidLimit,

    #[error("project message recipients must be a JSON array of strings")]
    InvalidRecipients,
}

pub fn project_graph_node_id(project_id: &str) -> String {
    node_id(GraphNodeKind::Project, project_id)
}

fn row_to_project(row: PgRow) -> Result<Project, ProjectStoreError> {
    Ok(Project {
        project_id: row.try_get("project_id")?,
        name: row.try_get("name")?,
        kind: row.try_get("kind")?,
        status: row.try_get("status")?,
        description: row.try_get("description")?,
        owner_display_name: row.try_get("owner_display_name")?,
        progress_percent: row.try_get("progress_percent")?,
        start_date: row.try_get("start_date")?,
        target_date: row.try_get("target_date")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_project_message(row: PgRow) -> Result<ProjectMessageSummary, ProjectStoreError> {
    Ok(ProjectMessageSummary {
        message_id: row.try_get("message_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

fn row_to_project_document(row: PgRow) -> Result<ProjectDocumentSummary, ProjectStoreError> {
    Ok(ProjectDocumentSummary {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        imported_at: row.try_get("imported_at")?,
    })
}

fn row_to_project_person(row: PgRow) -> Result<ProjectPersonSummary, ProjectStoreError> {
    Ok(ProjectPersonSummary {
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        interaction_count: row.try_get("interaction_count")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
    })
}

fn row_to_timeline_item(row: PgRow) -> Result<ProjectTimelineItem, ProjectStoreError> {
    Ok(ProjectTimelineItem {
        item_kind: row.try_get("item_kind")?,
        item_id: row.try_get("item_id")?,
        title: row.try_get("title")?,
        subtitle: row.try_get("subtitle")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

fn row_to_matched_message(row: PgRow) -> Result<ProjectMatchedMessage, ProjectStoreError> {
    Ok(ProjectMatchedMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        review_state: ProjectLinkReviewState::Suggested,
    })
}

fn row_to_matched_document(row: PgRow) -> Result<ProjectMatchedDocument, ProjectStoreError> {
    Ok(ProjectMatchedDocument {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        imported_at: row.try_get("imported_at")?,
        review_state: ProjectLinkReviewState::Suggested,
    })
}

fn reviewed_targets_and_map(
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

fn reviewed_target_ids(targets: &[ProjectReviewedTarget]) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.target_id.clone())
        .collect()
}

fn recipients_from_value(value: serde_json::Value) -> Result<Vec<String>, ProjectStoreError> {
    let Some(values) = value.as_array() else {
        return Err(ProjectStoreError::InvalidRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(ProjectStoreError::InvalidRecipients)
        })
        .collect()
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<String, ProjectStoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::EmptyField(field_name));
    }

    Ok(trimmed.to_owned())
}

fn validate_limit(limit: i64) -> Result<i64, ProjectStoreError> {
    if limit <= 0 {
        return Err(ProjectStoreError::InvalidLimit);
    }

    Ok(limit.min(MAX_PROJECT_LIMIT))
}
