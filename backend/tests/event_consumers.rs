use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::platform::events::{
    EventConsumerConfig, EventConsumerRunner, EventDeadLetterReviewState, EventStore,
    EventStoreError, NewEventEnvelope,
};
use hermes_hub_backend::platform::storage::Database;

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
