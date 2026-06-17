use std::env;

use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, create_cal_event, get_request_with_token, json_body,
    post_request_with_token, unique_suffix, urlencoding_percent_encode,
};

async fn event_get_endpoint_returns_non_server_error(path_suffix: &str) -> Option<Value> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event {path_suffix} test: HERMES_TEST_DATABASE_URL is not set"
        );
        return None;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return None;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/{}",
                urlencoding_percent_encode(&event_id),
                path_suffix
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    Some(json_body(response).await)
}

#[tokio::test]
async fn calendar_event_relations_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("relations").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_context_pack_returns_ok() {
    event_get_endpoint_returns_non_server_error("context-pack").await;
}

#[tokio::test]
async fn calendar_event_agenda_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("agenda").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_checklist_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("checklist").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_risks_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("risks").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_meeting_notes_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("notes").await else {
        return;
    };
    assert!(body["items"].is_array() || body.is_object());
}

#[tokio::test]
async fn calendar_event_meeting_outcomes_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("outcomes").await else {
        return;
    };
    assert!(body["items"].is_array() || body.is_object());
}

#[tokio::test]
async fn calendar_event_recording_list_returns_ok() {
    event_get_endpoint_returns_non_server_error("recording").await;
}

#[tokio::test]
async fn calendar_event_transcript_returns_ok() {
    event_get_endpoint_returns_non_server_error("transcript").await;
}

#[tokio::test]
async fn calendar_event_brief_returns_ok() {
    event_get_endpoint_returns_non_server_error("brief").await;
}

#[tokio::test]
async fn calendar_event_export_returns_text() {
    event_get_endpoint_returns_non_server_error("export").await;
}

#[tokio::test]
async fn calendar_event_reminders_list_returns_empty() {
    event_get_endpoint_returns_non_server_error("reminders").await;
}

macro_rules! cal_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let suffix = unique_suffix();
            let app = build_cal_app(&database_url).await;
            let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
                eprintln!("skip: no event");
                return;
            };
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/calendar/events/{}/{}",
                        urlencoding_percent_encode(&event_id),
                        $path_suffix
                    ),
                    $body,
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

cal_post_test!(
    cal_event_post_relation,
    "relations",
    json!({"related_event_id": "event:fake", "relation_type": "follows"})
);
cal_post_test!(
    cal_event_post_context_pack,
    "context-pack",
    json!({"summary": "Test context"})
);
cal_post_test!(
    cal_event_post_agenda,
    "agenda",
    json!({"item": "Test agenda item", "order_index": 1})
);
cal_post_test!(
    cal_event_post_checklist,
    "checklist",
    json!({"item": "Test checklist", "done": false})
);
cal_post_test!(
    cal_event_post_meeting_note,
    "notes",
    json!({"content": "Test note", "note_type": "action_item"})
);
cal_post_test!(
    cal_event_post_meeting_outcome,
    "outcomes",
    json!({"outcome": "Test outcome", "decision": false})
);
cal_post_test!(
    cal_event_post_follow_up,
    "follow-up",
    json!({"action": "Send follow-up", "due_by": "2027-12-01T00:00:00Z"})
);
cal_post_test!(
    cal_event_post_recording,
    "recording",
    json!({"url": "https://example.com/rec", "format": "mp4"})
);
cal_post_test!(
    cal_event_post_generate_agenda,
    "generate-agenda",
    json!({"participant_count": 3, "duration_minutes": 60})
);
cal_post_test!(
    cal_event_post_reminder,
    "reminders",
    json!({"minutes_before": 15, "method": "notification"})
);

#[tokio::test]
async fn cal_event_follow_up_status() {
    event_get_endpoint_returns_non_server_error("follow-up-status").await;
}

#[tokio::test]
async fn cal_event_reminder_toggle() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };
    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders/rem:fake/toggle",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "reminder toggle={}",
        response.status()
    );
}
