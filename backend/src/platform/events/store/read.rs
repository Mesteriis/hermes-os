use super::EventStore;
use crate::platform::events::errors::EventStoreError;
use crate::platform::events::models::EventEnvelope;
use crate::platform::events::rows::row_to_event;

impl EventStore {
    pub async fn get_by_id(
        &self,
        event_id: &str,
    ) -> Result<Option<EventEnvelope>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_event).transpose()
    }
}
