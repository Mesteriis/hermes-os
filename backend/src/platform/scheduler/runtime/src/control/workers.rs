//! Scheduler receipt and durable-dispatch worker lifecycle.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use hermes_clock_protocol::{ClockDiscontinuityV1, ClockPolicyV1};
use hermes_scheduler_jetstream::{
    SchedulerJetStreamDispatchPortV1, SchedulerJetStreamReceiptPortV1,
};
use hermes_scheduler_persistence::{
    SchedulerDispatchAdmissionV1, SchedulerMaterializationSourceV1, SchedulerPostgresStoreV1,
    SchedulerReceiptConsumerV1,
};

use super::clock::SchedulerSystemClockV1;

pub(super) struct SchedulerWorkerLaunchInputV1<'a> {
    pub(super) runtime: &'a tokio::runtime::Runtime,
    pub(super) store: SchedulerPostgresStoreV1,
    pub(super) dispatch: SchedulerJetStreamDispatchPortV1,
    pub(super) ports: Vec<SchedulerJetStreamReceiptPortV1>,
    pub(super) dispatch_batch_limit: u32,
    pub(super) reconcile_interval_millis: u32,
    pub(super) source: SchedulerMaterializationSourceV1,
    pub(super) admission: SchedulerDispatchAdmissionV1,
}

pub(super) fn launch_workers(input: SchedulerWorkerLaunchInputV1<'_>) -> Receiver<()> {
    let SchedulerWorkerLaunchInputV1 {
        runtime,
        store,
        dispatch,
        ports,
        dispatch_batch_limit,
        reconcile_interval_millis,
        source,
        admission,
    } = input;
    let (sender, receiver) = channel();
    for port in ports {
        let sender = sender.clone();
        let store = store.clone();
        runtime.spawn(async move { receive_receipts(port, store, sender).await });
    }
    let sender = sender.clone();
    runtime.spawn(async move {
        relay_dispatches(
            store,
            dispatch,
            dispatch_batch_limit,
            reconcile_interval_millis,
            source,
            admission,
            sender,
        )
        .await;
    });
    receiver
}

async fn relay_dispatches(
    mut store: SchedulerPostgresStoreV1,
    dispatch: SchedulerJetStreamDispatchPortV1,
    dispatch_batch_limit: u32,
    reconcile_interval_millis: u32,
    source: SchedulerMaterializationSourceV1,
    admission: SchedulerDispatchAdmissionV1,
    failure: Sender<()>,
) {
    let clock = SchedulerSystemClockV1::new(ClockPolicyV1::production_default());
    let mut interval =
        tokio::time::interval(Duration::from_millis(u64::from(reconcile_interval_millis)));
    loop {
        interval.tick().await;
        let reading = match clock.read() {
            Ok(reading) if reading.discontinuity() == ClockDiscontinuityV1::Stable => reading,
            _ => {
                let _ = failure.send(());
                return;
            }
        };
        if store
            .materialize_due(
                reading.wall_utc(),
                u16::try_from(dispatch_batch_limit).unwrap_or(u16::MAX),
                &source,
                &admission,
            )
            .await
            .is_err()
        {
            let _ = failure.send(());
            return;
        }
        if store
            .materialize_retries(
                reading.wall_utc(),
                u16::try_from(dispatch_batch_limit).unwrap_or(u16::MAX),
                &source,
                &admission,
            )
            .await
            .is_err()
        {
            let _ = failure.send(());
            return;
        }
        for _ in 0..dispatch_batch_limit {
            match dispatch.relay_once(&mut store).await {
                Ok(true) => {}
                Ok(false) => break,
                Err(_) => {
                    let _ = failure.send(());
                    return;
                }
            }
        }
    }
}

async fn receive_receipts(
    port: SchedulerJetStreamReceiptPortV1,
    store: SchedulerPostgresStoreV1,
    failure: Sender<()>,
) {
    let mut consumer = SchedulerReceiptConsumerV1::new(port, &store);
    loop {
        if consumer.consume_one().await.is_err() {
            let _ = failure.send(());
            return;
        }
    }
}
