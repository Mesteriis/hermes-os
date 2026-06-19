use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use hermes_hub_backend::ai::control_center::{
    AiControlCenterError, AiControlCenterStore, AiModelRouteUpdateRequest,
    AiProviderConsentRequest, AiProviderCreateRequest,
};
use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

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

#[tokio::test]
async fn remote_api_provider_models_require_host_vault_secret_before_private_context_use() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:api:openai-readiness".to_owned()),
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI Readiness".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: Some(true),
            api_key: Some("sk-not-persisted-by-store".to_owned()),
        })
        .await
        .expect("provider");

    let route_error = store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
            },
        )
        .await
        .expect_err("remote route requires host-vault secret binding");
    assert_invalid_request_contains(route_error, "host-vault API key");

    let prompt_error = store
        .test_prompt(
            "prompt:system:global:default_chat",
            &hermes_hub_backend::ai::control_center::AiPromptTestRequest {
                prompt_version_id: "prompt-version:system:global:default_chat:v1".to_owned(),
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
                variables: json!({"query": "hello"}),
                source_refs: Some(vec![]),
                score: None,
                notes: None,
            },
            "hermes-frontend",
        )
        .await
        .expect_err("prompt preview selection requires provider readiness");
    assert_invalid_request_contains(prompt_error, "host-vault API key");
}

#[tokio::test]
async fn remote_api_provider_model_route_accepts_host_vault_secret_binding() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = AiControlCenterStore::new(pool.clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:api:openai-ready".to_owned()),
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI Ready".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: Some(true),
            api_key: Some("sk-not-persisted-by-store".to_owned()),
        })
        .await
        .expect("provider");
    let secret_ref = format!("secret:ai-provider:{}:api_key", provider.provider_id);
    SecretReferenceStore::new(pool)
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "AI provider API key",
            )
            .metadata(json!({
                "provider_id": provider.provider_id,
                "secret_purpose": "api_key"
            })),
        )
        .await
        .expect("secret reference");
    store
        .bind_api_key_secret(&provider.provider_id, &secret_ref)
        .await
        .expect("secret binding");

    let route = store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
            },
        )
        .await
        .expect("ready remote route");

    assert_eq!(route.provider_id, provider.provider_id);
    assert_eq!(route.model_key, "gpt-5.5");
}

#[tokio::test]
async fn non_api_provider_rejects_api_key_secret_binding() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = AiControlCenterStore::new(pool.clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:built-in:ollama-no-secret".to_owned()),
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama-no-secret".to_owned(),
            display_name: "Ollama No Secret".to_owned(),
            base_url: None,
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: None,
            api_key: None,
        })
        .await
        .expect("provider");
    let secret_ref = format!("secret:ai-provider:{}:api_key", provider.provider_id);
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "AI provider API key",
            )
            .metadata(json!({
                "provider_id": provider.provider_id,
                "secret_purpose": "api_key"
            })),
        )
        .await
        .expect("secret reference");

    let error = store
        .bind_api_key_secret(&provider.provider_id, &secret_ref)
        .await
        .expect_err("non-API providers must not accept API key bindings");
    assert_invalid_request_contains(error, "only be bound to API providers");

    let binding_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM ai_provider_secret_refs WHERE provider_id = $1")
            .bind(&provider.provider_id)
            .fetch_one(&pool)
            .await
            .expect("binding count");
    assert_eq!(binding_count, 0);
}

#[tokio::test]
async fn non_api_provider_consent_mutation_is_rejected() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:built-in:ollama-consent".to_owned()),
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama-consent".to_owned(),
            display_name: "Ollama Consent".to_owned(),
            base_url: None,
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: None,
            api_key: None,
        })
        .await
        .expect("provider");

    let error = store
        .record_consent(
            &provider.provider_id,
            &AiProviderConsentRequest { consented: true },
        )
        .await
        .expect_err("non-API providers do not have remote-context consent");
    assert_invalid_request_contains(error, "only to API providers");

    let provider = store
        .provider(&provider.provider_id)
        .await
        .expect("provider lookup")
        .expect("provider remains");
    assert_eq!(provider.consent_state, "not_required");
}

#[tokio::test]
async fn api_provider_create_with_locked_host_vault_does_not_leave_provider_row() {
    let ctx = TestContext::new().await;
    let vault_home = tempfile::tempdir().expect("vault home");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let vault_home = vault_home.path().to_string_lossy().to_string();
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
        ("DATABASE_URL", database_url.as_str()),
        ("HERMES_VAULT_HOME", vault_home.as_str()),
    ])
    .expect("config");
    let app = build_router_with_database(config, database);
    let provider_id = "provider:api:locked-vault-create";

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/v1/ai/providers",
            json!({
                "provider_id": provider_id,
                "provider_kind": "api",
                "provider_key": "locked-vault-create",
                "display_name": "Locked Vault Create",
                "base_url": "https://api.example.invalid/v1",
                "remote_context_consent": true,
                "api_key": "sk-not-persisted"
            }),
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_eq!(body["error"], json!("host_vault_error"));

    let provider_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM ai_provider_accounts WHERE provider_id = $1)",
    )
    .bind(provider_id)
    .fetch_one(ctx.pool())
    .await
    .expect("provider exists query");
    assert!(!provider_exists);
}

#[tokio::test]
async fn ai_control_center_mutations_record_observation_trail_against_postgres() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = AiControlCenterStore::new(pool.clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:api:observation-trail".to_owned()),
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI Trail".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: Some(false),
            api_key: None,
        })
        .await
        .expect("provider");
    let secret_ref = format!("secret:ai-provider:{}:api_key", provider.provider_id);
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "AI provider API key",
            )
            .metadata(json!({
                "provider_id": provider.provider_id,
                "secret_purpose": "api_key"
            })),
        )
        .await
        .expect("secret reference");
    store
        .bind_api_key_secret(&provider.provider_id, &secret_ref)
        .await
        .expect("bind secret");
    store
        .update_provider(
            &provider.provider_id,
            &hermes_hub_backend::ai::control_center::AiProviderPatchRequest {
                display_name: Some("OpenAI Trail Updated".to_owned()),
                base_url: Some("https://api.openai.com/v1".to_owned()),
                config: Some(json!({"region": "us"})),
                enabled: Some(true),
                api_key: None,
            },
        )
        .await
        .expect("update provider");
    store
        .record_consent(
            &provider.provider_id,
            &AiProviderConsentRequest { consented: true },
        )
        .await
        .expect("record consent");
    store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
            },
        )
        .await
        .expect("put model route");
    store
        .provider_command(
            &provider.provider_id,
            hermes_hub_backend::ai::control_center::AiProviderCommandKind::SyncModels,
        )
        .await
        .expect("sync models");
    let prompt = store
        .create_prompt(
            &hermes_hub_backend::ai::control_center::AiPromptCreateRequest {
                prompt_id: Some("prompt:test:trail".to_owned()),
                name: "Trail prompt".to_owned(),
                entity_scope: "global".to_owned(),
                capability_slot: "default_chat".to_owned(),
                description: Some("Prompt observation trail".to_owned()),
                metadata: Some(json!({"team": "core"})),
            },
            "hermes-frontend",
        )
        .await
        .expect("create prompt");
    let version = store
        .create_prompt_version(
            &prompt.prompt_id,
            &hermes_hub_backend::ai::control_center::AiPromptVersionCreateRequest {
                prompt_version_id: Some("prompt-version:test:trail".to_owned()),
                version_label: Some("v1".to_owned()),
                body_template: "Answer {{query}}".to_owned(),
                variables: vec!["query".to_owned()],
            },
            "hermes-frontend",
        )
        .await
        .expect("create prompt version");
    store
        .activate_prompt_version(
            &prompt.prompt_id,
            &hermes_hub_backend::ai::control_center::AiPromptActivateRequest {
                prompt_version_id: version.prompt_version_id.clone(),
            },
            "hermes-frontend",
        )
        .await
        .expect("activate prompt version");
    let eval_run = store
        .test_prompt(
            &prompt.prompt_id,
            &hermes_hub_backend::ai::control_center::AiPromptTestRequest {
                prompt_version_id: version.prompt_version_id.clone(),
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
                variables: json!({"query": "hello"}),
                source_refs: Some(vec![json!({"kind": "observation", "id": "obs:test"})]),
                score: Some(5),
                notes: Some("looks good".to_owned()),
            },
            "hermes-frontend",
        )
        .await
        .expect("test prompt");

    let provider_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'provider_account'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&provider.provider_id)
    .fetch_all(&pool)
    .await
    .expect("provider observation rows");
    assert!(
        provider_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROVIDER_ACCOUNT"
                && row.get::<String, _>("relationship_kind") == "create"
        }),
        "provider create observation must exist"
    );
    assert!(
        provider_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROVIDER_ACCOUNT"
                && row.get::<String, _>("relationship_kind") == "update"
                && row.get::<Value, _>("payload")["display_name"] == "OpenAI Trail Updated"
        }),
        "provider update observation must exist"
    );
    assert!(
        provider_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROVIDER_ACCOUNT"
                && row.get::<String, _>("relationship_kind") == "consent_recorded"
                && row.get::<Value, _>("payload")["consent_state"] == "granted"
        }),
        "provider consent observation must exist"
    );
    let model_catalog_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'model_catalog_item'
          AND link.entity_id LIKE $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(format!("{}:%", provider.provider_id))
    .fetch_all(&pool)
    .await
    .expect("model catalog observations");
    assert!(
        model_catalog_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_MODEL_CATALOG_ITEM"
                && row.get::<String, _>("relationship_kind") == "seed"
        }),
        "model catalog seed observation must exist"
    );

    let prompt_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'prompt_template'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&prompt.prompt_id)
    .fetch_all(&pool)
    .await
    .expect("prompt observations");
    assert!(
        prompt_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE"
                && row.get::<String, _>("relationship_kind") == "create"
        }),
        "prompt create observation must exist"
    );
    assert!(
        prompt_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE"
                && row.get::<String, _>("relationship_kind") == "activate"
                && row.get::<Value, _>("payload")["active_version_id"]
                    == Value::String(version.prompt_version_id.clone())
        }),
        "prompt activate observation must exist"
    );

    let version_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'prompt_template_version'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&version.prompt_version_id)
    .fetch_all(&pool)
    .await
    .expect("prompt version observations");
    assert!(
        version_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE_VERSION"
                && row.get::<String, _>("relationship_kind") == "create"
        }),
        "prompt version create observation must exist"
    );
    assert!(
        version_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE_VERSION"
                && row.get::<String, _>("relationship_kind") == "activate"
                && row.get::<Value, _>("payload")["status"] == "active"
        }),
        "prompt version activate observation must exist"
    );

    let eval_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'prompt_eval_run'
          AND link.entity_id = $1
        "#,
    )
    .bind(&eval_run.eval_run_id)
    .fetch_all(&pool)
    .await
    .expect("prompt eval observations");
    assert!(
        eval_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_EVAL_RUN"
                && row.get::<String, _>("relationship_kind") == "test"
                && row.get::<Value, _>("payload")["provider_id"]
                    == Value::String(provider.provider_id.clone())
        }),
        "prompt eval observation must exist"
    );

    let binding_id = format!("{}:api_key", provider.provider_id);
    let binding_row = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'provider_secret_binding'
          AND link.entity_id = $1
        ORDER BY observation.captured_at DESC
        LIMIT 1
        "#,
    )
    .bind(&binding_id)
    .fetch_one(&pool)
    .await
    .expect("binding observation row");
    assert_eq!(
        binding_row.get::<String, _>("kind_code"),
        "AI_PROVIDER_SECRET_BINDING"
    );
    assert_eq!(binding_row.get::<String, _>("relationship_kind"), "bind");
    assert_eq!(
        binding_row.get::<Value, _>("payload")["secret_ref"],
        secret_ref
    );

    let route_row = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'model_route'
          AND link.entity_id = 'default_chat'
        ORDER BY observation.captured_at DESC
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("route observation row");
    assert_eq!(route_row.get::<String, _>("kind_code"), "AI_MODEL_ROUTE");
    assert_eq!(route_row.get::<String, _>("relationship_kind"), "put");
    assert_eq!(
        route_row.get::<Value, _>("payload")["provider_id"],
        provider.provider_id
    );
    assert_eq!(route_row.get::<Value, _>("payload")["model_key"], "gpt-5.5");
}

fn assert_invalid_request_contains(error: AiControlCenterError, expected: &str) {
    match error {
        AiControlCenterError::InvalidRequest(message) => assert!(
            message.contains(expected),
            "expected `{message}` to contain `{expected}`"
        ),
        other => panic!("expected invalid request, got {other:?}"),
    }
}
