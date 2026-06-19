use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_person_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT person_id, display_name, email_address
        FROM persons
        ORDER BY updated_at DESC, person_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let person_id: String = row.try_get("person_id")?;
        let display_name: String = row.try_get("display_name")?;
        let email_address: String = row.try_get("email_address")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Person,
            source_id: person_id,
            observation_id: None,
            title: display_name.clone(),
            source_text: format!("{display_name}\nEmail: {email_address}"),
            graph_node_id: None,
        });
    }

    Ok(())
}
