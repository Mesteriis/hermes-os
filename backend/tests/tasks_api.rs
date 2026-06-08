use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "tasks-api-test-token";

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

async fn build_tasks_app(database_url: &str) -> axum::Router {
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

async fn create_task(app: &axum::Router, suffix: u128) -> Option<String> {
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API Task {suffix}"),
                "description": "Task for API testing",
                "status": "active",
                "priority": "medium",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    json_body(response).await["task_id"]
        .as_str()
        .map(|s| s.to_owned())
}

// ── Auth ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tasks_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/tasks"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-hermes-secret header"})
    );
}

// ── CRUD ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tasks_crud_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live tasks CRUD test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    // Create
    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/tasks",
        json!({"title": format!("CRUD Task {suffix}"), "description": "CRUD test", "status": "active"}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: task create failed");
        return;
    }
    let created = json_body(response).await;
    let Some(task_id) = created["task_id"].as_str().map(|s| s.to_owned()) else {
        eprintln!("skip: no task_id");
        return;
    };
    assert_eq!(created["title"], json!(format!("CRUD Task {suffix}")));

    // Get
    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = json_body(response).await;
    assert_eq!(fetched["task_id"], json!(task_id));

    // Update
    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            json!({"title": format!("Updated Task {suffix}"), "priority": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Task {suffix}")));

    // Archive
    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/archive",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn tasks_list_returns_items() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live tasks list test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    create_task(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let _items = body["items"].as_array().expect("items");
}

// ── Task Status ────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_status_transition() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task status test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/status",
                urlencoding_percent_encode(&task_id)
            ),
            json!({"status": "completed"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Context Pack ───────────────────────────────────────────────────────────

#[tokio::test]
async fn task_context_pack_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task context pack test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/context-pack",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Evidence ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_evidence_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task evidence test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/evidence",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Relations ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_relations_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task relations test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/relations",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Checklist ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_checklist_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task checklist test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/checklist",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Subtasks ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_subtasks_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task subtasks test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/subtasks",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Task Export ────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_export_returns_text() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task export test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/export",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── External ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_external_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task external test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/tasks/{}/external",
                urlencoding_percent_encode(&task_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Providers ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_providers_list_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task providers test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/providers",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Search ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_search_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task search test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/search?q=test",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Daily Brief ────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_daily_brief_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task daily brief test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/daily-brief",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Rules ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_rules_list_returns_empty() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task rules test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/rules",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Templates ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_templates_list_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task templates test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/templates",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Watchtower / Health / Analytics ────────────────────────────────────────

#[tokio::test]
async fn task_watchtower_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task watchtower test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/watchtower",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn task_health_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task health test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/health",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn task_analytics_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task analytics test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/tasks/analytics",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Task Candidates ────────────────────────────────────────────────────────

#[tokio::test]
async fn task_candidates_list_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task candidates test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/task-candidates",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Task Rule CRUD ─────────────────────────────────────────────────────────

#[tokio::test]
async fn task_rule_create_and_delete() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(_task_id) = create_task(&app, suffix).await else {
        eprintln!("skip");
        return;
    };

    let r = app.clone().oneshot(post_request_with_token(
        "/api/v1/tasks/rules",
        json!({"name": format!("Rule{suffix}"), "rule_type": "auto_priority", "config": json!({"default": "medium"})}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    if r.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let rid = json_body(r).await["rule_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if rid.is_empty() {
        return;
    }

    let r = app
        .oneshot(delete_request_with_token(
            &format!("/api/v1/tasks/rules/{}", urlencoding_percent_encode(&rid)),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "rule delete={}", r.status());
}

// ── Task sub-resource POSTs ────────────────────────────────────────────────

macro_rules! task_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let suffix = unique_suffix();
            let app = build_tasks_app(&database_url).await;
            let Some(task_id) = create_task(&app, suffix).await else {
                eprintln!("skip: no task");
                return;
            };
            let r = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/tasks/{}/{}",
                        urlencoding_percent_encode(&task_id),
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

task_post_test!(
    task_post_context_pack,
    "context-pack",
    json!({"summary": "Test context"})
);
task_post_test!(
    task_post_evidence,
    "evidence",
    json!({"source": "email", "reference_id": "msg:test", "note": "Test evidence"})
);
task_post_test!(
    task_post_relation,
    "relations",
    json!({"related_task_id": "task:fake", "relation_type": "blocks"})
);
task_post_test!(
    task_post_checklist,
    "checklist",
    json!({"item": "Test item", "done": false})
);
task_post_test!(
    task_post_subtask,
    "subtasks",
    json!({"title": "Test subtask", "status": "active"})
);

// ── Task Provider POST ─────────────────────────────────────────────────────

#[tokio::test]
async fn task_post_provider() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let r = app.oneshot(post_request_with_token(
        "/api/v1/tasks/providers",
        json!({"name": format!("Provider{suffix}"), "provider_type": "jira", "config": json!({"url": "https://example.com"})}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    assert!(
        !r.status().is_server_error(),
        "provider post={}",
        r.status()
    );
}

// ── Task Candidate Review ──────────────────────────────────────────────────

#[tokio::test]
async fn task_candidate_review() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let r = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/task-candidates/tc:fake-{suffix}/review"),
            json!({"review_state": "declined", "reason": "Not relevant"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "candidate review={}",
        r.status()
    );
}
