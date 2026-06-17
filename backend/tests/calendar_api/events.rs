use std::env;

use chrono::{Duration, Utc};
use serde_json::json;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, create_cal_event, delete_request_with_token,
    get_request_with_token, json_body, post_request_with_token, put_request_with_token,
    unique_suffix, urlencoding_percent_encode,
};

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
    let _items = body["items"].as_array().expect("items");
}

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
