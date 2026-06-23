use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::{get as axum_get, post as axum_post};
use axum::{Json, Router};
use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::signal_hub::{
    SignalHubStore, SignalPolicy, SignalPolicyMode, SignalPolicyScope,
};
use hermes_hub_backend::platform::settings::ApplicationSettingsStore;
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;
use testkit::context::TestContext;
use tokio::net::TcpListener;
use tower::ServiceExt;

use super::support::{
    T, get, post, response_json, router, seed_projected_message, seed_projected_message_with_body,
    uid,
};

#[tokio::test]
async fn v1_messages_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-cursor-api-{suffix}");
    let mut seeded_message_ids = Vec::new();

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-cursor-api-{suffix}-{index}"),
            &format!("Cursor page subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic message ordering");
        seeded_message_ids.push(message_id);
    }

    let router = router(&context.connection_string()).await;
    let first = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = router
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"].as_array().expect("second items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
    for message_id in seeded_message_ids {
        assert!(returned_ids.contains(&message_id), "missing {message_id}");
    }
}

#[tokio::test]
async fn v1_threads_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-cursor-api-{suffix}");

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-thread-cursor-api-{suffix}-{index}"),
            &format!("Thread Cursor Subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic thread ordering");
    }

    let router = router(&context.connection_string()).await;
    let first = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first thread cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first thread items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next thread cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = router
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second thread cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"]
        .as_array()
        .expect("second thread items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["thread_id"].as_str().expect("thread id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
}

#[tokio::test]
async fn v1_translate_thread_returns_per_message_fallbacks_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-translate-{suffix}");
    let subject = "Thread Translation";
    let first_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-1-{suffix}"),
        subject,
        "Привет, нужна проверка договора.",
    )
    .await;
    let second_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-2-{suffix}"),
        &format!("Re: {subject}"),
        "Hello, the agreement is attached.",
    )
    .await;
    let router = router(&context.connection_string()).await;
    let response = router
        .oneshot(post(
            &format!(
                "/api/v1/communications/threads/translate?account_id={account_id}&subject=Thread%20Translation"
            ),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("thread translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["subject"], subject);
    assert_eq!(body["target_language"], "en");
    let items = body["items"].as_array().expect("translation items");
    assert_eq!(items.len(), 2);
    let returned_ids = items
        .iter()
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert!(returned_ids.contains(&first_id));
    assert!(returned_ids.contains(&second_id));
    assert!(
        items
            .iter()
            .any(|item| item["original_language"] == "ru" && item["translated"] == false)
    );
    assert!(items.iter().all(|item| {
        item["reason"]
            .as_str()
            .map(|reason| !reason.trim().is_empty())
            .unwrap_or(false)
    }));
}

#[tokio::test]
async fn v1_translate_thread_emits_signal_hub_ai_events_per_message() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-translate-signals-{suffix}");
    let subject = "Thread Translation";
    let first_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-signal-1-{suffix}"),
        subject,
        "Привет, нужна проверка договора.",
    )
    .await;
    let second_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-signal-2-{suffix}"),
        &format!("Re: {subject}"),
        "Hola equipo, revisemos el acuerdo.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!(
                "/api/v1/communications/threads/translate?account_id={account_id}&subject=Thread%20Translation"
            ),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("thread translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    let items = body["items"].as_array().expect("translation items");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| item["translated"] == true));

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.thread_message_translation.observed',
            'signal.accepted.ai.thread_message_translation'
        )
          AND subject->>'message_id' = ANY($1)
        "#,
    )
    .bind(vec![first_id.clone(), second_id.clone()])
    .fetch_one(&pool)
    .await
    .expect("thread translation signal count");
    assert_eq!(signal_count, 4);
}

#[tokio::test]
async fn v1_message_translate_returns_fallback_when_ai_source_is_muted() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-message-translate-muted-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("message-translate-muted-{suffix}"),
        "Translate me",
        "Hola equipo, necesitamos revisar el contrato hoy.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    SignalHubStore::new(pool)
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("ai".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "mute ai message translation".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create ai mute policy");

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/translate"),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["translated"], false);
    assert_eq!(body["reason"], "no LLM configured");
}

#[tokio::test]
async fn v1_message_translate_emits_signal_hub_ai_events_when_runtime_runs() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-message-translate-signals-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("message-translate-signals-{suffix}"),
        "Translate me",
        "Hola equipo, necesitamos revisar el contrato hoy.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/translate"),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["translated"], true);
    assert_eq!(body["target"], "en");

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.message_translation.observed',
            'signal.accepted.ai.message_translation'
        )
          AND subject->>'message_id' = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("translation signal hub event count");
    assert_eq!(signal_count, 2);
}

#[tokio::test]
async fn v1_ai_reply_emits_signal_hub_ai_events_when_runtime_runs() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-ai-reply-signals-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("ai-reply-signals-{suffix}"),
        "Need reply",
        "Hello team, can you confirm the review schedule?",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/ai-reply"),
            json!({
                "tone": "friendly",
                "language": "en"
            }),
        ))
        .await
        .expect("ai reply response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["tone"], "friendly");
    assert_eq!(body["language"], "en");

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.reply_drafting.observed',
            'signal.accepted.ai.reply_drafting'
        )
          AND subject->>'message_id' = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai reply signal hub event count");
    assert_eq!(signal_count, 2);
}

#[tokio::test]
async fn v1_ai_reply_variants_emit_signal_hub_ai_events_when_runtime_runs() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-ai-reply-variants-signals-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("ai-reply-variants-signals-{suffix}"),
        "Need reply variants",
        "Hello team, can you confirm the review schedule?",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/ai-reply-variants"),
            json!({
                "languages": ["en", "es"],
                "tones": ["professional", "friendly"]
            }),
        ))
        .await
        .expect("ai reply variants response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["variants"].as_array().map(Vec::len), Some(4));

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.reply_variant_generation.observed',
            'signal.accepted.ai.reply_variant_generation'
        )
          AND subject->>'message_id' = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai reply variants signal hub event count");
    assert_eq!(signal_count, 2);
}

#[tokio::test]
async fn v1_extract_tasks_skips_llm_candidates_when_ai_source_is_muted() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-extract-muted-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("extract-muted-{suffix}"),
        "Roadmap note",
        "General roadmap discussion without explicit task markers.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    SignalHubStore::new(pool)
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("ai".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "mute ai task extraction".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create ai mute policy");

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/extract-tasks"),
            json!({}),
        ))
        .await
        .expect("extract tasks response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["tasks"], json!([]));
}

#[tokio::test]
async fn v1_extract_tasks_emits_signal_hub_ai_events_when_llm_runs() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-extract-signals-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("extract-signals-{suffix}"),
        "Roadmap note",
        "General roadmap discussion without explicit task markers.",
    )
    .await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;

    let router = router_with_ollama(&context.connection_string(), &ollama_base_url).await;
    let response = router
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/extract-tasks"),
            json!({}),
        ))
        .await
        .expect("extract tasks response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["tasks"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["tasks"][0]["source"], "llm");

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.message_task_extraction.observed',
            'signal.accepted.ai.message_task_extraction'
        )
          AND subject->>'message_id' = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("task extraction signal hub event count");
    assert_eq!(signal_count, 2);
}

async fn router_with_ollama(database_url: &str, ollama_base_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url)
            .with_test_pairs([
                ("HERMES_OLLAMA_BASE_URL", ollama_base_url),
                ("HERMES_OLLAMA_CHAT_MODEL", "qwen3:4b"),
                ("HERMES_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
            ])
            .expect("config"),
        database,
    )
}

async fn spawn_fake_ollama() -> String {
    let app = Router::new()
        .route(
            "/api/version",
            axum_get(|| async { Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            axum_get(|| async {
                Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/chat",
            axum_post(|Json(body): Json<serde_json::Value>| async move {
                let text = body["messages"]
                    .as_array()
                    .and_then(|messages| messages.last())
                    .and_then(|message| message["content"].as_str())
                    .unwrap_or_default();
                let content = if text.contains("Extract tasks from this email") {
                    r#"[{"title":"Review the roadmap draft","due_date":null,"assignee":null,"priority":"medium","source":"llm"}]"#
                } else {
                    "Translated content from fake Ollama."
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

async fn configure_fake_ollama_setting(pool: &sqlx::PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "hermes-frontend",
        )
        .await
        .expect("fake Ollama setting");
}
