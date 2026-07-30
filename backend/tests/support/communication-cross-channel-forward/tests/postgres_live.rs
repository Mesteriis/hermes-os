//! Disposable PostgreSQL proof for the cross-channel forward durable lifecycle.

use hermes_communication_cross_channel_forward_core::{
    CrossChannelForwardDraftV1, CrossChannelForwardStateV1,
};
use hermes_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CreateCrossChannelForwardOutcomeV1,
    CreateCrossChannelForwardV1, CrossChannelForwardCleanupReasonV1,
    CrossChannelForwardPersistenceConformanceV1, CrossChannelForwardPersistenceErrorV1,
    CrossChannelForwardPreparedSourceV1, CrossChannelForwardWorkStageV1,
};

const POSTGRES_URL: &str = "HERMES_COMMUNICATION_CROSS_CHANNEL_FORWARD_POSTGRES_URL";
const OWNER: &str = "owner-1";

#[tokio::test]
#[ignore = "requires the disposable cross-channel forward PostgreSQL contour"]
async fn durable_forward_survives_reconnect_and_fences_conflicts_claims_and_cleanup() {
    let database_url = required(POSTGRES_URL);
    let persistence = connect(&database_url).await;
    CrossChannelForwardPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install cross-channel forward schema");
    persistence
        .verify_storage_ready()
        .await
        .expect("verify cross-channel forward storage");

    let create = create_command(1, 2, 3);
    assert_eq!(
        persistence.create_forward(create.clone()).await,
        Ok(CreateCrossChannelForwardOutcomeV1::Created { state_revision: 1 })
    );
    assert_eq!(
        persistence.create_forward(create).await,
        Ok(CreateCrossChannelForwardOutcomeV1::Existing { state_revision: 1 })
    );
    let conflicting = create_command(1, 9, 3);
    assert_eq!(
        persistence.create_forward(conflicting).await,
        Err(CrossChannelForwardPersistenceErrorV1::Conflict)
    );

    let preparing = persistence
        .claim_next_forward(OWNER, "worker-1", 1_100)
        .await
        .expect("claim source preparation")
        .expect("forward must be due");
    assert_eq!(
        preparing.stage,
        CrossChannelForwardWorkStageV1::PreparingSource
    );
    assert_eq!(preparing.prepared_source, None);
    let prepared_source = CrossChannelForwardPreparedSourceV1 {
        source_revision: 7,
        body_sha256: [5; 32],
        body_length: 12,
        blob_reference: vec![6; 16],
        custody_proof: vec![7; 32],
    };
    persistence
        .record_prepared_source(&preparing, &prepared_source, 1_200)
        .await
        .expect("persist prepared source without plaintext body");

    let prepared_claim = persistence
        .claim_next_forward(OWNER, "worker-2", 1_200)
        .await
        .expect("claim prepared source")
        .expect("prepared forward must be due");
    assert_eq!(
        prepared_claim.prepared_source,
        Some(prepared_source.clone())
    );
    persistence
        .reschedule_claim(&prepared_claim, 1_500, 1_250)
        .await
        .expect("persist dependency outage retry");
    drop(persistence);

    let reopened = connect(&database_url).await;
    assert_eq!(
        reopened
            .claim_next_forward(OWNER, "worker-3", 1_499)
            .await
            .expect("query before retry deadline"),
        None
    );
    let retried = reopened
        .claim_next_forward(OWNER, "worker-3", 1_500)
        .await
        .expect("claim after reconnect")
        .expect("retry must survive reconnect");
    assert_eq!(retried.attempt_count, 1);
    let dispatching = reopened
        .begin_dispatch(&retried, 1_600)
        .await
        .expect("enter durable dispatch state");
    assert_eq!(
        dispatching.stage,
        CrossChannelForwardWorkStageV1::Dispatching
    );
    let mut stale_dispatching = dispatching.clone();
    stale_dispatching.claim_epoch -= 1;
    assert_eq!(
        reopened
            .mark_delivery_accepted(&stale_dispatching, [8; 16], 1_700)
            .await,
        Err(CrossChannelForwardPersistenceErrorV1::ClaimLost)
    );
    reopened
        .mark_delivery_accepted(&dispatching, [8; 16], 1_700)
        .await
        .expect("persist downstream acceptance");

    let status = reopened
        .status(OWNER, &[1; 16])
        .await
        .expect("read terminal status");
    assert_eq!(status.state, CrossChannelForwardStateV1::DeliveryAccepted);
    assert_eq!(status.state_revision, 4);
    assert_eq!(status.delivery_intent_id, Some([8; 16]));
    assert_eq!(status.error_code, None);
    let transitions = reopened
        .client_realtime_window(OWNER, None, 16)
        .await
        .expect("replay client-safe state");
    assert_eq!(
        transitions
            .iter()
            .map(|transition| transition.state)
            .collect::<Vec<_>>(),
        vec![
            CrossChannelForwardStateV1::Accepted,
            CrossChannelForwardStateV1::PreparingSource,
            CrossChannelForwardStateV1::Dispatching,
            CrossChannelForwardStateV1::DeliveryAccepted,
        ]
    );

    let cleanup = reopened
        .next_cleanup(OWNER, 1_700)
        .await
        .expect("read cleanup queue")
        .expect("terminal source custody must be queued");
    assert_eq!(cleanup.forward_id, [1; 16]);
    assert_eq!(
        cleanup.reason,
        CrossChannelForwardCleanupReasonV1::DeliveryAccepted
    );
    reopened
        .reschedule_cleanup(OWNER, &[1; 16], 0, 2_000, 1_800)
        .await
        .expect("persist cleanup outage");
    drop(reopened);

    let after_cleanup_restart = connect(&database_url).await;
    assert_eq!(
        after_cleanup_restart
            .next_cleanup(OWNER, 1_999)
            .await
            .expect("cleanup before retry deadline"),
        None
    );
    let cleanup = after_cleanup_restart
        .next_cleanup(OWNER, 2_000)
        .await
        .expect("cleanup after retry deadline")
        .expect("cleanup retry survives reconnect");
    assert_eq!(cleanup.attempt_count, 1);
    after_cleanup_restart
        .complete_cleanup(OWNER, &[1; 16], 2_100)
        .await
        .expect("complete custody release");
    assert_eq!(
        after_cleanup_restart
            .next_cleanup(OWNER, 2_100)
            .await
            .expect("completed queue"),
        None
    );
    assert_eq!(
        after_cleanup_restart.status("owner-2", &[1; 16]).await,
        Err(CrossChannelForwardPersistenceErrorV1::NotFound)
    );
}

fn create_command(
    forward_id: u8,
    source_message_id: u8,
    target_conversation_id: u8,
) -> CreateCrossChannelForwardV1 {
    CreateCrossChannelForwardV1 {
        logical_owner_id: OWNER.to_owned(),
        draft: CrossChannelForwardDraftV1 {
            forward_operation_id: [forward_id; 16],
            source_message_id: [source_message_id; 16],
            target_conversation_id: [target_conversation_id; 16],
            target_reply_to_message_id: Some([4; 16]),
        },
        created_at_unix_millis: 1_000,
    }
}

async fn connect(database_url: &str) -> CommunicationCrossChannelForwardPersistenceV1 {
    CrossChannelForwardPersistenceConformanceV1::connect_url(database_url)
        .await
        .expect("connect cross-channel forward persistence")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}
