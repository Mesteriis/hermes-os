use super::super::errors::ProjectStoreError;
use super::super::models::ProjectPersonaSummary;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_project_persona;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_personas(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectPersonaSummary>, ProjectStoreError> {
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
            LEFT JOIN personas person ON person.email_address = participants.email_address
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

        rows.into_iter().map(row_to_project_persona).collect()
    }
}
