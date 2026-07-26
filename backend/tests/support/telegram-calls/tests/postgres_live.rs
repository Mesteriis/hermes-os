use hermes_telegram_calls_core::{
    TelegramCallCommand, TelegramCallDirection, TelegramCallFailureCategory,
    TelegramCallOperationState, TelegramProviderCallState, TelegramProviderCallUpdate,
};
use hermes_telegram_calls_persistence::{TelegramCallsPersistence, TelegramCallsPersistenceError};

const DATABASE_URL_ENV: &str = "HERMES_TELEGRAM_CALLS_POSTGRES_URL";

#[tokio::test]
#[ignore = "requires a disposable PostgreSQL database"]
async fn durable_signaling_is_idempotent_fenced_and_restart_safe() {
    let database_url =
        std::env::var(DATABASE_URL_ENV).expect("Telegram Calls conformance URL must be set");
    let persistence = TelegramCallsPersistence::connect_for_conformance(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    persistence
        .reset_prerequisites_for_conformance()
        .await
        .expect("create exact prerequisite schema");
    persistence
        .apply_schema_for_conformance()
        .await
        .expect("apply call history and signaling migrations");

    let initiate = TelegramCallCommand::InitiateAudio {
        operation_id: "operation-initiate".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        provider_user_id: "901".to_owned(),
    };
    let accepted = persistence
        .accept_call_command(&initiate, Some("900"), 1, 1, 100)
        .await
        .expect("durably accept initiate");
    assert!(!accepted.replayed);
    assert_eq!(
        accepted.operation.state,
        TelegramCallOperationState::Accepted
    );

    let replayed = persistence
        .accept_call_command(&initiate, Some("900"), 1, 1, 100)
        .await
        .expect("replay exact idempotency key");
    assert!(replayed.replayed);
    assert_eq!(replayed.operation, accepted.operation);

    let conflicting = TelegramCallCommand::InitiateAudio {
        operation_id: "operation-initiate".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        provider_user_id: "902".to_owned(),
    };
    assert_eq!(
        persistence
            .accept_call_command(&conflicting, Some("900"), 1, 1, 100)
            .await
            .expect_err("same key with another payload must fail"),
        TelegramCallsPersistenceError::IdempotencyConflict
    );

    let claimed = persistence
        .claim_accepted_call_operations("account-1", 1, 1, 101, 10)
        .await
        .expect("claim accepted operation");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].state, TelegramCallOperationState::Dispatching);
    persistence
        .mark_call_operation_awaiting_provider("account-1", "operation-initiate", Some(77), 102)
        .await
        .expect("persist provider dispatch");

    let provider_update = TelegramProviderCallUpdate {
        account_id: "account-1".to_owned(),
        runtime_generation: 1,
        tdlib_call_id: 77,
        provider_call_unique_id: None,
        provider_user_id: "901".to_owned(),
        direction: TelegramCallDirection::Outgoing,
        state: TelegramProviderCallState::Pending,
        pending_created: true,
        pending_received: false,
        discard_reason: None,
        failure_category: None,
        observed_at_unix_seconds: 103,
    };
    let projected = persistence
        .ingest_provider_update("call-session-1", &provider_update)
        .await
        .expect("project provider call");
    assert!(!projected.replayed);

    let duplicate = persistence
        .ingest_provider_update("different-unused-session", &provider_update)
        .await
        .expect("replay duplicate provider update");
    assert!(duplicate.replayed);
    assert_eq!(duplicate.session.call_session_id, "call-session-1");

    let restarted = persistence.clone();
    let completed = restarted
        .call_operation("account-1", "operation-initiate")
        .await
        .expect("load operation after persistence restart")
        .expect("operation exists");
    assert_eq!(completed.state, TelegramCallOperationState::Completed);
    assert_eq!(completed.tdlib_call_id, Some(77));

    restarted
        .ingest_provider_update(
            "call-session-1",
            &TelegramProviderCallUpdate {
                state: TelegramProviderCallState::MediaReady,
                pending_created: false,
                pending_received: false,
                observed_at_unix_seconds: 104,
                ..provider_update
            },
        )
        .await
        .expect("project media-ready state before mute");

    let mute = TelegramCallCommand::SetLocalMute {
        operation_id: "operation-mute".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        muted: true,
    };
    restarted
        .accept_call_command(&mute, None, 1, 1, 105)
        .await
        .expect("accept local mute");
    let claimed = restarted
        .claim_accepted_call_operations("account-1", 1, 1, 106, 10)
        .await
        .expect("claim local mute");
    assert_eq!(claimed.len(), 1);
    restarted
        .complete_local_mute_operation("account-1", "operation-mute", 107)
        .await
        .expect("complete local mute");
    assert!(
        restarted
            .local_mute("account-1", "call-session-1")
            .await
            .expect("read local mute")
    );

    let end = TelegramCallCommand::End {
        operation_id: "operation-end-stale".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
    };
    restarted
        .accept_call_command(&end, None, 1, 1, 108)
        .await
        .expect("accept end under old fence");
    assert_eq!(
        restarted
            .fail_stale_accepted_call_operations("account-1", 2, 2, 109)
            .await
            .expect("fence stale command"),
        1
    );
    let failed = restarted
        .call_operation("account-1", "operation-end-stale")
        .await
        .expect("load fenced operation")
        .expect("fenced operation exists");
    assert_eq!(failed.state, TelegramCallOperationState::Failed);
    assert_eq!(
        failed.failure_category,
        Some(TelegramCallFailureCategory::Permission)
    );

    let realtime = restarted
        .realtime_after("account-1", 0, 100)
        .await
        .expect("replay unified call events");
    assert!(realtime.len() >= 10);
    assert!(
        realtime
            .windows(2)
            .all(|window| window[0].sequence < window[1].sequence)
    );
}
