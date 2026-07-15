use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ProjectStoreError;
use super::models::{
    Project, ProjectDocumentSummary, ProjectMessageSummary, ProjectPersonaSummary,
    ProjectTimelineItem,
};

pub(super) fn row_to_project(row: PgRow) -> Result<Project, ProjectStoreError> {
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

pub(super) fn row_to_project_message(
    row: PgRow,
) -> Result<ProjectMessageSummary, ProjectStoreError> {
    Ok(ProjectMessageSummary {
        message_id: row.try_get("message_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

pub(super) fn row_to_project_document(
    row: PgRow,
) -> Result<ProjectDocumentSummary, ProjectStoreError> {
    Ok(ProjectDocumentSummary {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        observation_id: row.try_get("observation_id")?,
        imported_at: row.try_get("imported_at")?,
    })
}

pub(super) fn row_to_project_persona(
    row: PgRow,
) -> Result<ProjectPersonaSummary, ProjectStoreError> {
    Ok(ProjectPersonaSummary {
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        interaction_count: row.try_get("interaction_count")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
    })
}

pub(super) fn row_to_timeline_item(row: PgRow) -> Result<ProjectTimelineItem, ProjectStoreError> {
    Ok(ProjectTimelineItem {
        item_kind: row.try_get("item_kind")?,
        item_id: row.try_get("item_id")?,
        title: row.try_get("title")?,
        subtitle: row.try_get("subtitle")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}
