use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::person_identity::PersonIdentityStore;
use hermes_hub_backend::persons::PersonProjectionStore;
use hermes_hub_backend::storage::Database;
use hermes_hub_backend::{build_router, build_router_with_database};

const LOCAL_API_TOKEN: &str = "person-identity-api-test-token";
const LOCAL_API_ACTOR_ID: &str = "person-identity-api-test-client";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";

#[tokio::test]
async fn identity_candidates_reject_missing_local_api_token() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v2/identity-candidates"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_token",
            "message": "missing or invalid bearer token"
        })
    );
}

#[tokio::test]
async fn identity_candidates_returns_safe_candidate_payload() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let context = PersonIdentityApiContext {
        person_store: PersonProjectionStore::new(pool.clone()),
    };
    let shared_name = format!("Identity Api Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    promote_identity_candidate(&pool, &candidate_id)
        .await
        .expect("promote candidate");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v2/identity-candidates?limit=100",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());

    let item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(candidate_id))
        .expect("candidate payload");

    assert_eq!(item["candidate_kind"], "merge_persons");
    assert_eq!(item["review_state"], "suggested");
    assert_eq!(item["left_person_id"], json!(left.person_id));
    assert_eq!(item["right_person_id"], json!(right.person_id));
    assert!(item["evidence_summary"].is_string());
    assert!(item["confidence"].is_number());
}

#[tokio::test]
async fn identity_candidates_returns_split_candidate_for_confirmed_merge() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Api Split {suffix}");

    let left = person_store
        .upsert_email_person(&format!("split-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_person(&format!("split-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let merge_candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let command_id = format!("identity-api-split-confirm-{suffix}");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v2/identity-candidates/{merge_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let split_candidate_id =
        split_identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    promote_identity_candidate(&pool, &split_candidate_id)
        .await
        .expect("promote split candidate");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v2/identity-candidates?limit=100",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let split_item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(split_candidate_id))
        .expect("split candidate payload");

    assert_eq!(split_item["candidate_kind"], "split_person");
    assert_eq!(split_item["review_state"], "suggested");
    let evidence_summary = split_item["evidence_summary"]
        .as_str()
        .expect("evidence summary");
    assert!(evidence_summary.starts_with("Previously confirmed merge can be split:"));
    assert!(evidence_summary.contains(&left.person_id));
    assert!(evidence_summary.contains(&right.person_id));
}

#[tokio::test]
async fn put_identity_candidate_review_requires_actor_and_confirms_candidate() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person identity review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Review Api {suffix}");

    let left = person_store
        .upsert_email_person(&format!("review-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_person(&format!("review-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let command_id = format!("identity-api-confirm-{suffix}");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let missing_actor = app
        .clone()
        .oneshot(json_put_request_with_token(
            &format!("/api/v2/identity-candidates/{identity_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(missing_actor.status(), StatusCode::BAD_REQUEST);
    let missing_actor_body = json_body(missing_actor).await;
    assert_eq!(
        missing_actor_body,
        json!({
            "error": "invalid_actor_id",
            "message": "missing or invalid x-hermes-actor-id header"
        })
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            &format!("/api/v2/identity-candidates/{identity_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "identity_candidate_id": identity_candidate_id,
            "review_state": "user_confirmed",
            "event_id": format!("person_identity_review:{command_id}"),
        })
    );
}

#[tokio::test]
async fn person_identity_returns_confirmed_links_for_person() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person identity detail API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Detail Api {suffix}");

    let left = person_store
        .upsert_email_person(&format!("detail-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_person(&format!("detail-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v2/identity-candidates/{identity_candidate_id}/review"),
            json!({
                "command_id": format!("identity-detail-confirm-{suffix}"),
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v2/persons/{}/identity", left.person_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());
    let item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(identity_candidate_id))
        .expect("confirmed identity candidate");
    assert_eq!(item["review_state"], "user_confirmed");
}

#[derive(Clone)]
struct PersonIdentityApiContext {
    person_store: PersonProjectionStore,
}

fn config_with_api_token() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN)])
        .expect("valid local API token")
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
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request_with_token(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn json_put_request_with_actor(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn seed_normalized_persons(
    pool: &PgPool,
    left_person_id: &str,
    right_person_id: &str,
    display_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE persons
        SET display_name = $1
        WHERE person_id = $2 OR person_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_person_id)
    .bind(right_person_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn promote_identity_candidate(
    pool: &PgPool,
    identity_candidate_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE person_identity_candidates
        SET updated_at = clock_timestamp()
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(pool)
    .await
    .map(|result| {
        assert_eq!(
            result.rows_affected(),
            1,
            "identity candidate should exist before list promotion"
        );
    })?;

    Ok(())
}

fn identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_persons:{left_person_id}:{right_person_id}")
}

fn split_identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:split_person:{left_person_id}:{right_person_id}")
}

fn ordered_person_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
