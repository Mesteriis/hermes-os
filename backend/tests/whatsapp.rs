use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use serde_json::json;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::secrets::SecretKind;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "whatsapp-api-test-secret";

#[test]
fn whatsapp_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("whatsapp_web").expect("whatsapp web provider"),
        CommunicationProviderKind::WhatsappWeb
    );
    assert!(CommunicationProviderKind::WhatsappWeb.is_whatsapp());
    assert!(!CommunicationProviderKind::WhatsappWeb.is_email());
    assert!(!CommunicationProviderKind::WhatsappWeb.is_telegram());

    assert!(
        ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::PrivateKey)
    );
    assert!(
        ProviderAccountSecretPurpose::WhatsappWebSessionKey.accepts_secret_kind(SecretKind::Other)
    );
    assert!(
        !ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::Password)
    );
    assert!(
        !ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::ApiToken)
    );
}

#[tokio::test]
async fn whatsapp_fixture_message_ingestion_refreshes_decision_and_obligation_candidates_against_postgres()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live WhatsApp candidate refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-candidate-{suffix}");
    let chat_id = format!("wa-candidate-chat-{suffix}");
    let decision_title = format!("Use WhatsApp evidence for shared memory {suffix}");
    let decision_rationale = "chat context must feed the same domain model";
    let obligation_statement = format!("send the WhatsApp alignment note {suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/whatsapp/accounts/fixture",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Candidate Source",
                "external_account_id": format!("wa-candidate-{suffix}"),
                "device_name": "Hermes Desktop Fixture",
                "local_state_path": format!("docker/data/whatsapp/candidate-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/whatsapp/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("wa-candidate-message-{suffix}"),
                "chat_title": "WhatsApp Candidate Review",
                "sender_id": format!("whatsapp-candidate-sender-{suffix}"),
                "sender_display_name": "WhatsApp Candidate Sender",
                "text": format!(
                    "Decision: {decision_title} because {decision_rationale}. I will {obligation_statement} by Friday 5pm."
                ),
                "import_batch_id": format!("whatsapp-candidate-fixture-{suffix}"),
                "occurred_at": "2026-06-06T13:30:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let decision_row: (String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT d.title, d.rationale, d.review_state, e.source_kind, e.source_id
        FROM decisions d
        JOIN decision_evidence e ON e.decision_id = d.decision_id
        WHERE e.source_kind = 'communication'
          AND e.source_id = $1
          AND d.title = $2
        "#,
    )
    .bind(&message_id)
    .bind(&decision_title)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp message should create a suggested Decision candidate");
    assert_eq!(decision_row.1, decision_rationale);
    assert_eq!(decision_row.2, "suggested");
    assert_eq!(decision_row.3, "communication");
    assert_eq!(decision_row.4, message_id);

    let task_candidate_row: (String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text
        FROM task_candidates
        WHERE source_kind = 'message'
          AND source_id = $1
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp message should create an obligation-derived task candidate");
    assert_eq!(task_candidate_row.0, obligation_statement);
    assert_eq!(task_candidate_row.1, "suggested");
    assert_eq!(task_candidate_row.2, "obligation_task");
    assert_eq!(task_candidate_row.3.as_deref(), Some("Friday 5pm"));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&obligation_statement)
            .fetch_one(&pool)
            .await
            .expect("accepted obligation count");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn whatsapp_web_session_metadata_is_account_scoped_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live WhatsApp Web smoke test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-web-{suffix}");
    let session_id = format!("whatsapp-session-{suffix}");
    let secret_ref = format!("secret-whatsapp-session-{suffix}");

    store
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::WhatsappWeb,
                "WhatsApp Web fixture",
                format!("wa-device-{suffix}"),
            )
            .config(json!({
                "runtime": "fixture",
                "local_state_path": format!("docker/data/whatsapp/{account_id}")
            })),
        )
        .await
        .expect("store WhatsApp Web provider account");

    sqlx::query(
        r#"
        INSERT INTO secret_references (
            secret_ref,
            secret_kind,
            store_kind,
            label,
            metadata
        )
        VALUES ($1, 'private_key', 'test_double', 'WhatsApp Web session key', '{}'::jsonb)
        "#,
    )
    .bind(&secret_ref)
    .execute(&pool)
    .await
    .expect("store session secret reference metadata");

    let binding = store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
            &secret_ref,
        ))
        .await
        .expect("bind WhatsApp Web session key");
    assert_eq!(
        binding.secret_purpose,
        ProviderAccountSecretPurpose::WhatsappWebSessionKey
    );

    sqlx::query(
        r#"
        INSERT INTO whatsapp_web_sessions (
            session_id,
            account_id,
            device_name,
            companion_runtime,
            link_state,
            local_state_path,
            metadata
        )
        VALUES ($1, $2, 'Hermes Desktop', 'fixture', 'fixture', $3, $4)
        "#,
    )
    .bind(&session_id)
    .bind(&account_id)
    .bind(format!("docker/data/whatsapp/{account_id}"))
    .bind(json!({"source": "whatsapp-test"}))
    .execute(&pool)
    .await
    .expect("store WhatsApp Web session metadata");

    let stored_state: String =
        sqlx::query_scalar("SELECT link_state FROM whatsapp_web_sessions WHERE account_id = $1")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .expect("fetch WhatsApp Web session state");
    assert_eq!(stored_state, "fixture");
}

#[tokio::test]
async fn whatsapp_api_exercises_web_fixture_foundation() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live WhatsApp API smoke test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-web-api-{suffix}");
    let chat_id = format!("wa-chat-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/whatsapp/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(capabilities_body["runtime_mode"], json!("fixture"));
    assert_capability_status(
        &capabilities_body,
        "whatsapp_web_fixture_runtime",
        "available",
        true,
    );
    assert_capability_status(
        &capabilities_body,
        "whatsapp_web_fixture_ingestion",
        "available",
        true,
    );
    assert_capability_status(
        &capabilities_body,
        "whatsapp_web_live_runtime",
        "blocked",
        false,
    );
    assert_capability_status(
        &capabilities_body,
        "whatsapp_web_live_send",
        "blocked",
        false,
    );
    assert!(
        capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "hidden_web_scraping")
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/whatsapp/accounts/fixture",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Web",
                "external_account_id": format!("wa-device-{suffix}"),
                "device_name": "Hermes Desktop Fixture",
                "local_state_path": format!("docker/data/whatsapp/{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(account_body["provider_kind"], json!("whatsapp_web"));
    assert_eq!(account_body["runtime"], json!("fixture"));
    assert_eq!(account_body["session"]["link_state"], json!("fixture"));

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/whatsapp/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("wa-message-{suffix}"),
                "chat_title": "WhatsApp Planning",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "WhatsApp Fixture",
                "text": "Please carry WhatsApp Web context into graph-backed recall.",
                "import_batch_id": format!("whatsapp-fixture-{suffix}"),
                "occurred_at": "2026-06-06T13:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    assert!(
        message_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v5:whatsapp_web:")
    );

    let sessions_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/whatsapp/sessions?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("sessions response");
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_body = json_body(sessions_response).await;
    assert_eq!(sessions_body["items"][0]["account_id"], json!(account_id));
    assert_eq!(
        sessions_body["items"][0]["last_sync_at"],
        json!("2026-06-06T13:00:00Z")
    );

    let messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/whatsapp/messages?account_id={account_id}&provider_chat_id={chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_body = json_body(messages_response).await;
    assert_eq!(
        messages_body["items"][0]["channel_kind"],
        json!("whatsapp_web")
    );
    assert_eq!(
        messages_body["items"][0]["provider_chat_id"],
        json!(chat_id)
    );

    let projected_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM communication_messages WHERE account_id = $1 AND channel_kind = 'whatsapp_web'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("projected WhatsApp Web count");
    assert_eq!(projected_count, 1);
}

fn assert_capability_status(body: &Value, capability: &str, status: &str, closure_gate: bool) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    assert!(
        capabilities.iter().any(|item| {
            item["capability"] == capability
                && item["status"] == status
                && item["closure_gate"] == closure_gate
        }),
        "expected capability {capability} to have status {status} and closure_gate {closure_gate}"
    );
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

fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos()
        .to_string()
}
