//! Event Hub topology reconciliation connection.

use std::time::Duration;

use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy, ReplayPolicy, pull};
use async_nats::jetstream::stream::{
    Config as StreamConfig, DiscardPolicy, RetentionPolicy, StorageType,
};

use crate::topology::{ConsumerSpecV1, EventHubTopologyPlanV1, StreamSpecV1};

const DUPLICATE_WINDOW: Duration = Duration::from_secs(120);
const MAX_ENVELOPE_BYTES: i32 = 262_144;

/// Kernel Event Hub administration connection. It never transports owner payloads.
pub struct EventHubJetStreamConnection {
    context: async_nats::jetstream::Context,
}

impl EventHubJetStreamConnection {
    pub(super) const fn new(context: async_nats::jetstream::Context) -> Self {
        Self { context }
    }

    pub async fn reconcile(&self, topology: &EventHubTopologyPlanV1) -> Result<(), String> {
        for stream in topology.streams() {
            self.reconcile_stream(*stream).await?;
        }
        for consumer in topology.consumers() {
            self.reconcile_consumer(consumer).await?;
        }
        Ok(())
    }

    async fn reconcile_stream(&self, specification: StreamSpecV1) -> Result<(), String> {
        let expected = stream_config(specification);
        let stream = self
            .context
            .get_or_create_stream(expected.clone())
            .await
            .map_err(|_| "JetStream stream reconciliation failed".to_owned())?;
        stream_matches(&stream.cached_info().config, &expected)
            .then_some(())
            .ok_or_else(|| {
                "JetStream stream topology conflicts with the declared catalog".to_owned()
            })
    }

    async fn reconcile_consumer(&self, specification: &ConsumerSpecV1) -> Result<(), String> {
        let stream = self
            .context
            .get_stream(specification.stream_kind().stream_name())
            .await
            .map_err(|_| "JetStream consumer stream is unavailable".to_owned())?;
        let expected = consumer_config(specification);
        let consumer = stream
            .get_or_create_consumer(specification.durable_name(), expected.clone())
            .await
            .map_err(|_| "JetStream consumer reconciliation failed".to_owned())?;
        consumer_matches(&consumer.cached_info().config, &expected)
            .then_some(())
            .ok_or_else(|| {
                "JetStream consumer topology conflicts with the declared catalog".to_owned()
            })
    }
}

fn stream_config(specification: StreamSpecV1) -> StreamConfig {
    StreamConfig {
        name: specification.kind().stream_name().to_owned(),
        max_bytes: specification.budget().max_bytes(),
        max_age: specification.budget().max_age(),
        num_replicas: specification.budget().replicas(),
        subjects: vec![specification.kind().stream_subject().to_owned()],
        retention: RetentionPolicy::Limits,
        storage: StorageType::File,
        discard: DiscardPolicy::Old,
        max_consumers: 512,
        max_message_size: MAX_ENVELOPE_BYTES,
        duplicate_window: DUPLICATE_WINDOW,
        deny_delete: true,
        deny_purge: true,
        ..StreamConfig::default()
    }
}

fn consumer_config(specification: &ConsumerSpecV1) -> pull::Config {
    let budget = specification.budget();
    pull::Config {
        durable_name: Some(specification.durable_name().to_owned()),
        deliver_policy: DeliverPolicy::All,
        ack_policy: AckPolicy::Explicit,
        ack_wait: budget.ack_wait(),
        max_deliver: budget.max_deliver(),
        filter_subject: specification.filter_subject().to_owned(),
        max_ack_pending: budget.max_ack_pending(),
        max_batch: budget.max_ack_pending(),
        max_expires: budget.ack_wait(),
        inactive_threshold: Duration::ZERO,
        num_replicas: 1,
        replay_policy: ReplayPolicy::Instant,
        backoff: retry_backoff(budget.ack_wait(), budget.max_deliver()),
        ..pull::Config::default()
    }
}

fn retry_backoff(ack_wait: Duration, max_deliver: i64) -> Vec<Duration> {
    (0..max_deliver)
        .scan(ack_wait, |delay, _| {
            let current = *delay;
            *delay = delay.saturating_mul(2).min(Duration::from_secs(600));
            Some(current)
        })
        .collect()
}

fn stream_matches(actual: &StreamConfig, expected: &StreamConfig) -> bool {
    actual.name == expected.name
        && actual.subjects == expected.subjects
        && actual.max_bytes == expected.max_bytes
        && actual.max_age == expected.max_age
        && actual.num_replicas == expected.num_replicas
        && actual.retention == expected.retention
        && actual.storage == expected.storage
        && actual.discard == expected.discard
        && actual.max_consumers == expected.max_consumers
        && actual.max_message_size == expected.max_message_size
        && actual.duplicate_window == expected.duplicate_window
        && actual.deny_delete == expected.deny_delete
        && actual.deny_purge == expected.deny_purge
}

fn consumer_matches(
    actual: &async_nats::jetstream::consumer::Config,
    expected: &pull::Config,
) -> bool {
    actual.durable_name == expected.durable_name
        && actual.deliver_policy == expected.deliver_policy
        && actual.ack_policy == expected.ack_policy
        && actual.ack_wait == expected.ack_wait
        && actual.max_deliver == expected.max_deliver
        && actual.filter_subject == expected.filter_subject
        && actual.max_ack_pending == expected.max_ack_pending
        && actual.max_batch == expected.max_batch
        && actual.max_expires == expected.max_expires
        && actual.num_replicas == expected.num_replicas
        && actual.replay_policy == expected.replay_policy
        && actual.backoff == expected.backoff
}
