use crate::support::*;

#[tokio::test]
async fn communication_ingestion_records_raw_sources_idempotently_against_postgres() {
    let Some(database) =
        connect_database("communication raw source test: HERMES_TEST_DATABASE_URL is not set")
            .await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_raw_{suffix}");
    let provider_record_id = format!("gmail-message-{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Gmail raw source test",
            format!("raw-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail"}),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "gmail-api"})),
        )
        .await
        .expect("record raw source");

    let duplicate = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_duplicate_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:different-{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail", "changed": true}),
            )
            .provenance(json!({"source": "retry"})),
        )
        .await
        .expect("record duplicate raw source");

    assert_eq!(duplicate.raw_record_id, first.raw_record_id);
    assert_eq!(duplicate.payload, first.payload);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND record_kind = 'email_message'
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 1);

    let mutation = sqlx::query(
        "UPDATE communication_raw_records SET payload = '{}'::jsonb WHERE raw_record_id = $1",
    )
    .bind(&first.raw_record_id)
    .execute(&pool)
    .await;
    assert!(
        mutation.is_err(),
        "raw provider records must be append-only"
    );
}
