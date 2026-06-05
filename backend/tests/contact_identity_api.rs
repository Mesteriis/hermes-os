use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::contact_identity::ContactIdentityStore;
use hermes_hub_backend::contacts::ContactProjectionStore;
use hermes_hub_backend::storage::Database;
use hermes_hub_backend::{build_router, build_router_with_database};

const LOCAL_API_TOKEN: &str = "contact-identity-api-test-token";
const LOCAL_API_ACTOR_ID: &str = "contact-identity-api-test-client";
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
        eprintln!("skipping live contact identity API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let context = ContactIdentityApiContext {
        contact_store: ContactProjectionStore::new(pool.clone()),
    };
    let shared_name = format!("Identity Api Candidate {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");
    seed_normalized_contacts(&pool, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let store = ContactIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");

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
            "/api/v2/identity-candidates",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());

    let candidate_id = identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);
    let item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(candidate_id))
        .expect("candidate payload");

    assert_eq!(item["candidate_kind"], "merge_contacts");
    assert_eq!(item["review_state"], "suggested");
    assert_eq!(item["left_contact_id"], json!(left.contact_id));
    assert_eq!(item["right_contact_id"], json!(right.contact_id));
    assert!(item["evidence_summary"].is_string());
    assert!(item["confidence"].is_number());
}

#[tokio::test]
async fn put_identity_candidate_review_requires_actor_and_confirms_candidate() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contact identity review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let contact_store = ContactProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Review Api {suffix}");

    let left = contact_store
        .upsert_email_contact(&format!("review-left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = contact_store
        .upsert_email_contact(&format!("review-right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");
    seed_normalized_contacts(&pool, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let store = ContactIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);
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
            "event_id": format!("contact_identity_review:{command_id}"),
        })
    );
}

#[tokio::test]
async fn contact_identity_returns_confirmed_links_for_contact() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contact identity detail API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let contact_store = ContactProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Detail Api {suffix}");

    let left = contact_store
        .upsert_email_contact(&format!("detail-left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = contact_store
        .upsert_email_contact(&format!("detail-right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");
    seed_normalized_contacts(&pool, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let store = ContactIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);

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
            &format!("/api/v2/contacts/{}/identity", left.contact_id),
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
struct ContactIdentityApiContext {
    contact_store: ContactProjectionStore,
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

async fn seed_normalized_contacts(
    pool: &PgPool,
    left_contact_id: &str,
    right_contact_id: &str,
    display_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE contacts
        SET display_name = $1
        WHERE contact_id = $2 OR contact_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_contact_id)
    .bind(right_contact_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn identity_candidate_id_from_contacts(left_id: &str, right_id: &str) -> String {
    let (left_contact_id, right_contact_id) = ordered_contact_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_contacts:{left_contact_id}:{right_contact_id}")
}

fn ordered_contact_ids(left_id: &str, right_id: &str) -> (String, String) {
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
