use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "v1comms-bilingual-reply-flow-test-token";

#[tokio::test]
async fn v1_bilingual_reply_flow_returns_review_contract_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо, мы проверим контракт сегодня.",
                "tone": "business"
            }),
        ))
        .await
        .expect("bilingual reply flow response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["subject"], "Re: Contrato");
    assert_eq!(body["tone"], "business");
    assert_eq!(body["reply_language"], "ru");
    assert_eq!(body["send_ready"], false);
    assert_eq!(body["original"]["language"], "es");
    assert!(
        body["original"]["text"]
            .as_str()
            .expect("original text")
            .contains("Hola equipo")
    );
    assert_eq!(body["translation"]["target"], "ru");
    assert_eq!(body["translation"]["translated"], false);
    assert_eq!(body["translation"]["text"], Value::Null);
    assert_eq!(body["translation"]["model"], Value::Null);
    assert_eq!(
        body["translation"]["reason"],
        "translation runtime unavailable"
    );
    assert_eq!(body["reply"]["language"], "ru");
    assert_eq!(body["reply"]["tone"], "business");
    assert_eq!(
        body["reply"]["text"],
        "Спасибо, мы проверим контракт сегодня."
    );
    assert_eq!(body["back_translation"]["target"], "es");
    assert_eq!(body["back_translation"]["translated"], false);
    assert_eq!(body["back_translation"]["text"], Value::Null);
    assert_eq!(
        body["back_translation"]["reason"],
        "translation runtime unavailable"
    );
}

#[tokio::test]
async fn v1_bilingual_reply_flow_rejects_unsupported_tone_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо.",
                "tone": "casual"
            }),
        ))
        .await
        .expect("bilingual reply flow rejection response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(body["message"], "unsupported bilingual reply tone");
}

struct SeededMessage {
    message_id: String,
}

async fn seed_message(pool: sqlx::PgPool) -> SeededMessage {
    let suffix = uid();
    let account_id = format!("acct-bilingual-reply-flow-{suffix}");
    let provider_record_id = format!("provider-bilingual-reply-flow-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Bilingual Reply Flow Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            &account_id,
            "email_message",
            &provider_record_id,
            format!("sha256:{:0>64}", "e"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Contrato",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Hola equipo, gracias por enviar el contrato. Saludos."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    SeededMessage { message_id }
}

async fn router(database_url: &str) -> axum::Router {
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

fn post(uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", LOCAL_API_TOKEN)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
