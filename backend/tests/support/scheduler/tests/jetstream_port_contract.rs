use hermes_events_protocol::RuntimeNatsJwtCredentialV1;
use hermes_runtime_protocol::v1::{
    SchedulerRuntimeReceiptConsumerBindingV1, SchedulerRuntimeReceiptKindV1,
};
use hermes_scheduler_jetstream::{
    SchedulerJetStreamReceiptPortErrorV1, SchedulerJetStreamReceiptPortV1,
};
use nats_jwt::KeyPair;

#[tokio::test]
async fn scheduler_jetstream_port_rejects_an_expired_jwt_before_connecting() {
    let result = SchedulerJetStreamReceiptPortV1::connect(
        "nats://127.0.0.1:4222",
        credential(1),
        &binding(),
    )
    .await;

    assert!(matches!(
        result,
        Err(SchedulerJetStreamReceiptPortErrorV1::ExpiredCredential)
    ));
}

#[tokio::test]
async fn scheduler_jetstream_port_rejects_a_binding_before_connecting() {
    let mut invalid = binding();
    invalid.filter_subject = "hermes.ack.v1.>".to_owned();
    let result = SchedulerJetStreamReceiptPortV1::connect(
        "nats://127.0.0.1:4222",
        credential(u64::MAX),
        &invalid,
    )
    .await;

    assert!(matches!(
        result,
        Err(SchedulerJetStreamReceiptPortErrorV1::InvalidBinding)
    ));
}

fn credential(expires_at_unix_seconds: u64) -> RuntimeNatsJwtCredentialV1 {
    let key = KeyPair::new_user();
    RuntimeNatsJwtCredentialV1::new(
        "test-jwt".to_owned(),
        key.seed().expect("user seed"),
        key.public_key(),
        expires_at_unix_seconds,
    )
    .expect("runtime credential")
}

fn binding() -> SchedulerRuntimeReceiptConsumerBindingV1 {
    SchedulerRuntimeReceiptConsumerBindingV1 {
        kind: SchedulerRuntimeReceiptKindV1::Acceptance as i32,
        stream_name: "HERMES_ACK_V1".to_owned(),
        durable_name: "scheduler_receipt_acceptance".to_owned(),
        filter_subject: "hermes.ack.v1.mail.job_receipt.v1".to_owned(),
        ack_wait_millis: 30_000,
        max_deliver: 8,
        max_ack_pending: 32,
    }
}
