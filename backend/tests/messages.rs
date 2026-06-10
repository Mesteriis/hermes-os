use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};

use hermes_hub_backend::domains::mail::analytics::EmailAnalyticsStore;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionError, MessageProjectionStore, NewProjectedMessage, WorkflowState,
    project_raw_email_message, project_raw_email_message_from_blob,
};
use hermes_hub_backend::domains::mail::storage::LocalMailBlobStore;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

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
async fn message_projection_extracts_canonical_fields_from_raw_blob_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message raw blob projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_blob_projection_{suffix}");
    let raw_record_id = format!("raw_message_blob_projection_{suffix}");
    let provider_record_id = format!("provider-message-blob-{suffix}");
    let blob_root = tempfile::tempdir().expect("blob root");
    let blob_store = LocalMailBlobStore::new(blob_root.path());
    let local_blob = blob_store
        .put_blob(
            b"Subject: Real MIME\r\nFrom: Alice <alice@example.com>\r\nTo: Bob <bob@example.com>\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nHello=20from=20real=20mail.",
        )
        .await
        .expect("write raw mail blob");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection raw blob",
        format!("projection-raw-blob-{suffix}@example.com"),
    )
    .await;
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "provider": "imap",
                    "raw_blob_storage_kind": local_blob.storage_kind,
                    "raw_blob_storage_path": local_blob.storage_path,
                    "raw_blob_sha256": local_blob.sha256,
                    "raw_blob_size_bytes": local_blob.size_bytes
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"email_provider_sync"})),
        )
        .await
        .expect("record raw blob message");

    let projected = project_raw_email_message_from_blob(&message_store, &blob_store, &raw)
        .await
        .expect("project message from raw blob");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(projected.provider_record_id, provider_record_id);
    assert_eq!(projected.subject, "Real MIME");
    assert_eq!(projected.sender, "Alice <alice@example.com>");
    assert_eq!(
        projected.recipients,
        vec!["Bob <bob@example.com>".to_owned()]
    );
    assert_eq!(projected.body_text, "Hello from real mail.");
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
async fn message_projection_list_messages_filters_by_account_state_channel_and_query_against_postgres()
 {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message filtered listing").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_left = format!("acct_message_filter_left_{suffix}");
    let account_right = format!("acct_message_filter_right_{suffix}");

    store_provider_account(
        &communication_store,
        &account_left,
        "Filter Left",
        format!("filter-left-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &account_right,
        "Filter Right",
        format!("filter-right-{suffix}@example.com"),
    )
    .await;

    let left_raw = record_raw_email_message(
        &communication_store,
        &account_left,
        &format!("raw_message_filter_left_{suffix}"),
        &format!("provider-filter-left-{suffix}"),
        "Quarterly Alpha Contract",
        "The alpha renewal needs a legal review.",
    )
    .await;
    let right_raw = record_raw_email_message(
        &communication_store,
        &account_right,
        &format!("raw_message_filter_right_{suffix}"),
        &format!("provider-filter-right-{suffix}"),
        "Quarterly Beta Invoice",
        "The beta invoice is already paid.",
    )
    .await;

    let left = project_raw_email_message(&message_store, &left_raw)
        .await
        .expect("project left message");
    let right = project_raw_email_message(&message_store, &right_raw)
        .await
        .expect("project right message");
    message_store
        .transition_workflow_state(&left.message_id, WorkflowState::NeedsAction)
        .await
        .expect("set left state");
    message_store
        .transition_workflow_state(&right.message_id, WorkflowState::Reviewed)
        .await
        .expect("set right state");

    let filtered = message_store
        .list_messages(
            Some(&account_left),
            Some(WorkflowState::NeedsAction),
            Some("email"),
            Some("alpha legal"),
            10,
        )
        .await
        .expect("list filtered messages");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message.message_id, left.message_id);
    assert_eq!(filtered[0].message.account_id, account_left);

    let no_match = message_store
        .list_messages(
            Some(&account_left),
            Some(WorkflowState::NeedsAction),
            Some("email"),
            Some("beta"),
            10,
        )
        .await
        .expect("list non-matching messages");
    assert!(no_match.is_empty());
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
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some("alice@example.com".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
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
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some("alice@example.com".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
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

// --- Workflow state tests ---

#[test]
fn workflow_state_from_str_all_valid() {
    use hermes_hub_backend::domains::mail::messages::WorkflowState;

    for (input, expected) in [
        ("new", WorkflowState::New),
        ("reviewed", WorkflowState::Reviewed),
        ("needs_action", WorkflowState::NeedsAction),
        ("waiting", WorkflowState::Waiting),
        ("done", WorkflowState::Done),
        ("archived", WorkflowState::Archived),
        ("muted", WorkflowState::Muted),
        ("spam", WorkflowState::Spam),
    ] {
        let state = input.parse::<WorkflowState>().expect("valid state");
        assert_eq!(state, expected, "from_str({input}) should match");
    }
}

#[test]
fn workflow_state_from_str_invalid() {
    use hermes_hub_backend::domains::mail::messages::WorkflowState;

    assert!("".parse::<WorkflowState>().is_err());
    assert!("invalid_state".parse::<WorkflowState>().is_err());
    assert!("NEW".parse::<WorkflowState>().is_err());
}

#[test]
fn workflow_state_as_str_roundtrips() {
    use hermes_hub_backend::domains::mail::messages::WorkflowState;

    let states = [
        WorkflowState::New,
        WorkflowState::Reviewed,
        WorkflowState::NeedsAction,
        WorkflowState::Waiting,
        WorkflowState::Done,
        WorkflowState::Archived,
        WorkflowState::Muted,
        WorkflowState::Spam,
    ];

    for state in &states {
        let s = state.as_str();
        let roundtripped = s.parse::<WorkflowState>().expect("roundtrip");
        assert_eq!(*state, roundtripped, "roundtrip for {s}");
    }
}

#[test]
fn workflow_state_valid_transitions() {
    use hermes_hub_backend::domains::mail::messages::WorkflowState;

    // New can go to reviewed, needs_action, archived, muted, spam
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Reviewed
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::NeedsAction
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Muted
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Spam
    ));

    // New cannot go to done or waiting directly
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Done
    ));
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Waiting
    ));

    // NeedsAction can go to done, waiting, archived, reviewed
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Done
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Waiting
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Archived
    ));

    // Spam can go back to new (not spam)
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Spam,
        &WorkflowState::New
    ));

    // Done can go to archived
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Done,
        &WorkflowState::Archived
    ));

    // Archived can be restored to reviewed, needs_action, done
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Archived,
        &WorkflowState::Reviewed
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Archived,
        &WorkflowState::NeedsAction
    ));

    // Cannot transition to same state
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::New
    ));
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::Done,
        &WorkflowState::Done
    ));
}

#[test]
fn workflow_state_serde_roundtrips() {
    use hermes_hub_backend::domains::mail::messages::WorkflowState;

    let json = serde_json::to_string(&WorkflowState::NeedsAction).expect("serialize");
    assert_eq!(json, "\"needs_action\"");

    let deserialized: WorkflowState =
        serde_json::from_str("\"needs_action\"").expect("deserialize");
    assert_eq!(deserialized, WorkflowState::NeedsAction);

    let deserialized_new: WorkflowState = serde_json::from_str("\"new\"").expect("deserialize");
    assert_eq!(deserialized_new, WorkflowState::New);
}

#[tokio::test]
async fn message_projection_rejects_empty_fields_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message validation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_validation_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Validation Gmail",
        format!("validation-{suffix}@example.com"),
    )
    .await;

    // Empty subject should fail
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_empty_subj_{suffix}"),
        &format!("provider-empty-subj-{suffix}"),
        "", // empty subject
        "body",
    )
    .await;
    let result = project_raw_email_message(&message_store, &raw).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn message_workflow_state_transition_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("workflow state transition").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_workflow_{suffix}");
    let raw_record_id = format!("raw_workflow_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Workflow Gmail",
        format!("workflow-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &format!("provider-workflow-{suffix}"),
        "Workflow test subject",
        "Workflow test body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    // Default state is "new"
    assert_eq!(projected.workflow_state.as_str(), "new");

    // Transition to needs_action
    let updated = message_store
        .transition_workflow_state(
            &projected.message_id,
            hermes_hub_backend::domains::mail::messages::WorkflowState::NeedsAction,
        )
        .await
        .expect("transition to needs_action");
    assert_eq!(updated.workflow_state.as_str(), "needs_action");

    // Transition to done
    let updated = message_store
        .transition_workflow_state(
            &updated.message_id,
            hermes_hub_backend::domains::mail::messages::WorkflowState::Done,
        )
        .await
        .expect("transition to done");
    assert_eq!(updated.workflow_state.as_str(), "done");

    // Transition to archived
    let updated = message_store
        .transition_workflow_state(
            &updated.message_id,
            hermes_hub_backend::domains::mail::messages::WorkflowState::Archived,
        )
        .await
        .expect("transition to archived");
    assert_eq!(updated.workflow_state.as_str(), "archived");

    // Verify the message can be fetched with the new state
    let fetched = message_store
        .message(&projected.message_id)
        .await
        .expect("fetch message")
        .expect("message exists");
    assert_eq!(fetched.workflow_state.as_str(), "archived");
}

#[tokio::test]
async fn message_state_counts_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message state counts").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_counts_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Counts Gmail",
        format!("counts-{suffix}@example.com"),
    )
    .await;

    // Create two messages
    for i in 0..2 {
        let raw = record_raw_email_message(
            &communication_store,
            &account_id,
            &format!("raw_counts_{suffix}_{i}"),
            &format!("provider-counts-{suffix}-{i}"),
            &format!("Counts subject {i}"),
            &format!("Counts body {i}"),
        )
        .await;
        project_raw_email_message(&message_store, &raw)
            .await
            .expect("project message");
    }

    let counts = message_store
        .count_messages_by_state(Some(&account_id))
        .await
        .expect("count messages");

    let new_count = counts
        .iter()
        .find(|c| c.state.as_str() == "new")
        .map(|c| c.count)
        .unwrap_or(0);
    assert!(new_count >= 2, "expected at least 2 new messages");

    // Transition one to done
    let messages = message_store
        .list_messages(Some(&account_id), None, None, None, 10)
        .await
        .expect("list messages");
    assert!(!messages.is_empty());

    message_store
        .transition_workflow_state(
            &messages[0].message.message_id,
            hermes_hub_backend::domains::mail::messages::WorkflowState::Done,
        )
        .await
        .expect("transition to done");

    let counts = message_store
        .count_messages_by_state(Some(&account_id))
        .await
        .expect("count messages after transition");

    let done_count = counts
        .iter()
        .find(|c| c.state.as_str() == "done")
        .map(|c| c.count)
        .unwrap_or(0);
    assert_eq!(done_count, 1, "expected 1 done message");
}

#[tokio::test]
async fn message_list_filtering_by_state_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message list filtering").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_filter_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Filter Gmail",
        format!("filter-{suffix}@example.com"),
    )
    .await;

    // Create messages
    for i in 0..3 {
        let raw = record_raw_email_message(
            &communication_store,
            &account_id,
            &format!("raw_filter_{suffix}_{i}"),
            &format!("provider-filter-{suffix}-{i}"),
            &format!("Filter subject {i}"),
            &format!("Filter body {i}"),
        )
        .await;
        project_raw_email_message(&message_store, &raw)
            .await
            .expect("project message");
    }

    // All messages should be in "new" state
    let new_messages = message_store
        .list_messages(
            Some(&account_id),
            Some(hermes_hub_backend::domains::mail::messages::WorkflowState::New),
            None,
            None,
            10,
        )
        .await
        .expect("list new messages");
    assert!(new_messages.len() >= 3, "expected at least 3 new messages");

    // No messages should be in "done" state
    let done_messages = message_store
        .list_messages(
            Some(&account_id),
            Some(hermes_hub_backend::domains::mail::messages::WorkflowState::Done),
            None,
            None,
            10,
        )
        .await
        .expect("list done messages");
    assert_eq!(done_messages.len(), 0, "expected 0 done messages");
}

#[tokio::test]
async fn message_set_ai_analysis_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("ai analysis").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_ai_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "AI Gmail",
        format!("ai-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_ai_{suffix}"),
        &format!("provider-ai-{suffix}"),
        "AI test subject",
        "AI test body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    // Set AI analysis
    let updated = message_store
        .set_ai_analysis(
            &projected.message_id,
            Some("work"),
            Some("This is a work-related email about project updates."),
            Some(85),
        )
        .await
        .expect("set ai analysis");

    assert_eq!(updated.ai_category.as_deref(), Some("work"));
    assert_eq!(
        updated.ai_summary.as_deref(),
        Some("This is a work-related email about project updates.")
    );
    assert_eq!(updated.importance_score, Some(85));
    assert!(updated.ai_summary_generated_at.is_some());

    // Verify by fetching
    let fetched = message_store
        .message(&projected.message_id)
        .await
        .expect("fetch message")
        .expect("message exists");
    assert_eq!(fetched.ai_category.as_deref(), Some("work"));
    assert_eq!(fetched.importance_score, Some(85));
}

#[tokio::test]
async fn message_analytics_decodes_averages_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_analytics_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Analytics Gmail",
        format!("analytics-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_analytics_{suffix}"),
        &format!("provider-analytics-{suffix}"),
        "Analytics subject",
        "Analytics body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");
    message_store
        .set_ai_analysis(
            &projected.message_id,
            Some("work"),
            Some("summary"),
            Some(80),
        )
        .await
        .expect("set ai analysis");

    let analytics = EmailAnalyticsStore::new(pool);
    let health = analytics
        .mailbox_health(Some(&account_id))
        .await
        .expect("mailbox health");
    let senders = analytics
        .top_senders(Some(&account_id), 10)
        .await
        .expect("top senders");

    assert_eq!(health.total_messages, 1);
    assert_eq!(health.average_importance, 80.0);
    assert_eq!(senders.len(), 1);
    assert_eq!(senders[0].avg_importance, 80.0);
}

#[tokio::test]
async fn message_set_ai_analysis_rejects_invalid_score() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("ai analysis invalid score").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_ai_score_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "AI Score Gmail",
        format!("ai-score-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_ai_score_{suffix}"),
        &format!("provider-ai-score-{suffix}"),
        "Score test",
        "Score body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let result = message_store
        .set_ai_analysis(&projected.message_id, None, None, Some(101))
        .await;

    assert!(result.is_err());
}

#[test]
fn workflow_state_count_serialization() {
    use hermes_hub_backend::domains::mail::messages::{WorkflowState, WorkflowStateCount};

    let count = WorkflowStateCount {
        state: WorkflowState::NeedsAction,
        count: 42,
    };
    let json = serde_json::to_value(&count).expect("serialize");
    assert_eq!(json["state"], "needs_action");
    assert_eq!(json["count"], 42);
}
