use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::domains::mail::core::NewRawCommunicationRecord;
use hermes_hub_backend::domains::mail::messages::{
    NewProjectedMessage, project_raw_email_message, project_raw_email_message_from_blob,
};
use hermes_hub_backend::domains::mail::storage::LocalMailBlobStore;

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

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
