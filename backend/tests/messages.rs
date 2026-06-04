use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use hermes_hub_backend::messages::{
    MessageProjectionError, MessageProjectionStore, NewProjectedMessage, project_raw_email_message,
};
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn message_projection_upserts_canonical_message_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_projection_{suffix}");
    let raw_record_id = format!("raw_message_projection_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection Gmail",
        format!("projection-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &format!("provider-message-{suffix}"),
        "Projected subject",
        "Projected body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(
        projected.provider_record_id,
        format!("provider-message-{suffix}")
    );
    assert_eq!(projected.subject, "Projected subject");
    assert_eq!(projected.sender, "alice@example.com");
    assert_eq!(projected.recipients, vec!["bob@example.com".to_owned()]);
}

#[tokio::test]
async fn message_projection_distinguishes_delimiter_bearing_identities_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("delimiter-bearing message projection identities").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let base_account_id = format!("acct_message_identity_{suffix}");
    let left_account_id = format!("{base_account_id}:left");

    store_provider_account(
        &communication_store,
        &base_account_id,
        "Projection identity base",
        format!("projection-identity-base-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &left_account_id,
        "Projection identity left",
        format!("projection-identity-left-{suffix}@example.com"),
    )
    .await;

    let base_raw = record_raw_email_message(
        &communication_store,
        &base_account_id,
        &format!("raw_message_identity_base_{suffix}"),
        "left:right",
        "Delimiter subject base",
        "Delimiter body base",
    )
    .await;
    let left_raw = record_raw_email_message(
        &communication_store,
        &left_account_id,
        &format!("raw_message_identity_left_{suffix}"),
        "right",
        "Delimiter subject left",
        "Delimiter body left",
    )
    .await;

    let base_projected = project_raw_email_message(&message_store, &base_raw)
        .await
        .expect("project base delimiter message");
    let left_projected = project_raw_email_message(&message_store, &left_raw)
        .await
        .expect("project left delimiter message");

    assert_ne!(base_projected.message_id, left_projected.message_id);
    assert_eq!(base_projected.account_id, base_account_id);
    assert_eq!(base_projected.provider_record_id, "left:right");
    assert_eq!(left_projected.account_id, left_account_id);
    assert_eq!(left_projected.provider_record_id, "right");

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE account_id IN ($1, $2)
        "#,
    )
    .bind(&base_projected.account_id)
    .bind(&left_projected.account_id)
    .fetch_one(&pool)
    .await
    .expect("projected delimiter message count");
    assert_eq!(count, 2);
}

#[tokio::test]
async fn message_projection_reprojects_same_raw_record_idempotently_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("idempotent message projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_idempotent_{suffix}");
    let provider_record_id = format!("provider-message-idempotent-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection idempotent Gmail",
        format!("projection-idempotent-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_idempotent_{suffix}"),
        &provider_record_id,
        "Idempotent subject",
        "Idempotent body",
    )
    .await;

    let first = project_raw_email_message(&message_store, &raw)
        .await
        .expect("first message projection");
    let second = project_raw_email_message(&message_store, &raw)
        .await
        .expect("second message projection");

    assert_eq!(second.message_id, first.message_id);
    assert_eq!(second.raw_record_id, first.raw_record_id);
    assert_eq!(second.account_id, first.account_id);
    assert_eq!(second.provider_record_id, first.provider_record_id);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("idempotent projected message count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn message_projection_reports_missing_or_wrong_payload_fields() {
    let store = disconnected_message_store();
    let cases = [
        (
            "subject",
            json!({"from":"alice@example.com","to":["bob@example.com"],"body_text":"Body"}),
        ),
        (
            "from",
            json!({"subject":"Subject","from":42,"to":["bob@example.com"],"body_text":"Body"}),
        ),
        (
            "to",
            json!({"subject":"Subject","from":"alice@example.com","to":"bob@example.com","body_text":"Body"}),
        ),
        (
            "to",
            json!({"subject":"Subject","from":"alice@example.com","to":["bob@example.com",42],"body_text":"Body"}),
        ),
        (
            "body_text",
            json!({"subject":"Subject","from":"alice@example.com","to":["bob@example.com"]}),
        ),
    ];

    for (field_name, payload) in cases {
        let raw = stored_raw_record_with_payload(payload);
        let error = project_raw_email_message(&store, &raw)
            .await
            .expect_err("projecting malformed payload must fail");

        assert!(
            matches!(
                error,
                MessageProjectionError::MissingPayloadField(actual) if actual == field_name
            ),
            "expected MissingPayloadField({field_name}), got {error:?}"
        );
    }
}

#[tokio::test]
async fn message_projection_derives_message_id_for_direct_upsert_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("direct message upsert identity derivation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_direct_identity_{suffix}");
    let provider_record_id = format!("provider-message-direct-identity-{suffix}");
    let arbitrary_message_id = format!("not-canonical-message-id-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection direct identity",
        format!("projection-direct-identity-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_direct_identity_{suffix}"),
        &provider_record_id,
        "Direct identity subject",
        "Direct identity body",
    )
    .await;
    let message = NewProjectedMessage {
        message_id: arbitrary_message_id.clone(),
        raw_record_id: raw.raw_record_id,
        account_id,
        provider_record_id,
        subject: "Direct identity subject".to_owned(),
        sender: "alice@example.com".to_owned(),
        recipients: vec!["bob@example.com".to_owned()],
        body_text: "Direct identity body".to_owned(),
        occurred_at: raw.occurred_at,
    };

    let projected = message_store
        .upsert_message(&message)
        .await
        .expect("direct upsert derives canonical message ID");

    assert_ne!(projected.message_id, arbitrary_message_id);
    assert!(projected.message_id.starts_with("msg:v1:"));

    let arbitrary_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_messages WHERE message_id = $1",
    )
    .bind(&arbitrary_message_id)
    .fetch_one(&pool)
    .await
    .expect("arbitrary message ID count");
    assert_eq!(arbitrary_count, 0);
}

#[tokio::test]
async fn message_projection_rejects_direct_upsert_with_mismatched_raw_tuple_against_postgres() {
    let Some((pool, communication_store, message_store)) =
        live_projection_context("direct message upsert raw tuple mismatch").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let raw_account_id = format!("acct_message_raw_tuple_{suffix}");
    let mismatched_account_id = format!("acct_message_raw_tuple_other_{suffix}");
    let provider_record_id = format!("provider-message-raw-tuple-{suffix}");

    store_provider_account(
        &communication_store,
        &raw_account_id,
        "Projection raw tuple source",
        format!("projection-raw-tuple-source-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &mismatched_account_id,
        "Projection raw tuple mismatch",
        format!("projection-raw-tuple-mismatch-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &raw_account_id,
        &format!("raw_message_tuple_{suffix}"),
        &provider_record_id,
        "Raw tuple subject",
        "Raw tuple body",
    )
    .await;
    let message = NewProjectedMessage {
        message_id: format!("msg:mismatched:{suffix}"),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: mismatched_account_id.clone(),
        provider_record_id: provider_record_id.clone(),
        subject: "Raw tuple subject".to_owned(),
        sender: "alice@example.com".to_owned(),
        recipients: vec!["bob@example.com".to_owned()],
        body_text: "Raw tuple body".to_owned(),
        occurred_at: raw.occurred_at,
    };

    let error = message_store
        .upsert_message(&message)
        .await
        .expect_err("direct upsert must reject mismatched raw tuple");

    assert!(
        matches!(
            error,
            MessageProjectionError::RawRecordTupleMismatch {
                ref raw_record_id,
                ref account_id,
                provider_record_id: ref actual_provider_record_id,
            } if raw_record_id.as_str() == raw.raw_record_id
                && account_id.as_str() == mismatched_account_id
                && actual_provider_record_id.as_str() == provider_record_id
        ),
        "expected RawRecordTupleMismatch, got {error:?}"
    );

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE raw_record_id = $1
          AND account_id = $2
          AND provider_record_id = $3
        "#,
    )
    .bind(&raw.raw_record_id)
    .bind(&mismatched_account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("mismatched projected message count");
    assert_eq!(count, 0);
}

async fn live_projection_context(
    test_name: &str,
) -> Option<(PgPool, CommunicationIngestionStore, MessageProjectionStore)> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((
        pool.clone(),
        CommunicationIngestionStore::new(pool.clone()),
        MessageProjectionStore::new(pool),
    ))
}

async fn store_provider_account(
    store: &CommunicationIngestionStore,
    account_id: &str,
    display_name: &str,
    external_account_id: String,
) {
    store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            display_name,
            external_account_id,
        ))
        .await
        .expect("store provider account");
}

async fn record_raw_email_message(
    store: &CommunicationIngestionStore,
    account_id: &str,
    raw_record_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> StoredRawCommunicationRecord {
    store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                raw_record_id,
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "subject": subject,
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record raw message")
}

fn disconnected_message_store() -> MessageProjectionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool");
    MessageProjectionStore::new(pool)
}

fn stored_raw_record_with_payload(payload: Value) -> StoredRawCommunicationRecord {
    let suffix = unique_suffix();

    StoredRawCommunicationRecord {
        raw_record_id: format!("raw_payload_validation_{suffix}"),
        account_id: format!("acct_payload_validation_{suffix}"),
        record_kind: "email_message".to_owned(),
        provider_record_id: format!("provider-payload-validation-{suffix}"),
        source_fingerprint: format!("sha256:payload-validation-{suffix}"),
        import_batch_id: format!("batch_payload_validation_{suffix}"),
        occurred_at: Some(Utc::now()),
        captured_at: Utc::now(),
        payload,
        provenance: json!({"source":"test"}),
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
