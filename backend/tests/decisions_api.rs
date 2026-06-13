use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::decisions::{
    Decision, DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore,
    NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "decisions-api-test-token";

#[tokio::test]
async fn decisions_list_returns_entity_scoped_decisions() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live decisions API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let project_id = format!("project:v1:decision-api-{suffix}");
    let stored = seed_decision(&pool, suffix, &project_id).await;

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/decisions?entity_kind=project&entity_id={project_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["decision_id"] == json!(stored.decision_id))
        .expect("seeded decision");

    assert_eq!(item["title"], stored.title);
    assert_eq!(item["status"], "active");
    assert_eq!(item["review_state"], "suggested");
    assert_eq!(item["decided_by_entity_kind"], "persona");
}

#[tokio::test]
async fn decisions_list_returns_global_suggested_review_items() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live decisions global review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested_project_id = format!("project:v1:decision-global-suggested-{suffix}");
    let confirmed_project_id = format!("project:v1:decision-global-confirmed-{suffix}");
    let suggested = seed_decision_with_review_state(
        &pool,
        suffix,
        &suggested_project_id,
        DecisionReviewState::Suggested,
    )
    .await;
    let confirmed = seed_decision_with_review_state(
        &pool,
        suffix + 1,
        &confirmed_project_id,
        DecisionReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/decisions?review_state=suggested&limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(
        items
            .iter()
            .any(|item| item["decision_id"] == json!(suggested.decision_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["decision_id"] != json!(confirmed.decision_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_decision_review_updates_review_state_without_creating_work() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live decision review API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let project_id = format!("project:v1:decision-review-{suffix}");
    let stored = seed_decision(&pool, suffix, &project_id).await;
    let decision_id = path_segment(&stored.decision_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/decisions/{decision_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["decision_id"], stored.decision_id);
    assert_eq!(body["review_state"], "user_confirmed");

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(&stored.decision_id)
            .fetch_one(&pool)
            .await
            .expect("stored review state");
    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&stored.decision_id)
            .fetch_one(&pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE metadata @> $1")
            .bind(json!({"decision_id": stored.decision_id}))
            .fetch_one(&pool)
            .await
            .expect("obligation count");

    assert_eq!(review_state, "user_confirmed");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    );

    (app, pool)
}

async fn seed_decision(pool: &PgPool, suffix: u128, project_id: &str) -> Decision {
    seed_decision_with_review_state(pool, suffix, project_id, DecisionReviewState::Suggested).await
}

async fn seed_decision_with_review_state(
    pool: &PgPool,
    suffix: u128,
    project_id: &str,
    review_state: DecisionReviewState,
) -> Decision {
    let decision = NewDecision::new(
        format!("Adopt decision API route {suffix}"),
        "Accepted decisions need a guarded backend review surface.",
        0.84,
        review_state,
    )
    .decided_by(
        DecisionEntityKind::Persona,
        format!("person:v1:email:decision-api-{suffix}@example.com"),
    )
    .decided_at(Utc.with_ymd_and_hms(2026, 6, 12, 11, 0, 0).unwrap())
    .alternatives(json!([
        "store decisions only in meeting outcomes",
        "hide decisions in project notes"
    ]))
    .metadata(json!({"source": "decisions_api_test"}));
    let evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Event,
        format!("meeting:decision-api:{suffix}"),
    )
    .quote("We decided to expose accepted decisions through guarded backend routes.")
    .confidence(0.91)
    .metadata(json!({"source": "decisions_api_test"}));
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, project_id)
        .impact_type("architecture_direction")
        .metadata(json!({"source": "decisions_api_test"}));

    DecisionStore::new(pool.clone())
        .upsert_with_evidence(&decision, &[evidence], &[impact])
        .await
        .expect("seed decision")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
