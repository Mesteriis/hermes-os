use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router;
use hermes_hub_backend::platform::config::AppConfig;

const LOCAL_API_TOKEN: &str = "ai-control-center-test-token";

fn cfg() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config")
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-hermes-secret", LOCAL_API_TOKEN)
        .header("x-hermes-actor-id", "hermes-frontend")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", LOCAL_API_TOKEN)
        .header("x-hermes-actor-id", "hermes-frontend")
        .body(Body::empty())
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json response")
}

#[tokio::test]
async fn ai_settings_read_endpoints_exist_without_database() {
    let app = build_router(cfg());

    for path in [
        "/api/v1/ai/settings/overview",
        "/api/v1/ai/providers",
        "/api/v1/ai/models",
        "/api/v1/ai/prompts",
    ] {
        let response = app
            .clone()
            .oneshot(get_request(path))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{path}");
        let body = response_json(response).await;
        assert_eq!(body["error"], json!("database_not_configured"), "{path}");
    }
}

#[tokio::test]
async fn ai_settings_write_endpoints_exist_without_database() {
    let app = build_router(cfg());

    let requests = [
        json_request(
            Method::POST,
            "/api/v1/ai/providers",
            json!({
                "provider_kind": "api",
                "provider_key": "openai",
                "display_name": "OpenAI",
                "base_url": "https://api.openai.com/v1"
            }),
        ),
        json_request(
            Method::PATCH,
            "/api/v1/ai/providers/provider:missing",
            json!({"enabled": true}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/providers/provider:missing/test",
            json!({}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/providers/provider:missing/sync-models",
            json!({}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/providers/provider:missing/consent",
            json!({"consented": true}),
        ),
        json_request(
            Method::PUT,
            "/api/v1/ai/model-routes/default_chat",
            json!({
                "provider_id": "provider:missing",
                "model_key": "model:missing"
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts",
            json!({
                "prompt_id": "prompt:test",
                "name": "Test prompt",
                "entity_scope": "global",
                "capability_slot": "default_chat"
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts/prompt:test/versions",
            json!({
                "body_template": "Answer {{query}}",
                "variables": ["query"]
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts/prompt:test/activate",
            json!({"prompt_version_id": "prompt-version:test"}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts/prompt:test/test",
            json!({
                "prompt_version_id": "prompt-version:test",
                "provider_id": "provider:missing",
                "model_key": "model:missing",
                "variables": {"query": "hello"}
            }),
        ),
    ];

    for request in requests {
        let response = app.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_json(response).await;
        assert_eq!(body["error"], json!("database_not_configured"));
    }
}
