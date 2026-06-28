use std::sync::Arc;

use chrono::{Duration, Utc};
use hermes_hub_backend::domains::calendar::core::EventRelationStore;
use hermes_hub_backend::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use hermes_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use hermes_hub_backend::integrations::zoom::client::{ZoomMeetingObservationRequest, ZoomStore};
use hermes_hub_backend::platform::calls::CallIntelligenceStore;
use hermes_hub_backend::platform::events::{EventBus, EventLogQuery, EventStore};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::workflows::zoom_calendar_matching::{
    ZOOM_CALENDAR_RELATION_TYPE, project_zoom_calendar_matching,
};
use serde_json::json;
use testkit::context::TestContext;

#[tokio::test]
async fn zoom_meeting_events_match_calendar_events_into_call_relations() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let started_at = Utc::now();
    let ended_at = started_at + Duration::minutes(45);
    let join_url = format!("https://example.zoom.us/j/987654321?pwd={suffix}");

    let calendar_event = CalendarEventStore::new(pool.clone())
        .create(&NewCalendarEvent {
            title: format!("Zoom Match {suffix}"),
            start_at: started_at - Duration::minutes(5),
            end_at: ended_at + Duration::minutes(5),
            conference_url: Some(join_url.clone()),
            conference_provider: Some("zoom".to_owned()),
            event_type: Some("meeting".to_owned()),
            ..Default::default()
        })
        .await
        .expect("calendar event");

    let event_bus = EventBus::new();
    let zoom_store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            hermes_hub_backend::domains::communications::storage::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool.clone()),
        event_bus,
    );
    let account_id = format!("zoom-calendar-match-{suffix}");
    zoom_store
        .setup_fixture_account(
            &hermes_hub_backend::integrations::zoom::client::ZoomAccountSetupRequest {
                account_id: account_id.clone(),
                display_name: "Zoom Calendar Match Fixture".to_owned(),
                external_account_id: format!("zoom-calendar-match-external-{suffix}"),
                account_email: None,
                metadata: json!({}),
            },
        )
        .await
        .expect("fixture account");

    let meeting_id = "987654321".to_owned();
    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-calendar-match-observation-{suffix}")),
            account_id: account_id.clone(),
            meeting_id: meeting_id.clone(),
            meeting_uuid: Some(format!("zoom-calendar-match-uuid-{suffix}")),
            topic: Some("Weekly review".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(join_url.clone()),
            started_at: Some(started_at),
            ended_at: Some(ended_at),
            duration_seconds: Some(45 * 60),
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({ "source": "zoom_calendar_matching_test" }),
            causation_id: None,
            correlation_id: Some(format!("zoom-calendar-match-correlation-{suffix}")),
        })
        .await
        .expect("observe zoom meeting");
    let call_id = observed.call_id;

    let direct_match = CalendarEventStore::new(pool.clone())
        .find_zoom_conference_match(
            Some(&join_url),
            &meeting_id,
            Some(started_at),
            Some(ended_at),
        )
        .await
        .expect("direct calendar match");
    assert_eq!(
        direct_match.as_ref().map(|event| event.event_id.as_str()),
        Some(calendar_event.event_id.as_str())
    );

    let stored_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(format!("zoom-calendar-match-correlation-{suffix}")),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("stored zoom events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(call_id))
        .expect("matched stored zoom event");
    assert_eq!(stored_event.event.payload["meeting_id"], json!(meeting_id));
    assert_eq!(stored_event.event.payload["join_url"], json!(join_url));
    assert_eq!(stored_event.event.payload["started_at"], json!(started_at));

    project_zoom_calendar_matching(&pool, &stored_event.event)
        .await
        .expect("zoom calendar matching projection");

    let relations = EventRelationStore::new(pool.clone())
        .list(&calendar_event.event_id)
        .await
        .expect("calendar event relations");
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].entity_type, "call");
    assert_eq!(relations[0].entity_id, call_id);
    assert_eq!(relations[0].relation_type, ZOOM_CALENDAR_RELATION_TYPE);
    assert_eq!(relations[0].source, "zoom.meeting.observed");

    project_zoom_calendar_matching(&pool, &stored_event.event)
        .await
        .expect("repeat zoom calendar matching");

    let relations_after_repeat = EventRelationStore::new(pool)
        .list(&calendar_event.event_id)
        .await
        .expect("calendar event relations after repeat");
    assert_eq!(relations_after_repeat.len(), 1);
}
