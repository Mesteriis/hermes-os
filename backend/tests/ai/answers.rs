use crate::support::*;
use std::env;

#[tokio::test]
async fn ai_answer_api_returns_source_backed_answer_and_persists_run() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live AI answer API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let person_store = PersonProjectionStore::new(pool.clone());
    let owner = person_store
        .upsert_email_person(&format!("ai-owner-{suffix}@example.com"))
        .await
        .expect("owner persona candidate");
    let owner = person_store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");
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
    assert_eq!(
        body["agent_persona_id"],
        json!("persona:v1:ai_agent:MNEMOSYNE")
    );
    assert_eq!(body["owner_persona_id"], json!(owner.person_id));
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

    let run_attribution = sqlx::query(
        r#"
        SELECT agent_persona_id, owner_persona_id
        FROM ai_agent_runs
        WHERE run_id = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("run attribution");
    assert_eq!(
        run_attribution
            .try_get::<Option<String>, _>("agent_persona_id")
            .unwrap()
            .as_deref(),
        Some("persona:v1:ai_agent:MNEMOSYNE")
    );
    assert_eq!(
        run_attribution
            .try_get::<Option<String>, _>("owner_persona_id")
            .unwrap()
            .as_deref(),
        Some(owner.person_id.as_str())
    );
}

#[tokio::test]
async fn ai_task_refresh_creates_suggested_candidates_without_active_tasks() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live AI task refresh API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
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
