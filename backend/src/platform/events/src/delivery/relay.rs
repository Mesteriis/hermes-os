//! Owner-local outbox relay contract with publish-before-ack ordering.

use super::OutboxRecordV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboxEntryV1 {
    outbox_id: String,
    record: OutboxRecordV1,
}

impl OutboxEntryV1 {
    pub fn new(
        outbox_id: impl Into<String>,
        record: OutboxRecordV1,
    ) -> Result<Self, OutboxRelayErrorV1> {
        let outbox_id = outbox_id.into();
        valid_id(&outbox_id)
            .then_some(Self { outbox_id, record })
            .ok_or(OutboxRelayErrorV1::InvalidEntry)
    }

    #[must_use]
    pub fn outbox_id(&self) -> &str {
        &self.outbox_id
    }

    #[must_use]
    pub fn record(&self) -> &OutboxRecordV1 {
        &self.record
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboxPublishReceiptV1 {
    stream: String,
    sequence: u64,
    duplicate: bool,
}

impl OutboxPublishReceiptV1 {
    pub fn new(
        stream: impl Into<String>,
        sequence: u64,
        duplicate: bool,
    ) -> Result<Self, OutboxRelayErrorV1> {
        let stream = stream.into();
        (valid_stream(&stream) && sequence > 0)
            .then_some(Self {
                stream,
                sequence,
                duplicate,
            })
            .ok_or(OutboxRelayErrorV1::InvalidReceipt)
    }

    #[must_use]
    pub fn stream(&self) -> &str {
        &self.stream
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub const fn duplicate(&self) -> bool {
        self.duplicate
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutboxRelayOutcomeV1 {
    Idle,
    Published { outbox_id: String, duplicate: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutboxRelayErrorV1 {
    InvalidEntry,
    InvalidReceipt,
    Persistence,
    PublisherUnavailable,
}

/// Implemented only by an owner's PostgreSQL outbox adapter.
pub trait OwnerOutboxStorePortV1 {
    fn next_pending(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<OutboxEntryV1>, OutboxRelayErrorV1>> + Send;

    fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        receipt: &OutboxPublishReceiptV1,
    ) -> impl std::future::Future<Output = Result<(), OutboxRelayErrorV1>> + Send;
}

/// Implemented by the selected NATS transport for one fenced module runtime.
pub trait ExactOutboxPublisherPortV1 {
    fn publish_exact(
        &self,
        record: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send;
}

/// Publishes immutable outbox bytes before changing owner-local durable state.
pub async fn relay_once<S, P>(
    store: &mut S,
    publisher: &P,
) -> Result<OutboxRelayOutcomeV1, OutboxRelayErrorV1>
where
    S: OwnerOutboxStorePortV1,
    P: ExactOutboxPublisherPortV1,
{
    let Some(entry) = store.next_pending().await? else {
        return Ok(OutboxRelayOutcomeV1::Idle);
    };
    let receipt = publisher.publish_exact(entry.record()).await?;
    store.mark_published(&entry, &receipt).await?;
    Ok(OutboxRelayOutcomeV1::Published {
        outbox_id: entry.outbox_id,
        duplicate: receipt.duplicate,
    })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_stream(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}
