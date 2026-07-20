//! Validated desired topology for one non-destructive Event Hub reconciliation.

use std::collections::BTreeSet;

use super::{ConsumerSpecV1, StreamSpecV1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHubTopologyPlanV1 {
    streams: Vec<StreamSpecV1>,
    consumers: Vec<ConsumerSpecV1>,
}

impl EventHubTopologyPlanV1 {
    pub fn new(
        mut streams: Vec<StreamSpecV1>,
        mut consumers: Vec<ConsumerSpecV1>,
    ) -> Result<Self, EventHubTopologyPlanViolationV1> {
        streams.sort_by_key(|stream| stream.kind());
        consumers.sort_by(|left, right| {
            (left.stream_kind().subject_token(), left.durable_name())
                .cmp(&(right.stream_kind().subject_token(), right.durable_name()))
        });
        valid(&streams, &consumers)
            .then_some(Self { streams, consumers })
            .ok_or(EventHubTopologyPlanViolationV1::DuplicateOrUndeclaredConsumer)
    }

    #[must_use]
    pub fn streams(&self) -> &[StreamSpecV1] {
        &self.streams
    }

    #[must_use]
    pub fn consumers(&self) -> &[ConsumerSpecV1] {
        &self.consumers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventHubTopologyPlanViolationV1 {
    DuplicateOrUndeclaredConsumer,
}

fn valid(streams: &[StreamSpecV1], consumers: &[ConsumerSpecV1]) -> bool {
    let kinds = streams
        .iter()
        .map(|stream| stream.kind())
        .collect::<BTreeSet<_>>();
    kinds.len() == streams.len()
        && consumers
            .iter()
            .all(|consumer| kinds.contains(&consumer.stream_kind()))
        && consumers
            .iter()
            .map(|consumer| (consumer.stream_kind(), consumer.durable_name()))
            .collect::<BTreeSet<_>>()
            .len()
            == consumers.len()
}
