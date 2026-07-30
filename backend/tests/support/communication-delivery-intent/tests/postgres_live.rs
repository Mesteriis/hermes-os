use hermes_communication_delivery_intent_core::{
    CommunicationConversationIdV1, CommunicationDeliveryRouteV1, CommunicationProviderProvenanceV1,
    CommunicationSourceCursorV1,
};
use hermes_communication_delivery_intent_ingress_api::{
    CommunicationDeliveryIntentIngressEnvelopeContextV1,
    build_communication_delivery_intent_submitted_outbox_record_v1,
    wire::CommunicationDeliveryIntentSubmittedV1,
};
use hermes_communication_delivery_intent_persistence::{
    CommunicationDeliveryIntentPersistenceV1, CreateDeliveryIntentV1,
    DeliveryIntentBodyBlobReceiptV1, DeliveryIntentIngressDispositionV1,
    DeliveryIntentIngressEventV1, DeliveryIntentPersistenceConformanceV1,
    DeliveryIntentPersistenceErrorV1,
};

const POSTGRES_URL: &str = "HERMES_COMMUNICATION_DELIVERY_INTENT_POSTGRES_URL";
const OWNER: &str = "owner-1";

#[tokio::test]
#[ignore = "requires the disposable delivery-intent PostgreSQL contour"]
async fn event_ingress_is_atomic_replay_fenced_and_survives_reconnect() {
    let database_url = required(POSTGRES_URL);
    let persistence = connect(&database_url).await;
    DeliveryIntentPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install delivery-intent schema");
    let event = ingress_event([1; 16], [2; 32]);
    let result = submitted_result(event.command_message_id, event.intent_id);
    assert_eq!(
        persistence.inspect_event_ingress(&event).await,
        Ok(DeliveryIntentIngressDispositionV1::New)
    );
    assert_eq!(
        persistence
            .admit_event_ingress(&event, &create_command(event.intent_id), &result)
            .await,
        Ok(DeliveryIntentIngressDispositionV1::New)
    );
    assert_eq!(
        persistence.inspect_event_ingress(&event).await,
        Ok(DeliveryIntentIngressDispositionV1::ExactDuplicate)
    );
    assert_eq!(
        persistence
            .admit_event_ingress(&event, &create_command(event.intent_id), &result)
            .await,
        Ok(DeliveryIntentIngressDispositionV1::ExactDuplicate)
    );
    assert_eq!(
        persistence
            .pending_ingress_results(16)
            .await
            .expect("pending result"),
        vec![result.clone()]
    );
    let mut conflicting = event.clone();
    conflicting.envelope_sha256 = [9; 32];
    assert_eq!(
        persistence.inspect_event_ingress(&conflicting).await,
        Err(DeliveryIntentPersistenceErrorV1::Conflict)
    );
    drop(persistence);

    let reopened = connect(&database_url).await;
    assert_eq!(
        reopened.inspect_event_ingress(&event).await,
        Ok(DeliveryIntentIngressDispositionV1::ExactDuplicate)
    );
    assert_eq!(
        reopened
            .pending_ingress_results(16)
            .await
            .expect("result after reconnect"),
        vec![result.clone()]
    );
    reopened
        .mark_ingress_result_published(*result.message_id(), 1_800_000_020)
        .await
        .expect("mark result published");
    assert!(
        reopened
            .pending_ingress_results(16)
            .await
            .expect("published result")
            .is_empty()
    );
}

fn ingress_event(
    command_message_id: [u8; 16],
    envelope_sha256: [u8; 32],
) -> DeliveryIntentIngressEventV1 {
    DeliveryIntentIngressEventV1 {
        command_message_id,
        envelope_sha256,
        correlation_id: command_message_id,
        logical_owner_id: OWNER.to_owned(),
        intent_id: command_message_id,
        consumed_at_unix_seconds: 1_800_000_010,
    }
}

fn create_command(intent_id: [u8; 16]) -> CreateDeliveryIntentV1 {
    CreateDeliveryIntentV1 {
        logical_owner_id: OWNER.to_owned(),
        intent_id,
        canonical_conversation_id: CommunicationConversationIdV1::new([3; 16]),
        canonical_reply_message_id: None,
        route: CommunicationDeliveryRouteV1 {
            provider: CommunicationProviderProvenanceV1::MailSmtp,
            account_cursor: CommunicationSourceCursorV1::new([4; 32]),
            conversation_cursor: CommunicationSourceCursorV1::new([5; 32]),
            reply_to_source_cursor: None,
        },
        body_receipt: DeliveryIntentBodyBlobReceiptV1 {
            reference_id: [6; 16],
            declared_bytes: 42,
            sha256: [7; 32],
            custody_transfer_source_proof: vec![8; 64],
        },
        request_fingerprint: [9; 32],
        created_at_unix_seconds: 1_800_000_010,
    }
}

fn submitted_result(
    command_message_id: [u8; 16],
    intent_id: [u8; 16],
) -> hermes_events_protocol::delivery::OutboxRecordV1 {
    build_communication_delivery_intent_submitted_outbox_record_v1(
        command_message_id,
        CommunicationDeliveryIntentSubmittedV1 {
            intent_id: intent_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
        },
        &CommunicationDeliveryIntentIngressEnvelopeContextV1 {
            module_id: "hermes-communication-delivery-intent-runtime".to_owned(),
            runtime_instance_id: "delivery-intent-runtime-1".to_owned(),
            runtime_generation: 3,
            recorded_at_unix_seconds: 1_800_000_010,
            recorded_at_nanos: 0,
        },
    )
    .expect("submitted result")
}

async fn connect(database_url: &str) -> CommunicationDeliveryIntentPersistenceV1 {
    DeliveryIntentPersistenceConformanceV1::connect_url(database_url)
        .await
        .expect("connect delivery-intent persistence")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}
