use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::platform::events::{EventEnvelope, EventStoreError};

pub(crate) async fn dispatch_ai_runtime_signal(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<Option<EventEnvelope>, EventStoreError> {
    crate::domains::signal_hub::dispatch_ai_helper_signal(
        pool,
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )
    .await
    .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}
