use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_persona_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT persona_id, display_name, email_address
        FROM personas
        ORDER BY updated_at DESC, persona_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let persona_id: String = row.try_get("persona_id")?;
        let display_name: String = row.try_get("display_name")?;
        let email_address: Option<String> = row.try_get("email_address")?;
        let source_text = if let Some(email_address) = email_address {
            format!("{display_name}\nEmail: {email_address}")
        } else {
            display_name.clone()
        };
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Persona,
            source_id: persona_id,
            observation_id: None,
            title: display_name.clone(),
            source_text,
            graph_node_id: None,
        });
    }

    Ok(())
}
