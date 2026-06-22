use crate::support::*;
use testkit::context::TestContext;

#[tokio::test]
async fn ai_answer_api_returns_source_backed_answer_and_persists_run() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
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
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
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

    let run_observations: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'agent_run'
          AND link.entity_id = $1
          AND kind.code IN ('AI_AGENT_RUN', 'AI_AGENT_RUN_STATUS')
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("run observations");
    assert!(
        run_observations >= 2,
        "expected run requested + completed observations"
    );

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
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
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
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([("HERMES_OLLAMA_BASE_URL", ollama_base_url.as_str())])
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

    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let candidate = sqlx::query(
        r#"
        SELECT task_candidate_id, review_state, agent_run_id, observation_id
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("candidate");
    assert_eq!(candidate.get::<String, _>("review_state"), "suggested");
    assert!(candidate.get::<Option<String>, _>("agent_run_id").is_some());
    assert_eq!(
        candidate
            .get::<Option<String>, _>("observation_id")
            .as_deref(),
        Some(message_observation_id.as_str())
    );

    let active_task_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE source_id = $1")
            .bind(&message_observation_id)
            .fetch_one(&pool)
            .await
            .expect("active task count");
    assert_eq!(active_task_count, 0);

    let review_item_row = sqlx::query(
        r#"
        SELECT item_kind, status, metadata
        FROM review_items
        WHERE review_item_id IN (
            SELECT review_item_id
            FROM review_item_evidence
            WHERE observation_id = $1
        )
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("review item row");
    assert_eq!(
        review_item_row.get::<String, _>("item_kind"),
        "potential_task"
    );
    assert_eq!(review_item_row.get::<String, _>("status"), "new");
    let metadata = review_item_row.get::<serde_json::Value, _>("metadata");
    assert_eq!(metadata["mirrored_from"], json!("task_candidates"));
}
