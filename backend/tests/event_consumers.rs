use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderKind, NewProviderAccount,
};
use hermes_hub_backend::domains::communications::messages::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, ProviderChannelMessageStore,
    project_provider_observation_event,
};
use hermes_hub_backend::integrations::telegram::client::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState, TelegramStore,
};
use hermes_hub_backend::platform::communications::{
    EventStoreProviderMessageObservationEventPort, ProviderMessageObservationEvent,
    ProviderMessageObservationEventPort,
};
use hermes_hub_backend::platform::events::{
    EventConsumerConfig, EventConsumerRunner, EventDeadLetterReviewState, EventStore,
    EventStoreError, NewEventEnvelope,
};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::workflows::provider_communication_projection::record_and_project_telegram_message;
use testkit::context::TestContext;

async fn live_context(test_name: &str) -> Option<(Database, EventStore)> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name}: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());
    Some((database, store))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn consumer_config(name: String, max_attempts: i32) -> EventConsumerConfig {
    EventConsumerConfig {
        consumer_name: name,
        batch_size: 1,
        max_attempts,
        retry_base_seconds: 0,
    }
}

async fn append_test_event(store: &EventStore, suffix: u128, marker: &str) -> i64 {
    let event_id = format!("evt_consumer_{marker}_{suffix}");
    let event = NewEventEnvelope::builder(
        &event_id,
        "system.consumer_test_event",
        Utc::now(),
        json!({
            "kind": "test",
            "provider": "event-consumers",
            "source_id": event_id
        }),
        json!({"kind": "system", "entity_id": "event-consumer-test"}),
    )
    .payload(json!({"marker": marker}))
    .build()
    .expect("valid event");

    store.append(&event).await.expect("append test event")
}

#[tokio::test]
async fn consumer_cursor_does_not_advance_before_success_against_postgres() {
    let Some((database, store)) = live_context("event consumer cursor").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "cursor").await;
    let pool = database.pool().expect("configured pool").clone();
    let consumer_name = format!("consumer_cursor_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 3));
    let starting_cursor = position - 1;
    runner
        .store()
        .save_position(&consumer_name, starting_cursor)
        .await
        .expect("place cursor before test event");

    let failed = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "transient failure".to_owned(),
            ))
        })
        .await
        .expect("run failed handler");

    assert_eq!(failed.failed, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after failure"),
        starting_cursor
    );
    assert_eq!(
        runner
            .store()
            .failure_attempt_count(&consumer_name, position)
            .await
            .expect("failure attempt count"),
        Some(1)
    );

    let succeeded = runner
        .process_next_batch(|_| async { Ok(()) })
        .await
        .expect("run successful handler");

    assert_eq!(succeeded.processed, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after success"),
        position
    );
    assert_eq!(
        runner
            .store()
            .failure_attempt_count(&consumer_name, position)
            .await
            .expect("failure removed"),
        None
    );
}

#[tokio::test]
async fn consumer_retries_then_dead_letters_after_max_attempts_against_postgres() {
    let Some((database, store)) = live_context("event consumer DLQ").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "dlq").await;
    let pool = database.pool().expect("configured pool").clone();
    let consumer_name = format!("consumer_dlq_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 2));
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place cursor before test event");

    let first = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "first failure".to_owned(),
            ))
        })
        .await
        .expect("first failure");

    assert_eq!(first.failed, 1);
    assert_eq!(first.dead_lettered, 0);
    assert_eq!(
        runner
            .store()
            .failure_attempt_count(&consumer_name, position)
            .await
            .expect("first attempt count"),
        Some(1)
    );

    let second = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "second failure".to_owned(),
            ))
        })
        .await
        .expect("second failure");

    assert_eq!(second.failed, 1);
    assert_eq!(second.dead_lettered, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after DLQ"),
        position
    );

    let dead_letter = runner
        .store()
        .dead_letter_for_event(&consumer_name, position)
        .await
        .expect("load dead letter")
        .expect("dead letter exists");

    assert_eq!(dead_letter.attempts, 2);
    assert_eq!(dead_letter.review_state, EventDeadLetterReviewState::Open);
    assert_eq!(dead_letter.event.position, position);
}

#[tokio::test]
async fn dead_letter_replay_marks_event_replayed_against_postgres() {
    let Some((database, store)) = live_context("event consumer DLQ replay").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "replay").await;
    let pool = database.pool().expect("configured pool").clone();
    let consumer_name = format!("consumer_replay_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 1));
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place cursor before test event");

    runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "poison event".to_owned(),
            ))
        })
        .await
        .expect("dead letter event");

    let dead_letter = runner
        .store()
        .dead_letter_for_event(&consumer_name, position)
        .await
        .expect("load dead letter")
        .expect("dead letter exists");
    runner
        .store()
        .request_dead_letter_replay(&dead_letter.dead_letter_id)
        .await
        .expect("request replay");

    runner
        .replay_dead_letter(&dead_letter.dead_letter_id, |event| async move {
            assert_eq!(event.position, position);
            Ok(())
        })
        .await
        .expect("replay dead letter");

    let replayed = runner
        .store()
        .dead_letter_by_id(&dead_letter.dead_letter_id)
        .await
        .expect("load replayed dead letter");
    assert_eq!(replayed.review_state, EventDeadLetterReviewState::Replayed);
}

#[tokio::test]
async fn duplicate_consumer_event_delivery_is_idempotent_against_postgres() {
    let Some((database, store)) = live_context("event consumer idempotency").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "idempotent").await;
    let pool = database.pool().expect("configured pool").clone();
    let consumer_name = format!("consumer_idempotent_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 3));
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place cursor before test event");
    let call_count = Arc::new(AtomicUsize::new(0));

    let first_count = Arc::clone(&call_count);
    runner
        .process_next_batch(move |_| {
            let first_count = Arc::clone(&first_count);
            async move {
                first_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("first processing");

    let second_count = Arc::clone(&call_count);
    runner
        .process_next_batch(move |_| {
            let second_count = Arc::clone(&second_count);
            async move {
                second_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("second processing");

    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after idempotent processing"),
        position
    );

    assert_eq!(
        runner
            .store()
            .processed_event_count(&consumer_name, position)
            .await
            .expect("processed marker count"),
        1
    );

    sqlx::query(
        r#"
        UPDATE event_consumers
        SET last_processed_position = $2, updated_at = now()
        WHERE consumer_name = $1
        "#,
    )
    .bind(&consumer_name)
    .bind(position - 1)
    .execute(database.pool().expect("configured pool"))
    .await
    .expect("rewind consumer cursor");

    let duplicate_count = Arc::clone(&call_count);
    let duplicate = runner
        .process_next_batch(move |_| {
            let duplicate_count = Arc::clone(&duplicate_count);
            async move {
                duplicate_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("duplicate delivery");

    assert_eq!(duplicate.skipped_duplicates, 1);
    assert_eq!(duplicate.processed, 0);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        runner
            .store()
            .processed_event_count(&consumer_name, position)
            .await
            .expect("processed marker still single"),
        1
    );
}

#[tokio::test]
async fn provider_observation_events_are_emitted_with_required_telegram_event_types_against_postgres()
 {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let message = create_projected_telegram_message(&pool, "event-types").await;
    let event_port = EventStoreProviderMessageObservationEventPort::new(pool.clone());
    let observed_at = Utc::now();

    let observations = [
        (
            "content_observed",
            None,
            json!({
                "body_text": "event type content",
                "message_metadata": {"event_type_test": "content"},
                "observed_at": observed_at,
            }),
        ),
        (
            "metadata_observed",
            None,
            json!({"message_metadata": {"event_type_test": "metadata"}}),
        ),
        (
            "delivery_state_observed",
            None,
            json!({"delivery_state": "read", "observed_at": observed_at}),
        ),
        (
            "pinned_state_observed",
            None,
            json!({"is_pinned": true, "observed_at": observed_at}),
        ),
        (
            "attachment_download_state_observed",
            None,
            json!({
                "provider_attachment_id": "att-event-types",
                "provider_file_id": 42,
                "download_state": "downloaded",
                "local_path": "docker/data/telegram/att-event-types.bin",
                "size_bytes": 12,
                "content_type": "application/octet-stream",
                "filename": "att.bin",
                "observed_at": observed_at,
            }),
        ),
    ];

    for (event_kind, external_event_id, payload) in observations {
        append_provider_observation(
            &event_port,
            &message,
            event_kind,
            external_event_id,
            observed_at,
            &payload,
        )
        .await
        .expect("append provider observation");
    }

    let event_types = sqlx::query_scalar::<_, String>(
        r#"
        SELECT event_type
        FROM event_log
        WHERE source->>'kind' = 'provider_observation'
          AND source->>'account_id' = $1
        ORDER BY event_type ASC
        "#,
    )
    .bind(&message.account_id)
    .fetch_all(&pool)
    .await
    .expect("provider observation event types");

    assert!(event_types.contains(&"integration.telegram.message.content_observed".to_owned()));
    assert!(event_types.contains(&"integration.telegram.message.metadata_observed".to_owned()));
    assert!(
        event_types.contains(&"integration.telegram.message.delivery_state_observed".to_owned())
    );
    assert!(event_types.contains(&"integration.telegram.message.pinned_state_observed".to_owned()));
    assert!(
        event_types.contains(&"integration.telegram.attachment.download_state_observed".to_owned())
    );
}

#[tokio::test]
async fn communication_provider_observation_projection_is_idempotent_against_postgres() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let message = create_projected_telegram_message(&pool, "projection-idempotent").await;
    let event_port = EventStoreProviderMessageObservationEventPort::new(pool.clone());
    let observed_at = Utc::now();
    let payload = json!({"message_metadata": {"projection_marker": "external-event"}});

    let first_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        Some("provider-event-1"),
        observed_at,
        &payload,
    )
    .await
    .expect("first provider observation")
    .expect("first append position");
    let duplicate_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        Some("provider-event-1"),
        observed_at,
        &payload,
    )
    .await
    .expect("duplicate provider observation");
    assert_eq!(duplicate_position, None);

    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig {
            consumer_name: COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER.to_owned(),
            batch_size: 10,
            max_attempts: 3,
            retry_base_seconds: 0,
        },
    );
    runner
        .store()
        .save_position(
            COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
            first_position - 1,
        )
        .await
        .expect("place consumer before provider event");

    let first_run = runner
        .process_next_batch(|event| project_provider_observation_event(pool.clone(), event))
        .await
        .expect("project provider observation");
    assert_eq!(first_run.processed, 1);

    let projected = ProviderChannelMessageStore::new(pool.clone())
        .message_by_id(&message.message_id, &["telegram_user", "telegram_bot"])
        .await
        .expect("load projected message")
        .expect("projected message exists");
    assert_eq!(
        projected.message_metadata["projection_marker"],
        json!("external-event")
    );

    sqlx::query(
        r#"
        UPDATE event_consumers
        SET last_processed_position = $2, updated_at = now()
        WHERE consumer_name = $1
        "#,
    )
    .bind(COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER)
    .bind(first_position - 1)
    .execute(&pool)
    .await
    .expect("rewind projection consumer cursor");

    let replay = runner
        .process_next_batch(|event| project_provider_observation_event(pool.clone(), event))
        .await
        .expect("replay provider observation");
    assert_eq!(replay.skipped_duplicates, 1);

    let communication_update_events = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.updated'
          AND causation_id = (
              SELECT event_id
              FROM event_log
              WHERE position = $1
          )
        "#,
    )
    .bind(first_position)
    .fetch_one(&pool)
    .await
    .expect("communication update event count");
    assert_eq!(communication_update_events, 1);
}

#[tokio::test]
async fn provider_observation_fallback_idempotency_uses_payload_hash_against_postgres() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let message = create_projected_telegram_message(&pool, "fallback-idempotent").await;
    let event_port = EventStoreProviderMessageObservationEventPort::new(pool);
    let observed_at = Utc::now();
    let payload = json!({"message_metadata": {"projection_marker": "fallback"}});

    let first_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        None,
        observed_at,
        &payload,
    )
    .await
    .expect("first fallback observation");
    let duplicate_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        None,
        observed_at,
        &payload,
    )
    .await
    .expect("duplicate fallback observation");

    assert!(first_position.is_some());
    assert_eq!(duplicate_position, None);
}

async fn create_projected_telegram_message(
    pool: &sqlx::PgPool,
    suffix: &str,
) -> hermes_hub_backend::integrations::telegram::client::TelegramMessage {
    let unique = unique_suffix();
    let account_id = format!("acct-{suffix}-{unique}");
    let account = NewProviderAccount::new(
        account_id.clone(),
        CommunicationProviderKind::TelegramUser,
        format!("Telegram {suffix}"),
        format!("telegram:{suffix}:{unique}"),
    )
    .config(json!({"runtime": "fixture"}));
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&account)
        .await
        .expect("provider account");

    let store = TelegramStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(
            hermes_hub_backend::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(EventStoreProviderMessageObservationEventPort::new(pool.clone())),
    );
    let provider_chat_id = format!("-100{suffix}{unique}");
    let provider_message_id = format!("{provider_chat_id}:1");
    let observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: format!("Chat {suffix}"),
            sender_id: "user:1".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: format!("batch-{suffix}-{unique}"),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture");
    let projected = record_and_project_telegram_message(pool.clone(), observed.raw)
        .await
        .expect("project fixture");
    store
        .message_by_id(&projected.message_id)
        .await
        .expect("load projected message")
        .expect("projected message exists")
}

async fn append_provider_observation(
    event_port: &EventStoreProviderMessageObservationEventPort,
    message: &hermes_hub_backend::integrations::telegram::client::TelegramMessage,
    event_kind: &str,
    external_event_id: Option<&str>,
    observed_at: chrono::DateTime<Utc>,
    payload: &serde_json::Value,
) -> Result<
    Option<i64>,
    hermes_hub_backend::platform::communications::ProviderCommunicationMessagePortError,
> {
    event_port
        .append_provider_message_observation(ProviderMessageObservationEvent {
            provider: "telegram",
            account_id: &message.account_id,
            channel_kind: &message.channel_kind,
            message_id: &message.message_id,
            external_message_id: &message.provider_message_id,
            event_kind,
            observed_at,
            external_event_id,
            payload,
            causation_id: Some("event-consumer-test"),
        })
        .await
}
