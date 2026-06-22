use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Request, StatusCode, header};
pub(crate) use axum::routing::{get, post};
pub(crate) use axum::{Json, Router};
pub(crate) use hermes_hub_backend::ai::core::{
    AiRunStore, NewSemanticEmbedding, SemanticEmbeddingStore, SemanticSourceKind,
};
pub(crate) use hermes_hub_backend::app::{build_router, build_router_with_database};
pub(crate) use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
pub(crate) use hermes_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
pub(crate) use hermes_hub_backend::domains::documents::core::{
    DocumentImportStore, NewDocumentImport,
};
pub(crate) use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
pub(crate) use hermes_hub_backend::domains::projects::core::{NewProject, ProjectStore};
pub(crate) use hermes_hub_backend::platform::config::AppConfig;
pub(crate) use hermes_hub_backend::platform::settings::ApplicationSettingsStore;
pub(crate) use hermes_hub_backend::platform::storage::Database;
pub(crate) use serde_json::{Value, json};
pub(crate) use sqlx::Row;
pub(crate) use sqlx::postgres::PgPool;
pub(crate) use tokio::net::TcpListener;
pub(crate) use tower::ServiceExt;

pub(crate) const LOCAL_API_TOKEN: &str = "ai-api-test-token";
pub(crate) static AI_RUNTIME_TEST_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

pub(crate) async fn spawn_fake_ollama() -> String {
    let app = Router::new()
        .route(
            "/api/version",
            get(|| async { Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            get(|| async {
                Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/embed",
            post(|Json(_body): Json<Value>| async {
                Json(json!({
                    "model": "qwen3-embedding:4b",
                    "embeddings": [unit_embedding(0)],
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 8u32
                }))
            }),
        )
        .route(
            "/api/chat",
            post(|Json(body): Json<Value>| async move {
                let text = body["messages"]
                    .as_array()
                    .and_then(|messages| messages.last())
                    .and_then(|message| message["content"].as_str())
                    .unwrap_or_default();
                let content = if text.contains("Return JSON task candidates") {
                    r#"[{"source_kind":"message","source_id":"__first__","title":"Review the V3 implementation checklist","evidence_excerpt":"Please review the V3 implementation checklist.","confidence":0.82}]"#
                } else if text.contains("meeting briefing") {
                    "Discuss V3 risks and validation evidence."
                } else {
                    "Hermes Hub V3 is source-backed."
                };

                Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": content },
                    "done": true,
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 16u32,
                    "eval_count": 8u32
                }))
            }),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}

pub(crate) async fn configure_fake_ollama_setting(pool: &PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "hermes-frontend",
        )
        .await
        .expect("fake Ollama setting");
}

pub(crate) fn unit_embedding(active_index: usize) -> Vec<f32> {
    let mut embedding = vec![0.0; 2560];
    embedding[active_index] = 1.0;
    embedding
}

pub(crate) async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let account_id = format!("ai-account-{suffix}");

    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id.clone(),
                EmailProviderKind::Imap,
                format!("AI Account {suffix}"),
                format!("ai-external-{suffix}"),
            )
            .config(json!({
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": format!("ai-{suffix}@example.com")
            })),
        )
        .await
        .expect("provider account");

    let raw_record = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_ai_{suffix}_{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("fingerprint-ai-{suffix}-{provider_record_id}"),
                format!("batch-ai-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body
                }),
            )
            .provenance(json!({"source":"ai_test"})),
        )
        .await
        .expect("raw record");

    let projected = project_raw_email_message(&message_store, &raw_record)
        .await
        .expect("project raw message");

    projected.message_id
}

pub(crate) async fn seed_document(
    pool: &PgPool,
    fingerprint: &str,
    title: &str,
    text: &str,
) -> String {
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(fingerprint, title, text))
        .await
        .expect("document")
        .document_id
}

pub(crate) fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub(crate) fn get_request(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-hermes-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}
