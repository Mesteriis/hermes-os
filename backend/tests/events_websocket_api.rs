use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "events-websocket-test-token";

#[tokio::test]
async fn event_websocket_accepts_protected_upgrade_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_ws_{suffix}");

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("test server");
    });

    let mut stream = TcpStream::connect(address).await.expect("websocket socket");
    stream
        .write_all(
            format!(
                "GET /api/events/ws?after_position=0&batch_size=10&heartbeat_seconds=1&hermes_secret={LOCAL_API_TOKEN} HTTP/1.1\r\n\
                 Host: {address}\r\n\
                 Connection: Upgrade\r\n\
                 Upgrade: websocket\r\n\
                 Sec-WebSocket-Version: 13\r\n\
                 Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                 \r\n"
            )
            .as_bytes(),
        )
        .await
        .expect("websocket upgrade request");

    let mut buffer = [0_u8; 2048];
    let bytes_read = timeout(Duration::from_secs(2), stream.read(&mut buffer))
        .await
        .expect("websocket upgrade response timed out")
        .expect("websocket upgrade response");
    let response_text = std::str::from_utf8(&buffer[..bytes_read]).expect("utf-8 response");
    let normalized_response = response_text.to_ascii_lowercase();

    assert!(
        response_text.starts_with("HTTP/1.1 101 Switching Protocols"),
        "{response_text}"
    );
    assert!(
        normalized_response.contains("upgrade: websocket"),
        "{response_text}"
    );

    server.abort();
}

async fn app_with_database(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(config_with_api_token(), database)
}

fn config_with_api_token() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)])
        .expect("valid local API secret")
}

fn json_request_with_token(uri: &str, value: serde_json::Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}
