//! Telegram-owned exact-byte relay for Communications observations.

use hermes_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};

/// Publishes only records already committed in Telegram-owned PostgreSQL.
/// The permit is derived by Kernel from approved Event Hub topology; this
/// integration never constructs subjects or permissions itself.
pub async fn relay_communications_outbox_once(
    durable: &TelegramDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, TelegramCommunicationsOutboxRelayError> {
    let records = durable
        .pending_communications_outbox(64)
        .await
        .map_err(TelegramCommunicationsOutboxRelayError::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| TelegramCommunicationsOutboxRelayError::Unavailable)?;
        durable
            .mark_communications_outbox_published(record.message_id(), published_at_unix_seconds)
            .await
            .map_err(TelegramCommunicationsOutboxRelayError::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Debug)]
pub enum TelegramCommunicationsOutboxRelayError {
    Persistence(TelegramDurablePersistenceError),
    Unavailable,
}
