use serde_json::Value;
use sqlx::postgres::PgPool;

pub async fn ensure_runtime_processing_state(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    metadata: &Value,
) -> Result<String, sqlx::Error> {
    let existing = sqlx::query_scalar::<_, String>(
        r#"
        SELECT state
        FROM signal_runtime_states
        WHERE source_code = $1
          AND connection_id IS NULL
          AND runtime_kind = $2
        "#,
    )
    .bind(source_code)
    .bind(runtime_kind)
    .fetch_optional(pool)
    .await?;

    if let Some(state) = existing {
        return Ok(state);
    }

    let default_state = source_runtime_state_from_policies(pool, source_code).await?;

    sqlx::query(
        r#"
        INSERT INTO signal_runtime_states (
            id,
            source_code,
            runtime_kind,
            state,
            last_started_at,
            metadata
        )
        VALUES (
            gen_random_uuid(),
            $1,
            $2,
            $3,
            CASE WHEN $3 = 'running' THEN now() ELSE NULL END,
            $4
        )
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(source_code)
    .bind(runtime_kind)
    .bind(default_state)
    .bind(metadata)
    .execute(pool)
    .await?;

    sqlx::query_scalar::<_, String>(
        r#"
        SELECT state
        FROM signal_runtime_states
        WHERE source_code = $1
          AND connection_id IS NULL
          AND runtime_kind = $2
        "#,
    )
    .bind(source_code)
    .bind(runtime_kind)
    .fetch_one(pool)
    .await
}

pub async fn source_runtime_state_from_policies(
    pool: &PgPool,
    source_code: &str,
) -> Result<&'static str, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT mode
        FROM signal_policies
        WHERE (expires_at IS NULL OR expires_at > now())
          AND connection_id IS NULL
          AND event_pattern IS NULL
          AND (
                (scope = 'global' AND source_code IS NULL)
             OR (scope = 'source' AND source_code = $1)
          )
        "#,
    )
    .bind(source_code)
    .fetch_all(pool)
    .await?;

    if rows.iter().any(|mode| mode == "disabled") {
        return Ok("stopped");
    }
    if rows.iter().any(|mode| mode == "paused") {
        return Ok("paused");
    }
    if rows.iter().any(|mode| mode == "muted") {
        return Ok("muted");
    }

    Ok("running")
}

pub fn runtime_state_allows_processing(state: &str) -> bool {
    matches!(state, "running" | "starting" | "reconnecting")
}

pub async fn runtime_allows_processing(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    metadata: &Value,
) -> Result<bool, sqlx::Error> {
    let state = ensure_runtime_processing_state(pool, source_code, runtime_kind, metadata).await?;
    Ok(runtime_state_allows_processing(&state))
}
