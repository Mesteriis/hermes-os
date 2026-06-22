use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use serde_json::Value;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

pub const LOCAL_API_TOKEN: &str = "persons-api-test-token";

pub fn config_with_api_token() -> AppConfig {
    app_config_with_pairs(Vec::new())
}

pub fn app_config_with_pairs(mut extra_pairs: Vec<(&'static str, String)>) -> AppConfig {
    let suffix = unique_suffix();
    let vault_home = format!("/tmp/hermes-persons-api-vault-{suffix}");
    let dev_key_path = format!("{vault_home}/dev.key");
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
        .with_test_dev_vault_paths(vault_home, dev_key_path)
        .with_test_pairs(extra_pairs.drain(..))
        .expect("valid local API config")
}

pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

pub async fn build_persons_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_persons_app_with_database(database_url, database)
}

pub fn build_persons_app_with_database(database_url: &str, database: Database) -> axum::Router {
    build_router_with_database(
        app_config_with_pairs(Vec::new()).with_test_database_url(database_url),
        database,
    )
}

pub fn build_persons_app_without_database() -> axum::Router {
    build_router(config_with_api_token())
}

pub async fn live_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}
