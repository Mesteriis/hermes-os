use hermes_communications_persistence::{CommunicationsDurablePersistence, CommunicationsPersistenceError};
use hermes_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

pub async fn relay_domain_outbox_once(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, CommunicationsDomainOutboxRelayErrorV1> {
    let records = persistence
        .pending_domain_outbox(64)
        .await
        .map_err(CommunicationsDomainOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| CommunicationsDomainOutboxRelayErrorV1::Unavailable)?;
        persistence
            .mark_domain_outbox_published(record.message_id(), published_at_unix_seconds)
            .await
            .map_err(CommunicationsDomainOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Debug)]
pub enum CommunicationsDomainOutboxRelayErrorV1 {
    Persistence(CommunicationsPersistenceError),
    Unavailable,
}
