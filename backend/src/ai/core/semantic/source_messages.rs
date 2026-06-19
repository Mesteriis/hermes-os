use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::domains::graph::core::{GraphNodeKind, node_id};

use super::super::errors::AiError;
use super::super::helpers::recipients_text;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_message_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT message_id, observation_id, subject, sender, recipients, body_text
        FROM communication_messages
        ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let message_id: String = row.try_get("message_id")?;
        let observation_id: String = row.try_get("observation_id")?;
        let subject: String = row.try_get("subject")?;
        let sender: String = row.try_get("sender")?;
        let recipients = recipients_text(row.try_get("recipients")?);
        let body_text: String = row.try_get("body_text")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Message,
            source_id: message_id.clone(),
            observation_id: Some(observation_id),
            title: subject.clone(),
            source_text: format!(
                "Subject: {subject}\nFrom: {sender}\nTo: {recipients}\n\n{body_text}"
            ),
            graph_node_id: Some(node_id(GraphNodeKind::Message, &message_id)),
        });
    }

    Ok(())
}
