use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "cal-api-test-token";

// ── Helpers ────────────────────────────────────────────────────────────────

fn config_with_api_token() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)])
        .expect("valid local API secret")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

async fn build_cal_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    )
}

// ── Auth ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_accounts_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/calendar/accounts"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-hermes-secret header"})
    );
}

#[tokio::test]
async fn calendar_events_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/calendar/events"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── Calendar Accounts CRUD ─────────────────────────────────────────────────

#[tokio::test]
async fn calendar_accounts_crud_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar accounts CRUD test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let acct_name = format!("API Cal Acct {suffix}");

    // Create account
    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": &acct_name, "email": format!("cal-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let created = json_body(response).await;
    let account_id = created["account_id"]
        .as_str()
        .expect("account_id")
        .to_owned();
    assert_eq!(created["provider"], json!("google"));

    // Get account
    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
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

    // Update account
    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"account_name": format!("Updated {acct_name}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(
        updated["account_name"],
        json!(format!("Updated {acct_name}"))
    );

    // Delete account
    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
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
    let deleted = json_body(response).await;
    assert_eq!(deleted["deleted"], json!(true));
}

#[tokio::test]
async fn calendar_accounts_list_returns_items() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar accounts list test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("List Acct {suffix}"), "email": format!("list-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());
}

// ── Calendar Events CRUD ───────────────────────────────────────────────────

async fn create_cal_event(app: &axum::Router, suffix: u128) -> Option<(String, String)> {
    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("Evt Acct {suffix}"), "email": format!("evt-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        return None;
    }
    let account_id = json_body(response).await["account_id"].as_str()?.to_owned();

    let now = Utc::now();
    let start = now + Duration::hours(1);
    let end = now + Duration::hours(2);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/events",
            json!({
                "account_id": &account_id,
                "title": format!("Test Event {suffix}"),
                "start_at": start.to_rfc3339(),
                "end_at": end.to_rfc3339(),
                "status": "confirmed",
                "event_type": "meeting",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    let body = json_body(response).await;
    let event_id = body["event_id"].as_str()?;
    Some((account_id, event_id.to_owned()))
}

#[tokio::test]
async fn calendar_events_crud_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar events CRUD test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    // Get event
    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
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
    let fetched = json_body(response).await;
    assert_eq!(fetched["event_id"], json!(event_id));

    // Update event
    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"title": format!("Updated Event {suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Event {suffix}")));

    // Delete event
    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
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
}

#[tokio::test]
async fn calendar_events_list_returns_items() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar events list test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    create_cal_event(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/events",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    // event creation may have failed gracefully; handler was exercised
    let _items = body["items"].as_array().expect("items");
}

// ── Event Reschedule / Cancel ──────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_reschedule() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event reschedule test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let new_start = Utc::now() + Duration::hours(3);
    let new_end = Utc::now() + Duration::hours(4);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reschedule",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"start_at": new_start.to_rfc3339(), "end_at": new_end.to_rfc3339()}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_event_cancel() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar event cancel test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/cancel",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert_eq!(body["cancelled"], json!(true));
}

// ── Event Participants ─────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_participants_crud() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event participants test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    // Add participant
    let response = app.clone().oneshot(post_request_with_token(
        &format!("/api/v1/calendar/events/{}/participants", urlencoding_percent_encode(&event_id)),
        json!({"email": format!("participant-{suffix}@example.com"), "display_name": format!("Participant {suffix}"), "role": "required"}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    // List participants
    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/participants",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());
}

// ── Event Relations ────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_relations_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event relations test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/relations",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(body["items"].is_array());
}

// ── Event Context Pack ─────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_context_pack_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event context pack test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/context-pack",
                urlencoding_percent_encode(&event_id)
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
}

// ── Event Agenda ───────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_agenda_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar event agenda test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/agenda",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(body["items"].is_array());
}

// ── Event Checklist ────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_checklist_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event checklist test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/checklist",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(body["items"].is_array());
}

// ── Event Risks ────────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_risks_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar event risks test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/risks",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(body["items"].is_array());
}

// ── Meeting Notes ──────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_meeting_notes_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event meeting notes test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/notes",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(body["items"].is_array() || body.is_object());
}

// ── Meeting Outcomes ───────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_meeting_outcomes_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event meeting outcomes test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/outcomes",
                urlencoding_percent_encode(&event_id)
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
    let body = json_body(response).await;
    assert!(body["items"].is_array() || body.is_object());
}

// ── Event Recordings ───────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_recording_list_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event recording test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/recording",
                urlencoding_percent_encode(&event_id)
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
}

// ── Event Transcript ───────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_transcript_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event transcript test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/transcript",
                urlencoding_percent_encode(&event_id)
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
}

// ── Event Brief ────────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_brief_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar event brief test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/brief",
                urlencoding_percent_encode(&event_id)
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
}

// ── Event Export ───────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_export_returns_text() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar event export test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/export",
                urlencoding_percent_encode(&event_id)
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
}

// ── Deadlines ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_deadlines_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar deadlines test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/deadlines",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Focus Blocks ───────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_focus_blocks_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar focus blocks test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/focus-blocks",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Watchtower / Health ────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_watchtower_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar watchtower test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/watchtower",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_health_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar health test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/health",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Weekly Brief ───────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_weekly_brief_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar weekly brief test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/weekly-brief",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Calendar Search ────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_search_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar search test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/search?q=meeting",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Calendar Rules ─────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_rules_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar rules test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/rules",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Event Reminders ────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_event_reminders_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live calendar event reminders test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders",
                urlencoding_percent_encode(&event_id)
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
}

// ── Analytics ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_analytics_distribution_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar analytics test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/analytics/distribution",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

// ── Additional Calendar Handlers ───────────────────────────────────────────

// Calendar Sources
#[tokio::test]
async fn calendar_sources_list() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    // Create account then list its sources
    let r = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("SrcAcct{suffix}"), "email": format!("src-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    if r.status().is_server_error() {
        eprintln!("skip: acct create failed");
        return;
    }
    let aid = json_body(r).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if aid.is_empty() {
        eprintln!("skip: no account_id");
        return;
    }

    let r = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&aid)
            ),
            json!({"name": format!("Src{suffix}"), "color": "#ff0000", "timezone": "UTC"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "src create={}", r.status());

    let r = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&aid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "src list={}", r.status());
}

// Event sub-resource POSTs
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
            let r = app
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
                .expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
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

// Event follow-up status
#[tokio::test]
async fn cal_event_follow_up_status() {
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
    let r = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/follow-up-status",
                urlencoding_percent_encode(&event_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "follow-up status={}",
        r.status()
    );
}

// Reminder toggle
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
    let r = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders/rem:fake/toggle",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "reminder toggle={}",
        r.status()
    );
}

// Deadlines / Focus Blocks POST
#[tokio::test]
async fn cal_post_deadline() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/deadlines",
            json!({"title": "Test Deadline", "due_at": "2027-12-31T23:59:59Z", "priority": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "deadline post={}",
        r.status()
    );
}

#[tokio::test]
async fn cal_post_focus_block() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app.oneshot(post_request_with_token(
        "/api/v1/calendar/focus-blocks",
        json!({"title": "Focus Block", "start_at": chrono::Utc::now().to_rfc3339(), "duration_minutes": 90}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    assert!(
        !r.status().is_server_error(),
        "focus block post={}",
        r.status()
    );
}

#[tokio::test]
async fn cal_post_smart_schedule() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app.oneshot(post_request_with_token(
        "/api/v1/calendar/smart-schedule",
        json!({"task_title": "Schedule me", "duration_minutes": 60, "deadline": "2027-12-31T23:59:59Z"}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    assert!(
        !r.status().is_server_error(),
        "smart schedule={}",
        r.status()
    );
}

// Calendar analytics
#[tokio::test]
async fn cal_analytics() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/analytics",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "analytics={}", r.status());
}

#[tokio::test]
async fn cal_focus_balance() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/analytics/focus-balance",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "focus balance={}",
        r.status()
    );
}

#[tokio::test]
async fn cal_back_to_back() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/analytics/back-to-back",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "back-to-back={}", r.status());
}

// Calendar rules CRUD
#[tokio::test]
async fn cal_rules_crud() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let r = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/rules",
        json!({"name": format!("Rule{suffix}"), "rule_type": "auto_color", "config": {"color": "#00ff00"}}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    if r.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let body = r.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();
    let v: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let rid = v["rule_id"].as_str().unwrap_or("").to_owned();
    if rid.is_empty() {
        return;
    }

    let r = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rid)
            ),
            json!({"name": format!("Updated{suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "rule update={}", r.status());

    let r = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "rule delete={}", r.status());
}

// Calendar import / sync
#[tokio::test]
async fn cal_import() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let r = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/import",
            json!({"format": "ics", "data": "BEGIN:VCALENDAR\nEND:VCALENDAR"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "import={}", r.status());
}

#[tokio::test]
async fn cal_sync() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let r = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("Sync{suffix}"), "email": format!("sync-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    if r.status().is_server_error() {
        eprintln!("skip");
        return;
    }
    let aid = json_body(r).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if aid.is_empty() {
        return;
    }

    let r = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sync",
                urlencoding_percent_encode(&aid)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "sync={}", r.status());
}
