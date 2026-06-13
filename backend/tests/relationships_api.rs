use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::graph::core::{GraphNodeKind, node_id};
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEvidenceSourceKind,
    RelationshipReviewState, RelationshipStore,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "relationships-api-test-token";

#[tokio::test]
async fn relationships_list_returns_entity_scoped_relationships() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live relationships API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_persona_relationship(&pool, suffix).await;
    let source_entity_id = &stored.source_entity_id;

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/relationships?entity_kind=persona&entity_id={source_entity_id}&limit=10"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["relationship_id"] == json!(stored.relationship_id))
        .expect("seeded relationship");

    assert_eq!(item["source_entity_kind"], "persona");
    assert_eq!(item["source_entity_id"], stored.source_entity_id);
    assert_eq!(item["target_entity_kind"], "persona");
    assert_eq!(item["target_entity_id"], stored.target_entity_id);
    assert_eq!(item["relationship_type"], "collaborates_with");
    assert_eq!(item["review_state"], "suggested");
}

#[tokio::test]
async fn relationships_list_returns_global_suggested_review_items() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live relationships global review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested = seed_persona_relationship_with_state(
        &pool,
        suffix,
        "global_review_suggested",
        RelationshipReviewState::Suggested,
    )
    .await;
    let confirmed = seed_persona_relationship_with_state(
        &pool,
        suffix + 1,
        "global_review_confirmed",
        RelationshipReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/relationships?review_state=suggested&limit=10",
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
            .any(|item| item["relationship_id"] == json!(suggested.relationship_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["relationship_id"] != json!(confirmed.relationship_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_relationship_review_updates_relationship_and_graph_projection() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live relationship review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_persona_relationship(&pool, suffix).await;
    let relationship_id = path_segment(&stored.relationship_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/relationships/{relationship_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["relationship_id"], stored.relationship_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert_eq!(body["trust_score"], json!(0.72));
    assert_eq!(body["strength_score"], json!(0.66));

    let stored_review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&stored.relationship_id)
            .fetch_one(&pool)
            .await
            .expect("relationship review state");
    let graph_row = sqlx::query(
        r#"
        SELECT review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(node_id(GraphNodeKind::Person, &stored.source_entity_id))
    .bind(node_id(GraphNodeKind::Person, &stored.target_entity_id))
    .fetch_one(&pool)
    .await
    .expect("relationship graph edge");
    let graph_review_state: String = graph_row.try_get("review_state").expect("graph review");
    let graph_properties: Value = graph_row.try_get("properties").expect("graph properties");

    assert_eq!(stored_review_state, "user_confirmed");
    assert_eq!(graph_review_state, "user_confirmed");
    assert_eq!(
        graph_properties["relationship_id"],
        json!(stored.relationship_id)
    );
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

async fn seed_persona_relationship(pool: &PgPool, suffix: u128) -> Relationship {
    seed_persona_relationship_with_state(
        pool,
        suffix,
        "collaborates_with",
        RelationshipReviewState::Suggested,
    )
    .await
}

async fn seed_persona_relationship_with_state(
    pool: &PgPool,
    suffix: u128,
    relationship_type: &str,
    review_state: RelationshipReviewState,
) -> Relationship {
    let person_store = PersonProjectionStore::new(pool.clone());
    let source = person_store
        .upsert_email_person(&format!("relationship-api-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_person(&format!("relationship-api-target-{suffix}@example.com"))
        .await
        .expect("target persona");
    let relationship = NewRelationship::between_personas(
        &source.person_id,
        &target.person_id,
        relationship_type,
        0.72,
        0.66,
        0.88,
        review_state,
    )
    .metadata(json!({"source": "relationships_api_test"}));
    let evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        format!("message:relationship-api:{suffix}"),
    )
    .excerpt("They agreed to collaborate on the relationship API.")
    .metadata(json!({"source": "relationships_api_test"}));

    RelationshipStore::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await
        .expect("seed relationship")
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
