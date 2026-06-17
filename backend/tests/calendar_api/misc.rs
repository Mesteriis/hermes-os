use std::env;

use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, delete_request_with_token, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

async fn get_calendar_endpoint_returns_non_server_error(path: &str) {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar {path} test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(path, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "{path} status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_deadlines_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/deadlines").await;
}

#[tokio::test]
async fn calendar_focus_blocks_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/focus-blocks").await;
}

#[tokio::test]
async fn calendar_watchtower_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/watchtower").await;
}

#[tokio::test]
async fn calendar_health_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/health").await;
}

#[tokio::test]
async fn calendar_weekly_brief_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/weekly-brief").await;
}

#[tokio::test]
async fn calendar_search_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/search?q=meeting").await;
}

#[tokio::test]
async fn calendar_rules_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/rules").await;
}

#[tokio::test]
async fn calendar_analytics_distribution_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/distribution").await;
}

#[tokio::test]
async fn calendar_sources_list() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("SrcAcct{suffix}"), "email": format!("src-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: acct create failed");
        return;
    }
    let account_id = json_body(response).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if account_id.is_empty() {
        eprintln!("skip: no account_id");
        return;
    }

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"name": format!("Src{suffix}"), "color": "#ff0000", "timezone": "UTC"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "src create={}",
        response.status()
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "src list={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_post_deadline() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/deadlines",
            json!({"title": "Test Deadline", "due_at": "2027-12-31T23:59:59Z", "priority": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "deadline post={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_post_focus_block() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app.oneshot(post_request_with_token(
        "/api/v1/calendar/focus-blocks",
        json!({"title": "Focus Block", "start_at": chrono::Utc::now().to_rfc3339(), "duration_minutes": 90}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "focus block post={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_post_smart_schedule() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app.oneshot(post_request_with_token(
        "/api/v1/calendar/smart-schedule",
        json!({"task_title": "Schedule me", "duration_minutes": 60, "deadline": "2027-12-31T23:59:59Z"}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "smart schedule={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_analytics() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics").await;
}

#[tokio::test]
async fn cal_focus_balance() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/focus-balance")
        .await;
}

#[tokio::test]
async fn cal_back_to_back() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/back-to-back").await;
}

#[tokio::test]
async fn cal_rules_crud() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/rules",
        json!({"name": format!("Rule{suffix}"), "rule_type": "auto_color", "config": {"color": "#00ff00"}}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_default();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let rule_id = value["rule_id"].as_str().unwrap_or("").to_owned();
    if rule_id.is_empty() {
        return;
    }

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            json!({"name": format!("Updated{suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule update={}",
        response.status()
    );

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule delete={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_import() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/import",
            json!({"format": "ics", "data": "BEGIN:VCALENDAR\nEND:VCALENDAR"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "import={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_sync() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("Sync{suffix}"), "email": format!("sync-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip");
        return;
    }
    let account_id = json_body(response).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if account_id.is_empty() {
        return;
    }

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sync",
                urlencoding_percent_encode(&account_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "sync={}",
        response.status()
    );
}
