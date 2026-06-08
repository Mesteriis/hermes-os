use std::env;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tokio::net::TcpListener;
use tower::ServiceExt;

use hermes_hub_backend::ai::core::{
    AiRunStore, NewSemanticEmbedding, SemanticEmbeddingStore, SemanticSourceKind,
};
use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::domains::projects::core::{NewProject, ProjectStore};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::settings::ApplicationSettingsStore;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "ai-api-test-token";

#[tokio::test]
async fn pgvector_semantic_store_indexes_and_searches_sources_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live semantic store test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let store = SemanticEmbeddingStore::new(pool.clone());
    let embedding_model = format!("qwen3-embedding:4b-semantic-{suffix}");

    let extension_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(&pool)
            .await
            .expect("vector extension");
    assert!(extension_exists);

    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            title: "Roadmap planning",
            source_text: "Discussed Hermes Hub AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("upsert first embedding");
    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Document,
            source_id: &format!("document-semantic-{suffix}"),
            title: "Garden notes",
            source_text: "Tomatoes need watering this weekend.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(8),
            graph_node_id: None,
        })
        .await
        .expect("upsert second embedding");

    let indexed = store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            title: "Roadmap planning",
            source_text: "Discussed Hermes Hub AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("idempotent upsert");
    assert_eq!(indexed.source_kind, "message");
    assert_eq!(indexed.embedding_dimension, 2560);

    let results = store
        .search(&embedding_model, &unit_embedding(0), 5)
        .await
        .expect("search");
    assert_eq!(results[0].source_kind, "message");
    assert_eq!(results[0].source_id, format!("message-semantic-{suffix}"));
    assert!(results[0].score > results[1].score);
    assert_eq!(
        results[0].graph_node_id,
        Some(format!("graph:message:{suffix}"))
    );
}

#[tokio::test]
async fn ai_answer_api_returns_source_backed_answer_and_persists_run() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live AI answer API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let retrieval_token = format!("V3AIAnswer{suffix}");
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("ai-answer-{suffix}@example.com"),
        &[format!("ai-recipient-{suffix}@example.com")],
        &format!("provider-ai-answer-{suffix}"),
        &format!("Hermes AI roadmap {retrieval_token}"),
        &format!("The V3 AI plan for {retrieval_token} uses Ollama and source-backed citations."),
    )
    .await;

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
            ("HERMES_OLLAMA_BASE_URL", ollama_base_url.as_str()),
            ("HERMES_OLLAMA_CHAT_MODEL", "qwen3:4b"),
            ("HERMES_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/answers",
            json!({
                "command_id": format!("answer-{suffix}"),
                "query": format!("V3 AI plan for {retrieval_token}"),
                "agent_id": "MNEMOSYNE"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["agent_id"], json!("MNEMOSYNE"));
    assert_eq!(body["status"], json!("completed"));
    assert_eq!(body["model"], json!("qwen3:4b"));
    assert_eq!(body["embedding_model"], json!("qwen3-embedding:4b"));
    assert_eq!(body["answer"], json!("Hermes Hub V3 is source-backed."));
    assert!(body["duration_ms"].as_i64().expect("duration") >= 0);

    let citations = body["citations"].as_array().expect("citations");
    assert!(!citations.is_empty());
    assert!(citations.iter().any(|citation| {
        citation["source_kind"] == json!("message") && citation["source_id"] == json!(message_id)
    }));

    let run_id = body["run_id"].as_str().expect("run id");
    let stored = AiRunStore::new(pool.clone())
        .get_run(run_id)
        .await
        .expect("load run")
        .expect("stored run");
    assert_eq!(
        stored.answer.as_deref(),
        Some("Hermes Hub V3 is source-backed.")
    );
    assert_eq!(stored.status, "completed");
}

#[tokio::test]
async fn ai_task_refresh_creates_suggested_candidates_without_active_tasks() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live AI task refresh API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("ai-task-{suffix}@example.com"),
        &[format!("ai-task-recipient-{suffix}@example.com")],
        &format!("provider-ai-task-{suffix}"),
        "AI task source",
        &format!("Please review the V3 implementation checklist {suffix}."),
    )
    .await;

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
            ("HERMES_OLLAMA_BASE_URL", ollama_base_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/task-candidates/refresh",
            json!({
                "command_id": format!("task-refresh-{suffix}"),
                "query": format!("Please review the V3 implementation checklist {suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["status"], json!("completed"));
    assert_eq!(body["created_count"], json!(1));

    let candidate = sqlx::query(
        "SELECT task_candidate_id, review_state, agent_run_id FROM task_candidates WHERE source_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("candidate");
    assert_eq!(candidate.get::<String, _>("review_state"), "suggested");
    assert!(candidate.get::<Option<String>, _>("agent_run_id").is_some());

    let active_task_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&pool)
            .await
            .expect("active task count");
    assert_eq!(active_task_count, 0);
}

#[tokio::test]
async fn ai_meeting_prep_returns_briefing_without_calendar_dependency() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live AI meeting prep API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let project_id = format!("project:v1:ai-meeting:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("AI Meeting Project {suffix}"),
                "Product Development",
                "Meeting prep project",
                "Alex Morgan",
                vec![format!("MeetingPrep{suffix}")],
            )
            .progress(42),
        )
        .await
        .expect("project");
    seed_document(
        &pool,
        &format!("ai_meeting_doc_{suffix}"),
        &format!("MeetingPrep{suffix} notes"),
        "Discuss V3 AI risks and validation.",
    )
    .await;

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
            ("HERMES_OLLAMA_BASE_URL", ollama_base_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/meeting-prep",
            json!({
                "command_id": format!("meeting-prep-{suffix}"),
                "topic": "V3 AI implementation review",
                "project_id": project_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["agent_id"], json!("HESTIA"));
    assert_eq!(body["status"], json!("completed"));
    assert_eq!(
        body["briefing"],
        json!("Discuss V3 risks and validation evidence.")
    );
    assert!(!body["citations"].as_array().expect("citations").is_empty());
}

#[tokio::test]
async fn ai_status_and_agents_are_protected() {
    let app = build_router(config_with_api_token());

    let missing_token = app
        .clone()
        .oneshot(get_request("/api/v1/ai/status"))
        .await
        .expect("response");
    assert_eq!(missing_token.status(), StatusCode::FORBIDDEN);

    let agents = app
        .oneshot(get_request_with_token("/api/v1/ai/agents", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(agents.status(), StatusCode::OK);
    let body = json_body(agents).await;
    let items = body["items"].as_array().expect("agents");
    assert_eq!(items.len(), 4);
    assert_eq!(items[0]["agent_id"], json!("HESTIA"));
}

async fn spawn_fake_ollama() -> String {
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

async fn configure_fake_ollama_setting(pool: &PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "hermes-frontend",
        )
        .await
        .expect("fake Ollama setting");
}

fn unit_embedding(active_index: usize) -> Vec<f32> {
    let mut embedding = vec![0.0; 2560];
    embedding[active_index] = 1.0;
    embedding
}

async fn seed_message(
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

async fn seed_document(pool: &PgPool, fingerprint: &str, title: &str, text: &str) -> String {
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(fingerprint, title, text))
        .await
        .expect("document")
        .document_id
}

fn config_with_api_token() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config")
}

fn get_request(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-hermes-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}
