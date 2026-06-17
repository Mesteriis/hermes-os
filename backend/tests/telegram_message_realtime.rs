use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
};
use hermes_hub_backend::integrations::telegram::client::lifecycle::{
    self, reconcile_delete_commands_from_provider_state,
    reconcile_edit_commands_from_provider_state,
    reconcile_message_pin_commands_from_provider_state, record_provider_delete_observation,
    record_provider_edit_observation,
};
use hermes_hub_backend::integrations::telegram::client::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState, TelegramStore,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_provider_delete_observation_is_idempotent_and_reconciles_delete_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "message-delete", "telegram:delete").await;
    let store = TelegramStore::new(pool.clone());
    let provider_chat_id = "-100message-delete";
    let provider_message_id = format!("{provider_chat_id}:42");

    let ingested = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Delete Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");
    let message = store
        .message_by_id(&ingested.message_id)
        .await
        .expect("load message")
        .expect("message");

    lifecycle::insert_command(
        &pool,
        "tcmd_delete_observed",
        &account_id,
        "delete",
        "delete-observed",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "destructive",
        "confirmed",
        "hermes-frontend",
        json!({"reason_class": "deleted_by_owner", "is_provider_delete": true}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert delete command");

    let first_tombstone = record_provider_delete_observation(
        &pool,
        &message,
        Utc::now(),
        "updateDeleteMessages",
        true,
        false,
    )
    .await
    .expect("first tombstone");
    let second_tombstone = record_provider_delete_observation(
        &pool,
        &message,
        Utc::now(),
        "updateDeleteMessages",
        true,
        false,
    )
    .await
    .expect("second tombstone");

    assert_eq!(first_tombstone.tombstone_id, second_tombstone.tombstone_id);
    assert_eq!(first_tombstone.reason_class, "deleted_by_provider");
    assert_eq!(first_tombstone.actor_class, "provider");
    assert!(!first_tombstone.is_local_visible);

    let reconciled = reconcile_delete_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        Utc::now(),
        "tdlib.updateDeleteMessages",
    )
    .await
    .expect("reconcile delete commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_delete_observed");
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");

    let tombstones = lifecycle::list_tombstones(&pool, &message.message_id)
        .await
        .expect("list tombstones");
    assert_eq!(tombstones.len(), 1);
}

#[tokio::test]
async fn telegram_provider_edit_observation_is_idempotent_and_reconciles_edit_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "message-edit", "telegram:edit").await;
    let store = TelegramStore::new(pool.clone());
    let provider_chat_id = "-100message-edit";
    let provider_message_id = format!("{provider_chat_id}:42");

    let ingested = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edit Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");
    let message = store
        .message_by_id(&ingested.message_id)
        .await
        .expect("load message")
        .expect("message");

    lifecycle::insert_command(
        &pool,
        "tcmd_edit_observed",
        &account_id,
        "edit",
        "edit-observed",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "hermes-frontend",
        json!({"new_text": "after"}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert edit command");

    let first_version = record_provider_edit_observation(
        &pool,
        &message,
        "after",
        Utc::now(),
        "updateMessageContent",
        json!({"previous_text": "before", "new_text": "after"}),
        json!({"provider": "telegram", "runtime": "tdlib"}),
    )
    .await
    .expect("first version");
    let second_version = record_provider_edit_observation(
        &pool,
        &message,
        "after",
        first_version.edit_timestamp,
        "updateMessageContent",
        json!({"previous_text": "before", "new_text": "after"}),
        json!({"provider": "telegram", "runtime": "tdlib"}),
    )
    .await
    .expect("second version");

    assert_eq!(first_version.version_id, second_version.version_id);
    assert_eq!(first_version.body_text.as_deref(), Some("after"));
    assert_eq!(
        first_version.source_event.as_deref(),
        Some("updateMessageContent")
    );

    let reconciled = reconcile_edit_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        "after",
        Utc::now(),
        "tdlib.updateMessageContent",
    )
    .await
    .expect("reconcile edit commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_edit_observed");
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");
}

#[tokio::test]
async fn telegram_provider_edit_observation_marks_mismatched_edit_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id =
        create_telegram_account(&pool, "message-edit-mismatch", "telegram:edit-mismatch").await;
    let store = TelegramStore::new(pool.clone());
    let provider_chat_id = "-100message-edit-mismatch";
    let provider_message_id = format!("{provider_chat_id}:42");

    store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edit Mismatch Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    lifecycle::insert_command(
        &pool,
        "tcmd_edit_mismatch",
        &account_id,
        "edit",
        "edit-mismatch",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "hermes-frontend",
        json!({"new_text": "expected provider body"}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert edit command");

    let reconciled = reconcile_edit_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        "different provider body",
        Utc::now(),
        "tdlib.updateMessageContent",
    )
    .await
    .expect("reconcile mismatched edit commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_edit_mismatch");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different message body than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_body_text"],
        json!("expected provider body")
    );
    assert_eq!(
        reconciled[0].provider_state["observed_body_text"],
        json!("different provider body")
    );
    assert!(reconciled[0].completed_at.is_none());
    assert!(reconciled[0].reconciled_at.is_some());
}

#[tokio::test]
async fn telegram_provider_pin_state_reconciles_message_pin_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "message-pin", "telegram:pin").await;
    let store = TelegramStore::new(pool.clone());
    let provider_chat_id = "-100message-pin";
    let provider_message_id = format!("{provider_chat_id}:42");

    store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Pin Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "pin me".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    lifecycle::insert_command(
        &pool,
        "tcmd_pin_observed",
        &account_id,
        "pin",
        "pin-observed",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "hermes-frontend",
        json!({"is_pinned": true}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert pin command");

    let reconciled = reconcile_message_pin_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        true,
        Utc::now(),
        "tdlib.updateMessageIsPinned",
    )
    .await
    .expect("reconcile message pin commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_pin_observed");
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");
}

#[tokio::test]
async fn telegram_provider_pin_state_marks_mismatched_unpin_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id =
        create_telegram_account(&pool, "message-pin-mismatch", "telegram:pin-mismatch").await;
    let store = TelegramStore::new(pool.clone());
    let provider_chat_id = "-100message-pin-mismatch";
    let provider_message_id = format!("{provider_chat_id}:42");

    store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Pin Mismatch Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "pin mismatch".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    lifecycle::insert_command(
        &pool,
        "tcmd_unpin_mismatch",
        &account_id,
        "unpin",
        "unpin-mismatch",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "hermes-frontend",
        json!({"is_pinned": false}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert unpin command");

    let reconciled = reconcile_message_pin_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        true,
        Utc::now(),
        "tdlib.updateMessageIsPinned",
    )
    .await
    .expect("reconcile mismatched pin commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_unpin_mismatch");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different pin state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_pinned"],
        json!(false)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_pinned"],
        json!(true)
    );
    assert!(reconciled[0].completed_at.is_none());
    assert!(reconciled[0].reconciled_at.is_some());
}

async fn create_telegram_account(
    pool: &sqlx::PgPool,
    suffix: &str,
    external_account_id: &str,
) -> String {
    let account_id = format!("telegram-realtime-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Realtime {suffix}"),
                external_account_id.to_owned(),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    account_id
}
