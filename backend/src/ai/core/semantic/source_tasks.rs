use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_task_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT task_id, title, source_kind, source_id, status
        FROM tasks
        ORDER BY updated_at DESC, task_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let task_id: String = row.try_get("task_id")?;
        let title: String = row.try_get("title")?;
        let source_kind: String = row.try_get("source_kind")?;
        let source_id: String = row.try_get("source_id")?;
        let status: String = row.try_get("status")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Task,
            source_id: task_id,
            title: title.clone(),
            source_text: format!("{title}\nStatus: {status}\nSource: {source_kind}:{source_id}"),
            graph_node_id: None,
        });
    }

    Ok(())
}
