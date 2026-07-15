use super::support::{live_task_candidate_context, seed_document, seed_message, unique_suffix};

#[tokio::test]
async fn task_candidate_refresh_creates_message_and_document_candidates_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("TaskCandidate{suffix}");

    let message_id = seed_message(
        &context,
        suffix,
        &format!("sender-{suffix}@example.com"),
        &[format!("recipient-{suffix}@example.com")],
        &format!("provider-task-candidate-msg-{suffix}"),
        &format!("{keyword} Update"),
        "Please action: schedule sync call",
    )
    .await;
    let document_id = seed_document(
        &context.pool,
        &format!("document_task_candidate_{suffix}"),
        &format!("{keyword} architecture"),
        "Follow up: draft document",
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let document_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM documents WHERE document_id = $1")
            .bind(&document_id)
            .fetch_one(&context.pool)
            .await
            .expect("document observation id");

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");
    assert!(refreshed >= 2);

    let message_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, source_kind, review_state
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&message_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("message candidate rows");
    assert_eq!(
        message_rows.len(),
        1,
        "should persist deterministic message candidate"
    );
    assert_eq!(message_rows[0].1, "observation");
    assert_eq!(message_rows[0].2, "suggested");
    let message_observation_id: Option<String> = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("message candidate observation id");
    assert!(
        message_observation_id.is_some(),
        "message candidates must carry canonical observation evidence"
    );

    let document_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, source_kind, review_state
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&document_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("document candidate rows");
    assert_eq!(
        document_rows.len(),
        1,
        "should persist deterministic document candidate"
    );
    assert_eq!(document_rows[0].1, "observation");
}

#[tokio::test]
async fn task_candidate_refresh_uses_obligation_engine_for_message_commitments_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the redlined agreement {suffix}");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("commitment-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-{suffix}"),
        &format!("Obligation engine {suffix}"),
        &format!("I will {statement} by Friday 5pm."),
    )
    .await;

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= 1);
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");

    let rows: Vec<(String, String, Option<String>, f64, String)> = sqlx::query_as(
        r#"
        SELECT title, review_state, due_text, confidence, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("message candidate rows");

    assert_eq!(
        rows.len(),
        1,
        "commitment language should create one reviewable task candidate"
    );
    assert_eq!(rows[0].0, statement);
    assert_eq!(rows[0].1, "suggested");
    assert_eq!(rows[0].2.as_deref(), Some("Friday 5pm"));
    assert!(rows[0].3 > 0.7);
    assert_eq!(rows[0].4, format!("I will {statement} by Friday 5pm."));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&context.pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&statement)
            .fetch_one(&context.pool)
            .await
            .expect("accepted obligation count");

    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn task_candidate_refresh_detects_russian_message_action_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("Надо проверить backup retention {suffix} до пятницы.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("russian-action-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-russian-{suffix}"),
        &format!("Russian action {suffix}"),
        &statement,
    )
    .await;

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= 1);
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");

    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        r#"
        SELECT title, review_state, due_text, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
          AND candidate_kind = 'task'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("message candidate rows");

    assert_eq!(
        rows.len(),
        1,
        "Russian action language should create one reviewable task candidate"
    );
    assert_eq!(rows[0].0, statement);
    assert_eq!(rows[0].1, "suggested");
    assert_eq!(rows[0].2.as_deref(), Some("пятницы"));
    assert_eq!(rows[0].3, statement);
}

#[tokio::test]
async fn task_candidate_refresh_detects_multilingual_message_actions_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let cases = [
        (
            "spanish",
            format!("Acción: preparar el resumen Zulip {suffix} para mañana."),
            Some("mañana"),
        ),
        (
            "french",
            format!("À faire: préparer le résumé Zulip {suffix} avant demain."),
            Some("demain"),
        ),
        (
            "german",
            format!("Aufgabe: Zulip-Zusammenfassung {suffix} vorbereiten bis morgen."),
            Some("morgen"),
        ),
    ];

    for (language, statement, _) in cases.iter() {
        seed_message(
            &context,
            suffix,
            &format!("{language}-action-{suffix}@example.com"),
            &[format!("owner-{suffix}@example.com")],
            &format!("provider-task-candidate-{language}-{suffix}"),
            &format!("Multilingual action {language} {suffix}"),
            statement,
        )
        .await;
    }

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= cases.len());

    let mut candidate_ids = Vec::new();
    for (language, statement, expected_due_text) in cases {
        let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
            r#"
            SELECT
                candidate.task_candidate_id,
                candidate.title,
                candidate.review_state,
                candidate.due_text,
                candidate.evidence_excerpt
            FROM task_candidates candidate
            JOIN communication_messages message
              ON message.observation_id = candidate.source_id
            WHERE message.provider_record_id = $1
              AND candidate.source_kind = 'observation'
              AND candidate.candidate_kind = 'task'
            "#,
        )
        .bind(format!("provider-task-candidate-{language}-{suffix}"))
        .fetch_all(&context.pool)
        .await
        .expect("multilingual candidate rows");

        assert_eq!(
            rows.len(),
            1,
            "{language} action language should create one reviewable task candidate"
        );
        candidate_ids.push(rows[0].0.clone());
        assert_eq!(rows[0].1, statement);
        assert_eq!(rows[0].2, "suggested");
        assert_eq!(rows[0].3.as_deref(), expected_due_text);
        assert_eq!(rows[0].4, statement);
    }

    let task_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM tasks WHERE task_candidate_id = ANY($1)",
    )
    .bind(candidate_ids)
    .fetch_one(&context.pool)
    .await
    .expect("auto-created task count");
    assert_eq!(task_count, 0);
}

#[tokio::test]
async fn task_candidate_refresh_detects_freeform_multilingual_message_requests_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let cases = [
        (
            "english",
            format!("Could you check the Zulip backup retention {suffix} by Friday?"),
            Some("friday"),
        ),
        (
            "russian",
            format!("Можешь проверить Zulip backup retention {suffix} до пятницы?"),
            Some("пятницы"),
        ),
        (
            "spanish",
            format!("¿Puedes preparar el resumen Zulip {suffix} para mañana?"),
            Some("mañana"),
        ),
        (
            "french",
            format!("Peux-tu préparer le résumé Zulip {suffix} avant demain?"),
            Some("demain"),
        ),
        (
            "german",
            format!("Kannst du die Zulip-Zusammenfassung {suffix} prüfen bis morgen?"),
            Some("morgen"),
        ),
    ];

    for (language, statement, _) in cases.iter() {
        seed_message(
            &context,
            suffix,
            &format!("{language}-freeform-{suffix}@example.com"),
            &[format!("owner-{suffix}@example.com")],
            &format!("provider-task-candidate-freeform-{language}-{suffix}"),
            &format!("Freeform request {language} {suffix}"),
            statement,
        )
        .await;
    }

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= cases.len());

    let mut candidate_ids = Vec::new();
    for (language, statement, expected_due_text) in cases {
        let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
            r#"
            SELECT
                candidate.task_candidate_id,
                candidate.title,
                candidate.review_state,
                candidate.due_text,
                candidate.evidence_excerpt
            FROM task_candidates candidate
            JOIN communication_messages message
              ON message.observation_id = candidate.source_id
            WHERE message.provider_record_id = $1
              AND candidate.source_kind = 'observation'
              AND candidate.candidate_kind = 'task'
            "#,
        )
        .bind(format!(
            "provider-task-candidate-freeform-{language}-{suffix}"
        ))
        .fetch_all(&context.pool)
        .await
        .expect("freeform multilingual candidate rows");

        assert_eq!(
            rows.len(),
            1,
            "{language} free-form request should create one reviewable task candidate"
        );
        candidate_ids.push(rows[0].0.clone());
        assert_eq!(rows[0].1, statement);
        assert_eq!(rows[0].2, "suggested");
        assert_eq!(rows[0].3.as_deref(), expected_due_text);
        assert_eq!(rows[0].4, statement);
    }

    let task_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM tasks WHERE task_candidate_id = ANY($1)",
    )
    .bind(candidate_ids)
    .fetch_one(&context.pool)
    .await
    .expect("auto-created task count");
    assert_eq!(task_count, 0);
}

#[tokio::test]
async fn task_candidate_refresh_uses_obligation_engine_for_document_commitments_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the document-backed commitment {suffix}");
    let document_id = seed_document(
        &context.pool,
        &format!("document_obligation_candidate_{suffix}"),
        &format!("Document obligation {suffix}"),
        &format!("I will {statement} by Friday 5pm."),
    )
    .await;

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= 1);
    let document_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM documents WHERE document_id = $1")
            .bind(&document_id)
            .fetch_one(&context.pool)
            .await
            .expect("document observation id");

    let rows: Vec<(String, String, String, Option<String>, f64, String)> = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text, confidence, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&document_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("document obligation candidate rows");

    assert_eq!(
        rows.len(),
        1,
        "document commitment language should create one reviewable obligation-derived task candidate"
    );
    assert_eq!(rows[0].0, statement);
    assert_eq!(rows[0].1, "suggested");
    assert_eq!(rows[0].2, "obligation_task");
    assert_eq!(rows[0].3.as_deref(), Some("Friday 5pm"));
    assert!(rows[0].4 > 0.7);
    assert_eq!(rows[0].5, format!("I will {statement} by Friday 5pm."));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&document_id)
            .fetch_one(&context.pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&statement)
            .fetch_one(&context.pool)
            .await
            .expect("accepted obligation count");

    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn task_candidate_refresh_updates_existing_source_title_candidate_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("source-title-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-source-title-{suffix}"),
        &format!("Source title conflict {suffix}"),
        "Action: Review This Item",
    )
    .await;
    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");

    sqlx::query(
        r#"
        INSERT INTO task_candidates (
            task_candidate_id,
            source_kind,
            source_id,
            observation_id,
            candidate_kind,
            candidate_metadata,
            title,
            confidence,
            review_state,
            evidence_excerpt
        )
        VALUES ($1, 'observation', $2, $3, 'task', '{}'::jsonb, $4, 0.5, 'suggested', $5)
        "#,
    )
    .bind(format!("task_candidate:v1:legacy-source-title:{suffix}"))
    .bind(&observation_id)
    .bind(&observation_id)
    .bind("action: review this item")
    .bind("legacy evidence")
    .execute(&context.pool)
    .await
    .expect("legacy candidate");

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh should update source/title candidate without duplicate-key failure");
    assert!(refreshed >= 1);

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, title, evidence_excerpt
        FROM task_candidates
        WHERE source_kind = 'observation' AND source_id = $1
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("candidate rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, "Action: Review This Item");
    assert_eq!(rows[0].2, "Action: Review This Item");
}
