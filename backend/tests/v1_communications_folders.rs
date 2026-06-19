use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-folder-test-token";

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", T),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    )
}

fn request(method: Method, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-hermes-secret", T);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(Body::from(
            body.map_or_else(String::new, |value| value.to_string()),
        ))
        .expect("request")
}

#[tokio::test]
async fn v1_custom_folders_copy_move_and_events_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-folders-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-folders-{suffix}"),
        "Folder candidate",
    )
    .await;

    let app = router(&context.connection_string()).await;
    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/folders",
            Some(json!({
                "name": "Clients",
                "description": "Client project mail",
                "account_id": account_id,
                "color": "#3b82f6",
                "sort_order": 10
            })),
        ))
        .await
        .expect("create first folder");
    assert_eq!(response.status(), StatusCode::OK);
    let first_folder = response_json(response).await;
    let first_folder_id = first_folder["folder_id"]
        .as_str()
        .expect("first folder id")
        .to_owned();
    assert!(first_folder_id.starts_with("mail_folder:"));
    assert_eq!(first_folder["name"], "Clients");
    assert_eq!(first_folder["message_count"], 0);

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/communications/folders",
            Some(json!({ "name": "Archive copy", "account_id": account_id, "sort_order": 20 })),
        ))
        .await
        .expect("create second folder");
    assert_eq!(response.status(), StatusCode::OK);
    let second_folder = response_json(response).await;
    let second_folder_id = second_folder["folder_id"]
        .as_str()
        .expect("second folder id")
        .to_owned();
    assert_eq!(second_folder["message_count"], 0);
    let response = app
        .clone()
        .oneshot(request(
            Method::PUT,
            &format!("/api/v1/communications/folders/{first_folder_id}"),
            Some(json!({
                "name": "Clients updated",
                "color": "#2563eb"
            })),
        ))
        .await
        .expect("update first folder");
    assert_eq!(response.status(), StatusCode::OK);
    let updated_folder = response_json(response).await;
    assert_eq!(updated_folder["name"], "Clients updated");
    assert_eq!(updated_folder["color"], "#2563eb");

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            &format!("/api/v1/communications/folders/{first_folder_id}/messages/{message_id}/copy"),
            None,
        ))
        .await
        .expect("copy message");
    assert_eq!(response.status(), StatusCode::OK);
    let copied = response_json(response).await;
    assert_eq!(copied["operation"], "copy");
    assert_eq!(copied["folder_id"], first_folder_id);
    assert_eq!(copied["message_id"], message_id);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders?account_id={account_id}"),
            None,
        ))
        .await
        .expect("list folders after copy");
    assert_eq!(response.status(), StatusCode::OK);
    let folders_after_copy = response_json(response).await;
    assert_folder_count(&folders_after_copy, &first_folder_id, 1);
    assert_folder_count(&folders_after_copy, &second_folder_id, 0);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders/{first_folder_id}/messages?limit=1"),
            None,
        ))
        .await
        .expect("list first folder messages");
    assert_eq!(response.status(), StatusCode::OK);
    let first_list = response_json(response).await;
    assert_eq!(first_list["items"].as_array().expect("items").len(), 1);
    assert_eq!(first_list["items"][0]["message_id"], message_id);
    assert_eq!(first_list["items"][0]["subject"], "Folder candidate");
    assert_eq!(first_list["has_more"], false);

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            &format!(
                "/api/v1/communications/folders/{second_folder_id}/messages/{message_id}/move"
            ),
            None,
        ))
        .await
        .expect("move message");
    assert_eq!(response.status(), StatusCode::OK);
    let moved = response_json(response).await;
    assert_eq!(moved["operation"], "move");
    assert_eq!(moved["folder_id"], second_folder_id);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders?account_id={account_id}"),
            None,
        ))
        .await
        .expect("list folders after move");
    assert_eq!(response.status(), StatusCode::OK);
    let folders_after_move = response_json(response).await;
    assert_folder_count(&folders_after_move, &first_folder_id, 0);
    assert_folder_count(&folders_after_move, &second_folder_id, 1);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders/{first_folder_id}/messages"),
            None,
        ))
        .await
        .expect("first folder after move");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await["items"]
            .as_array()
            .expect("items")
            .len(),
        0
    );

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            &format!("/api/v1/communications/folders/{second_folder_id}/messages"),
            None,
        ))
        .await
        .expect("second folder after move");
    assert_eq!(response.status(), StatusCode::OK);
    let second_list = response_json(response).await;
    assert_eq!(second_list["items"].as_array().expect("items").len(), 1);
    assert_eq!(second_list["items"][0]["message_id"], message_id);
    let response = app
        .clone()
        .oneshot(request(
            Method::DELETE,
            &format!("/api/v1/communications/folders/{second_folder_id}"),
            None,
        ))
        .await
        .expect("delete second folder");
    assert_eq!(response.status(), StatusCode::OK);
    let deleted = response_json(response).await;
    assert_eq!(deleted["deleted"], true);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE subject->>'kind' IN ('mail_folder', 'mail_folder_message')",
    )
    .fetch_one(&pool)
    .await
    .expect("event count");
    assert_eq!(event_count, 6);
    let folder_links = sqlx::query(
        "SELECT observation_id, entity_id, relationship_kind, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'mail_folder'
         ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("folder observation links");
    assert_eq!(folder_links.len(), 4);
    let folder_operations: Vec<String> = folder_links
        .iter()
        .map(|row| {
            row.try_get::<Value, _>("metadata")
                .expect("folder metadata")["operation"]
                .as_str()
                .expect("folder operation")
                .to_owned()
        })
        .collect();
    assert_eq!(
        folder_operations,
        vec![
            "folder_create".to_owned(),
            "folder_create".to_owned(),
            "folder_update".to_owned(),
            "folder_delete".to_owned()
        ]
    );
    let folder_observation_id: String = folder_links[0]
        .try_get("observation_id")
        .expect("folder observation id");
    let folder_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&folder_observation_id)
    .fetch_one(&pool)
    .await
    .expect("folder observation");
    let folder_origin_kind: String = folder_observation
        .try_get("origin_kind")
        .expect("folder origin kind");
    let folder_payload: Value = folder_observation
        .try_get("payload")
        .expect("folder payload");
    assert_eq!(folder_origin_kind, "manual");
    assert_eq!(folder_payload["operation"], "folder_create");

    let message_links = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'folder_message_transition'
         ORDER BY created_at ASC",
    )
    .bind(&message_id)
    .fetch_all(&pool)
    .await
    .expect("folder message observation links");
    assert_eq!(message_links.len(), 2);
    let message_operations: Vec<String> = message_links
        .iter()
        .map(|row| {
            row.try_get::<Value, _>("metadata")
                .expect("message metadata")["operation"]
                .as_str()
                .expect("message operation")
                .to_owned()
        })
        .collect();
    assert_eq!(
        message_operations,
        vec!["copy".to_owned(), "move".to_owned()]
    );
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Folder Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            account_id,
            "email_message",
            provider_record_id,
            format!("sha256:{provider_record_id}"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Body for folder API"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn assert_folder_count(response: &Value, folder_id: &str, expected_count: i64) {
    let folder = response["items"]
        .as_array()
        .expect("folder items")
        .iter()
        .find(|item| item["folder_id"] == folder_id)
        .expect("folder exists in response");
    assert_eq!(folder["message_count"], expected_count);
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
