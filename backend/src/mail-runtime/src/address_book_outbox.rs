//! Exact-byte Mail address-book terminal result relay.

use hermes_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use hermes_mail_address_book_persistence::{
    MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookOutboxRelayErrorV1 {
    Persistence,
    Unavailable,
}

pub async fn relay_mail_address_book_outbox_once_v1(
    persistence: &MailAddressBookPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, MailAddressBookOutboxRelayErrorV1> {
    let records = persistence
        .pending_results(64)
        .await
        .map_err(persistence_error)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| MailAddressBookOutboxRelayErrorV1::Unavailable)?;
        persistence
            .mark_result_published(*record.message_id(), published_at_unix_seconds)
            .await
            .map_err(persistence_error)?;
        published += 1;
    }
    Ok(published)
}

fn persistence_error(_: MailAddressBookPersistenceErrorV1) -> MailAddressBookOutboxRelayErrorV1 {
    MailAddressBookOutboxRelayErrorV1::Persistence
}
