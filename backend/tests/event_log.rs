use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde_json::json;

use hermes_hub_backend::platform::events::{
    EventEnvelope, EventStore, NewEventEnvelope, ProjectionCursorStore, StoredEventEnvelope,
};
use hermes_hub_backend::platform::storage::Database;

#[test]
fn new_event_envelope_rejects_empty_event_type() {
    let error = NewEventEnvelope::builder(
        "evt_test_empty_type",
        " ",
        Utc::now(),
        json!({"kind": "test", "source_id": "source-empty-type"}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect_err("empty event type must fail");

    assert_eq!(error.to_string(), "event_type must not be empty");
}

#[test]
fn new_event_envelope_rejects_non_object_source() {
    let error = NewEventEnvelope::builder(
        "evt_test_bad_source",
        "system_test_event",
        Utc::now(),
        json!("not-an-object"),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect_err("non-object source must fail");

    assert_eq!(error.to_string(), "source must be a JSON object");
}

#[tokio::test]
async fn event_store_appends_and_loads_event_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live event store test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_test_{suffix}");
    let occurred_at: DateTime<Utc> = Utc::now();

    let event = NewEventEnvelope::builder(
        &event_id,
        "system_test_event",
        occurred_at,
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": event_id,
            "import_batch_id": "event-log-test"
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .payload(json!({"test": true}))
    .provenance(json!({"confidence": 1.0}))
    .correlation_id("corr_event_log_test")
    .build()
    .expect("valid event");

    store.append(&event).await.expect("append event");

    let loaded = store
        .get_by_id(&event_id)
        .await
        .expect("load event")
        .expect("event exists");

    assert_eq!(
        loaded,
        EventEnvelope {
            event_id: event_id.clone(),
            event_type: "system_test_event".to_owned(),
            schema_version: 1,
            occurred_at,
            recorded_at: loaded.recorded_at,
            source: json!({
                "kind": "test",
                "provider": "integration",
                "source_id": loaded.event_id,
                "import_batch_id": "event-log-test"
            }),
            actor: None,
            subject: json!({"kind": "system", "entity_id": "backend"}),
            payload: json!({"test": true}),
            provenance: json!({"confidence": 1.0}),
            causation_id: None,
            correlation_id: Some("corr_event_log_test".to_owned()),
        }
    );

    let duplicate_source_event = NewEventEnvelope::builder(
        format!("{event_id}_duplicate"),
        "system_test_event",
        occurred_at,
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": loaded.event_id,
            "import_batch_id": "event-log-test"
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid duplicate source event");

    assert!(
        store.append(&duplicate_source_event).await.is_err(),
        "same event_type and source identity must be idempotent"
    );

    let mutation = sqlx::query("UPDATE event_log SET payload = '{}'::jsonb WHERE event_id = $1")
        .bind(&loaded.event_id)
        .execute(database.pool().expect("configured pool"))
        .await;

    assert!(mutation.is_err(), "event_log must be append-only");
}

#[tokio::test]
async fn event_store_replays_events_after_position_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live replay test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let occurred_at = Utc::now();
    let first_id = format!("evt_replay_first_{suffix}");
    let second_id = format!("evt_replay_second_{suffix}");

    let first = NewEventEnvelope::builder(
        &first_id,
        "system_replay_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": first_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid first event");
    let first_position = store.append(&first).await.expect("append first event");

    let second = NewEventEnvelope::builder(
        &second_id,
        "system_replay_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": second_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid second event");
    let second_position = store.append(&second).await.expect("append second event");

    let replayed = store
        .list_after_position(first_position, 10)
        .await
        .expect("replay events");

    assert_eq!(
        replayed,
        vec![StoredEventEnvelope {
            position: second_position,
            event: EventEnvelope {
                event_id: second_id,
                event_type: "system_replay_test_event".to_owned(),
                schema_version: 1,
                occurred_at,
                recorded_at: replayed[0].event.recorded_at,
                source: json!({
                    "kind": "test",
                    "provider": "integration",
                    "source_id": replayed[0].event.event_id
                }),
                actor: None,
                subject: json!({"kind": "system", "entity_id": "backend"}),
                payload: json!({}),
                provenance: json!({}),
                causation_id: None,
                correlation_id: None,
            },
        }]
    );
}

#[tokio::test]
async fn projection_cursor_store_tracks_monotonic_positions_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live projection cursor test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let cursors = ProjectionCursorStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let projection_name = format!("projection_cursor_test_{suffix}");

    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("initial cursor"),
        0
    );

    cursors
        .save_position(&projection_name, 10)
        .await
        .expect("save initial position");
    cursors
        .save_position(&projection_name, 7)
        .await
        .expect("lower position must not regress cursor");

    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("updated cursor"),
        10
    );
}
