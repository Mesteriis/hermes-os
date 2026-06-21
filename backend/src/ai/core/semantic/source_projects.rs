use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::platform::graph::{GraphNodeKind, node_id};

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_project_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.project_id,
            p.name,
            p.kind,
            p.status,
            p.description,
            p.owner_display_name,
            COALESCE(string_agg(k.keyword, ', ' ORDER BY k.keyword), '') AS keywords
        FROM projects p
        LEFT JOIN project_keywords k ON k.project_id = p.project_id
        GROUP BY p.project_id
        ORDER BY p.updated_at DESC, p.project_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let project_id: String = row.try_get("project_id")?;
        let name: String = row.try_get("name")?;
        let kind: String = row.try_get("kind")?;
        let status: String = row.try_get("status")?;
        let description: String = row.try_get("description")?;
        let owner: String = row.try_get("owner_display_name")?;
        let keywords: String = row.try_get("keywords")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Project,
            source_id: project_id.clone(),
            observation_id: None,
            title: name.clone(),
            source_text: format!(
                "{name}\nKind: {kind}\nStatus: {status}\nOwner: {owner}\nKeywords: {keywords}\n\n{description}"
            ),
            graph_node_id: Some(node_id(GraphNodeKind::Project, &project_id)),
        });
    }

    Ok(())
}
