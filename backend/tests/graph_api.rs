use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tower::ServiceExt;
use url::Url;

use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::graph::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};
use hermes_hub_backend::storage::Database;
use hermes_hub_backend::{build_router, build_router_with_database};

const LOCAL_API_TOKEN: &str = "graph-api-test-token";
const LOCAL_API_ACTOR_ID: &str = "graph-api-test-client";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";
const EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT: usize = 100;

#[tokio::test]
async fn graph_summary_rejects_missing_local_api_token() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v2/graph/summary"))
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
async fn graph_summary_rejects_missing_local_api_actor_id() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_without_actor(
            "/api/v2/graph/summary",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_actor_id",
            "message": "missing or invalid x-hermes-actor-id header"
        })
    );
}

#[tokio::test]
async fn graph_search_rejects_missing_local_api_token_before_missing_query_validation() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v2/graph/search"))
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
async fn graph_neighborhood_rejects_missing_local_api_token_before_malformed_query_validation() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request(
            "/api/v2/graph/neighborhood?node_id=graph:node:v1:person:alex&depth=not-a-number",
        ))
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
async fn graph_summary_returns_empty_state_for_empty_database() {
    let Some(context) = live_graph_api_context("empty summary").await else {
        return;
    };

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v2/graph/summary",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["node_counts"], json!([]));
    assert_eq!(body["edge_counts"], json!([]));
    assert_eq!(body["evidence_count"], json!(0));
    assert_eq!(body["latest_projection_at"], Value::Null);
    assert_eq!(body["is_empty"], json!(true));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_search_returns_matching_nodes() {
    let Some(context) = live_graph_api_context("search").await else {
        return;
    };
    let suffix = unique_suffix();
    let alex = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("contact:alex:{suffix}"),
            format!("Alex Morgan {suffix}"),
        ))
        .await
        .expect("alex node");
    context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("contact:blair:{suffix}"),
            format!("Blair Morgan {suffix}"),
        ))
        .await
        .expect("blair node");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v2/graph/search?q=alex",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body.as_array().expect("node array");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["node_id"], json!(alex.node_id));
    assert_eq!(nodes[0]["label"], json!(alex.label));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_search_rejects_empty_query() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v2/graph/search?q=",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_graph_query",
            "message": "q must not be empty"
        })
    );
}

#[tokio::test]
async fn graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence() {
    let Some(context) = live_graph_api_context("neighborhood").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("contact:alex-neighborhood:{suffix}"),
            format!("Alex Neighborhood {suffix}"),
        ))
        .await
        .expect("person node");
    let email = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("alex-neighborhood-{suffix}@example.com"),
            format!("alex-neighborhood-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let edge = context
        .store
        .upsert_edge_with_evidence(
            &NewGraphEdge::new(
                person.node_id.clone(),
                email.node_id.clone(),
                RelationshipType::PersonHasEmailAddress,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[NewGraphEvidence::new(
                GraphEvidenceSourceKind::Contact,
                format!("contact-source:{suffix}"),
            )
            .excerpt("confirmed by contact record")
            .metadata(json!({"source": "graph_api_test"}))],
        )
        .await
        .expect("graph edge");

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v2/graph/neighborhood?node_id={}&depth=1",
                person.node_id
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["selected_node"]["node_id"], json!(person.node_id));
    assert_eq!(body["selected_node"]["label"], json!(person.label));
    assert_eq!(
        body["edge_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
    );
    assert_eq!(body["truncated"], json!(false));

    let nodes = body["nodes"].as_array().expect("node array");
    assert_eq!(nodes.len(), 1);
    assert!(nodes.iter().all(|node| node["node_id"] != person.node_id));
    assert!(nodes.iter().any(|node| node["node_id"] == email.node_id));

    let edges = body["edges"].as_array().expect("edge array");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["edge_id"], json!(edge.edge_id));
    assert_eq!(edges[0]["source_node_id"], json!(person.node_id));
    assert_eq!(edges[0]["target_node_id"], json!(email.node_id));

    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0]["edge_id"], json!(edge.edge_id));
    assert_eq!(evidence[0]["source_kind"], json!("contact"));
    assert_eq!(
        evidence[0]["source_id"],
        json!(format!("contact-source:{suffix}"))
    );
    assert_eq!(evidence[0]["excerpt"], json!("confirmed by contact record"));
    assert_eq!(evidence[0]["metadata"], json!({"source": "graph_api_test"}));

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_caps_depth_one_edges_nodes_and_evidence() {
    let Some(context) = live_graph_api_context("neighborhood cap").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = context
        .store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("contact:alex-neighborhood-cap:{suffix}"),
            format!("Alex Neighborhood Cap {suffix}"),
        ))
        .await
        .expect("person node");

    for index in 0..=EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT {
        let email = context
            .store
            .upsert_node(&NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                format!("alex-neighborhood-cap-{suffix}-{index:03}@example.com"),
                format!("alex-neighborhood-cap-{suffix}-{index:03}@example.com"),
            ))
            .await
            .expect("email node");
        context
            .store
            .upsert_edge_with_evidence(
                &NewGraphEdge::new(
                    person.node_id.clone(),
                    email.node_id,
                    RelationshipType::PersonHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Contact,
                    format!("contact-source:{suffix}:{index:03}"),
                )],
            )
            .await
            .expect("graph edge");
    }

    let response = context
        .app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v2/graph/neighborhood?node_id={}", person.node_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let nodes = body["nodes"].as_array().expect("node array");
    let edges = body["edges"].as_array().expect("edge array");
    let evidence = body["evidence"].as_array().expect("evidence array");
    assert_eq!(
        body["edge_limit"],
        json!(EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
    );
    assert_eq!(body["truncated"], json!(true));
    assert_eq!(nodes.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);
    assert!(nodes.iter().all(|node| node["node_id"] != person.node_id));
    assert_eq!(edges.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);
    assert_eq!(evidence.len(), EXPECTED_GRAPH_NEIGHBORHOOD_EDGE_LIMIT);

    context.cleanup().await;
}

#[tokio::test]
async fn graph_neighborhood_returns_not_found_when_node_id_is_missing() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v2/graph/neighborhood?depth=1",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "graph_node_not_found",
            "message": "graph node was not found"
        })
    );
}

#[tokio::test]
async fn graph_neighborhood_rejects_unsupported_depth() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v2/graph/neighborhood?node_id=graph:node:v1:person:alex&depth=2",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_graph_query",
            "message": "depth supports only 1"
        })
    );
}

struct LiveGraphApiContext {
    app: Router,
    store: GraphStore,
    pool: PgPool,
    admin_pool: PgPool,
    database_name: String,
}

impl LiveGraphApiContext {
    async fn cleanup(self) {
        let Self {
            app,
            store,
            pool,
            admin_pool,
            database_name,
        } = self;
        drop(app);
        drop(store);
        pool.close().await;
        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS {}",
            quote_identifier(&database_name)
        ))
        .execute(&admin_pool)
        .await
        .expect("drop graph API test database");
        admin_pool.close().await;
    }
}

async fn live_graph_api_context(test_name: &str) -> Option<LiveGraphApiContext> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live graph API {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let admin_database_url = database_url_with_database(&database_url, "postgres");
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&admin_database_url)
        .await
        .expect("admin database connection");
    let database_name = format!("hermes_graph_api_test_{}", unique_suffix());
    assert_safe_identifier(&database_name);
    sqlx::query(&format!(
        "CREATE DATABASE {}",
        quote_identifier(&database_name)
    ))
    .execute(&admin_pool)
    .await
    .expect("create graph API test database");

    let test_database_url = database_url_with_database(&database_url, &database_name);
    let database = Database::connect(Some(&test_database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = GraphStore::new(pool.clone());
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", test_database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    Some(LiveGraphApiContext {
        app,
        store,
        pool,
        admin_pool,
        database_name,
    })
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

fn get_request_with_token_without_actor(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn database_url_with_database(database_url: &str, database_name: &str) -> String {
    let mut url = Url::parse(database_url).expect("valid database URL");
    url.set_path(database_name);
    url.set_query(None);
    url.to_string()
}

fn assert_safe_identifier(identifier: &str) {
    assert!(
        identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
        "test database identifier must be simple ASCII"
    );
}

fn quote_identifier(identifier: &str) -> String {
    format!(r#""{}""#, identifier.replace('"', r#""""#))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
