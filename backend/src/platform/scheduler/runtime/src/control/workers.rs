//! Scheduler receipt and durable-dispatch worker lifecycle.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use hermes_clock_protocol::{ClockDiscontinuityV1, ClockPolicyV1};
use hermes_scheduler_jetstream::{
    SchedulerJetStreamDispatchPortV1, SchedulerJetStreamReceiptPortV1,
    SchedulerJetStreamScheduleControlPortV1,
};
use hermes_scheduler_persistence::{
    SchedulerDispatchAdmissionV1, SchedulerMaterializationSourceV1, SchedulerPostgresStoreV1,
    SchedulerReceiptConsumerV1,
};

use super::{
    clock::SchedulerSystemClockV1,
    schedule_control::{SchedulerScheduleControlWorkerConfigV1, run_schedule_control_worker},
};

pub(super) struct SchedulerWorkerLaunchInputV1<'a> {
    pub(super) runtime: &'a tokio::runtime::Runtime,
    pub(super) store: SchedulerPostgresStoreV1,
    pub(super) dispatch: SchedulerJetStreamDispatchPortV1,
    pub(super) ports: Vec<SchedulerJetStreamReceiptPortV1>,
    pub(super) schedule_control: Option<(
        SchedulerJetStreamScheduleControlPortV1,
        SchedulerScheduleControlWorkerConfigV1,
    )>,
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
        schedule_control,
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
    if let Some((port, configuration)) = schedule_control {
        let sender = sender.clone();
        let store = store.clone();
        runtime.spawn(async move {
            run_schedule_control_worker(port, store, configuration, sender).await;
        });
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
                report_failure(&failure, "dispatch_clock");
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
            report_failure(&failure, "materialize_due");
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
            report_failure(&failure, "materialize_retries");
            return;
        }
        for _ in 0..dispatch_batch_limit {
            match dispatch.relay_once(&mut store).await {
                Ok(true) => {}
                Ok(false) => break,
                Err(_) => {
                    report_failure(&failure, "dispatch_relay");
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
            report_failure(&failure, "receipt_consumer");
            return;
        }
    }
}

pub(super) fn report_failure(failure: &Sender<()>, code: &'static str) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_scheduler_worker_failure={code}");
    }
    let _ = failure.send(());
}
