use chrono::Utc;
use hermes_events_api::NewEventEnvelope;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

use crate::application::provider_runtime_services::WhatsAppProviderRuntimeRef;
use crate::domains::communications::messages::{
    CommunicationSignalProjectionError, MessageProjectionError, MessageProjectionStore,
    NewProjectedMessage, ProviderChannelMessageStore, ProviderCommunicationMessagePortError,
    project_accepted_signal_if_runtime_allows, project_whatsapp_content_observed,
    project_whatsapp_delivery_state_observed,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use crate::domains::personas::core::{PersonaCoreError, PersonaIdentityStore};
use crate::domains::signal_hub::store::SignalHubError;
use crate::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use crate::domains::signal_hub::whatsapp::dispatch_whatsapp_raw_signal;
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramError, TelegramMessageIngestResult, TelegramStore, telegram_chat_id,
};
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappWebCallIngestResult, WhatsappWebDialogIngestResult,
    WhatsappWebMediaIngestResult, WhatsappWebMessageDeleteIngestResult,
    WhatsappWebMessageIngestResult, WhatsappWebMessageUpdateIngestResult,
    WhatsappWebParticipantIngestResult, WhatsappWebPresenceIngestResult,
    WhatsappWebReactionIngestResult, WhatsappWebReceiptIngestResult,
    WhatsappWebRuntimeEventIngestResult, WhatsappWebStatusDeleteIngestResult,
    WhatsappWebStatusIngestResult, WhatsappWebStatusViewIngestResult,
};
use crate::platform::calls::CallError;
use crate::platform::calls::{CallDirection, CallIntelligenceStore, CallState, NewTelegramCall};
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::{telegram_event_types, whatsapp_event_types};
use crate::workflows::review_inbox::{
    ReviewInboxWorkflowError, refresh_message_decisions_into_review,
    refresh_message_knowledge_candidates_into_review,
    refresh_message_people_candidates_into_review, refresh_message_task_candidates_into_review,
};
use hermes_communications_api::evidence::NewRawCommunicationRecord;
use hermes_communications_postgres::store::CommunicationIngestionStore;
use hermes_events_postgres::errors::EventStoreError;
use hermes_events_postgres::store::EventStore;

const AUDIT_ACTOR_ID: &str = "hermes-frontend";
const WHATSAPP_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];
static WHATSAPP_FIXTURE_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct TelegramFixtureIngestApplicationService {
    pool: PgPool,
    store: TelegramStore,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
}

impl TelegramFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        store: TelegramStore,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            pool,
            store,
            event_store,
            event_bus,
        }
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.store.ingest_fixture_message(request).await?;
        let stored_raw = CommunicationIngestionStore::new(self.pool.clone())
            .record_raw_source(&observed.raw)
            .await?;
        let Some(accepted_event) =
            dispatch_telegram_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "telegram fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.pool.clone(), &accepted_event).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "telegram fixture signal did not produce an accepted projection".to_owned(),
            ));
        };
        let response = TelegramMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        };

        self.store
            .recompute_chat_unread_count(&observed.telegram_chat_id)
            .await?;
        let message_ids = vec![response.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;

        let event = build_event(
            telegram_event_types::MESSAGE_CREATED,
            &request.account_id,
            &response.message_id,
            telegram_message_snapshot_payload(
                &self.store,
                &response.message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "delivery_state": request.delivery_state.as_str(),
                    "runtime_kind": "fixture",
                }),
            )
            .await?,
        );
        self.publish_event(event).await?;

        Ok(response)
    }

    async fn publish_event(
        &self,
        event: NewEventEnvelope,
    ) -> Result<(), CommunicationFixtureIngestError> {
        if let Err(error) = self.event_store.append(&event).await {
            tracing::warn!(error = %error, "failed to append fixture ingest event");
            return Err(error.into());
        }
        let _ = self.event_bus.broadcast(event);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct WhatsappFixtureIngestApplicationService {
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
}

impl WhatsappFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        runtime: WhatsAppProviderRuntimeRef,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            pool,
            runtime,
            event_store,
            event_bus,
        }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) fn event_store(&self) -> &EventStore {
        &self.event_store
    }

    pub(crate) fn event_bus(&self) -> &InMemoryEventBus {
        &self.event_bus
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_with_reconciliation_source(request, "provider_observed.fixture_message")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message",
        )
        .await
    }

    async fn ingest_message_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessage,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_message(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = CommunicationIngestionStore::new(self.pool.clone())
            .record_raw_source(&observed_raw)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.pool.clone(), &accepted_event).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal did not produce an accepted projection".to_owned(),
            ));
        };
        self.project_whatsapp_message_refs(
            request,
            &projected.message_id,
            &projected.raw_record_id,
        )
        .await?;
        self.upsert_whatsapp_persona_identity_traces_for_message(
            request,
            &stored_raw.observation_id,
        )
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;
        let message_ids = vec![projected.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        if let Some(projected_message) = MessageProjectionStore::new(self.pool.clone())
            .message(&projected.message_id)
            .await?
        {
            let _ = refresh_message_people_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
            let _ = refresh_message_knowledge_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
        }

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        })
    }

    pub(crate) async fn ingest_reaction(
        &self,
        request: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        self.ingest_reaction_with_reconciliation_source(
            request,
            "provider_observed.fixture_reaction",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_reaction(
        &self,
        request: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        self.ingest_reaction_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_reaction",
        )
        .await
    }

    async fn ingest_reaction_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebReaction,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_reaction(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessageStore::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp reaction target message `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let reaction_id = whatsapp_reaction_id(
            &request.account_id,
            &request.provider_message_id,
            &request.provider_actor_id,
            &request.reaction,
        );
        sqlx::query(
            r#"
            INSERT INTO communication_message_reactions (
                reaction_id, message_id, account_id, provider_message_id,
                provider_conversation_id, sender_display_name, reaction, is_active,
                observed_at, source_event, provider_actor_id, metadata, provenance, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
            ON CONFLICT (reaction_id)
            DO UPDATE SET
                sender_display_name = EXCLUDED.sender_display_name,
                reaction = EXCLUDED.reaction,
                is_active = EXCLUDED.is_active,
                observed_at = EXCLUDED.observed_at,
                source_event = EXCLUDED.source_event,
                provider_actor_id = EXCLUDED.provider_actor_id,
                metadata = EXCLUDED.metadata,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            "#,
        )
        .bind(&reaction_id)
        .bind(&message.message_id)
        .bind(&request.account_id)
        .bind(&request.provider_message_id)
        .bind(&message.conversation_id)
        .bind(&request.sender_display_name)
        .bind(&request.reaction)
        .bind(request.is_active)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(&request.provider_actor_id)
        .bind(json!({
            "provider": account_context.provider_kind,
            "provider_chat_id": request.provider_chat_id,
        }))
        .bind(json!({
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_reaction_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebReactionIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            reaction_id,
        })
    }

    pub(crate) async fn ingest_message_update(
        &self,
        request: &NewWhatsappWebMessageUpdate,
    ) -> Result<WhatsappWebMessageUpdateIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_update_with_reconciliation_source(
            request,
            "provider_observed.fixture_message_update",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_message_update(
        &self,
        request: &NewWhatsappWebMessageUpdate,
    ) -> Result<WhatsappWebMessageUpdateIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_update_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message_update",
        )
        .await
    }

    async fn ingest_message_update_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessageUpdate,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageUpdateIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_message_update(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessageStore::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp message update target `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let updated_metadata = merged_whatsapp_message_metadata(
            &message.message_metadata,
            json!({
                "edited": true,
                "provider_chat_id": request.provider_chat_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
            }),
        )?;
        let updated_message = project_whatsapp_content_observed(
            self.pool.clone(),
            &message.message_id,
            &request.text,
            &updated_metadata,
            request.observed_at,
        )
        .await?
        .ok_or_else(|| {
            CommunicationFixtureIngestError::SignalControlBlocked(format!(
                "whatsapp message update target `{}` disappeared during projection",
                request.provider_message_id
            ))
        })?;
        let version_id = whatsapp_message_version_id(&stored_raw.accepted_event_id);
        let next_version_number: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(MAX(version_number), 0) + 1
            FROM communication_message_versions
            WHERE message_id = $1
            "#,
        )
        .bind(&message.message_id)
        .fetch_one(&self.pool)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO communication_message_versions (
                version_id, message_id, account_id, provider_message_id,
                provider_conversation_id, version_number, body_text, edited_at,
                source_event, diff_payload, provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (version_id) DO NOTHING
            "#,
        )
        .bind(&version_id)
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.conversation_id)
        .bind(next_version_number)
        .bind(&request.text)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(whatsapp_local_edit_diff(
            Some(message.body_text.as_str()),
            &request.text,
        ))
        .bind(json!({
            "provider": account_context.provider_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_update_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebMessageUpdateIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: updated_message.message_id,
            version_id,
        })
    }

    pub(crate) async fn ingest_message_delete(
        &self,
        request: &NewWhatsappWebMessageDelete,
    ) -> Result<WhatsappWebMessageDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_delete_with_reconciliation_source(
            request,
            "provider_observed.fixture_message_delete",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_message_delete(
        &self,
        request: &NewWhatsappWebMessageDelete,
    ) -> Result<WhatsappWebMessageDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_delete_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message_delete",
        )
        .await
    }

    async fn ingest_message_delete_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessageDelete,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageDeleteIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_message_delete(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessageStore::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp message delete target `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let tombstone_id = whatsapp_message_tombstone_id(&stored_raw.accepted_event_id);
        sqlx::query(
            r#"
            INSERT INTO communication_message_tombstones (
                tombstone_id, message_id, account_id, provider_message_id,
                provider_conversation_id, reason_class, actor_class, observed_at,
                source_event, is_provider_delete, is_local_visible, metadata, provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, TRUE, FALSE, $10, $11)
            ON CONFLICT (tombstone_id) DO NOTHING
            "#,
        )
        .bind(&tombstone_id)
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.conversation_id)
        .bind(&request.reason_class)
        .bind(&request.actor_class)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(json!({
            "provider": account_context.provider_kind,
            "provider_chat_id": request.provider_chat_id,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .bind(json!({
            "provider": account_context.provider_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_delete_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebMessageDeleteIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            tombstone_id,
        })
    }

    pub(crate) async fn ingest_receipt(
        &self,
        request: &NewWhatsappWebReceipt,
    ) -> Result<WhatsappWebReceiptIngestResult, CommunicationFixtureIngestError> {
        self.ingest_receipt_with_observed_source(request, "provider_observed.fixture_receipt")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_receipt(
        &self,
        request: &NewWhatsappWebReceipt,
    ) -> Result<WhatsappWebReceiptIngestResult, CommunicationFixtureIngestError> {
        self.ingest_receipt_with_observed_source(
            request,
            "provider_observed.runtime_bridge_receipt",
        )
        .await
    }

    async fn ingest_receipt_with_observed_source(
        &self,
        request: &NewWhatsappWebReceipt,
        observed_source: &str,
    ) -> Result<WhatsappWebReceiptIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_receipt(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessageStore::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp receipt target `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let updated_message = project_whatsapp_delivery_state_observed(
            self.pool.clone(),
            &message.message_id,
            request.delivery_state.as_message_delivery_state(),
            request.observed_at,
        )
        .await?
        .ok_or_else(|| {
            CommunicationFixtureIngestError::SignalControlBlocked(format!(
                "whatsapp receipt target `{}` disappeared during projection",
                request.provider_message_id
            ))
        })?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_receipt_commands(request)
                .await?,
            "provider_observation_consumer",
        )
        .await?;

        Ok(WhatsappWebReceiptIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: updated_message.message_id,
            delivery_state: request.delivery_state.as_str().to_owned(),
        })
    }

    pub(crate) async fn ingest_media(
        &self,
        request: &NewWhatsappWebMedia,
    ) -> Result<WhatsappWebMediaIngestResult, CommunicationFixtureIngestError> {
        self.ingest_media_with_reconciliation_source(request, "provider_observed.fixture_media")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_media(
        &self,
        request: &NewWhatsappWebMedia,
    ) -> Result<WhatsappWebMediaIngestResult, CommunicationFixtureIngestError> {
        self.ingest_media_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_media",
        )
        .await
    }

    async fn ingest_media_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMedia,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMediaIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_media(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessageStore::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp media target message `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let storage = CommunicationStorageStore::new(self.pool.clone());
        let storage_kind = normalized_whatsapp_media_storage_kind(&request.storage_kind);
        let sha256 = normalized_whatsapp_media_sha256(&request.sha256);
        let blob = storage
            .upsert_blob(
                &NewCommunicationBlob::new(
                    &storage_kind,
                    &request.storage_path,
                    &sha256,
                    request.size_bytes,
                )
                .content_type(&request.content_type),
            )
            .await?;
        let mut attachment = NewCommunicationAttachment::new(
            &message.message_id,
            &stored_raw.raw_record_id,
            &blob.blob_id,
            &request.provider_attachment_id,
            &request.content_type,
            request.size_bytes,
            &sha256,
        );
        if let Some(filename) = request.filename.as_deref() {
            attachment = attachment.filename(filename);
        }
        let attachment = storage.upsert_attachment(&attachment).await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_media_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebMediaIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            attachment_id: attachment.attachment_id,
        })
    }

    pub(crate) async fn ingest_status(
        &self,
        request: &NewWhatsappWebStatus,
    ) -> Result<WhatsappWebStatusIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_with_reconciliation_source(request, "provider_observed.fixture_status")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_status(
        &self,
        request: &NewWhatsappWebStatus,
    ) -> Result<WhatsappWebStatusIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_status",
        )
        .await
    }

    async fn ingest_status_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebStatus,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebStatusIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_status(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let channel_id = self.ensure_whatsapp_channel(&request.account_id).await?;
        let status_author_identity_id = self
            .upsert_whatsapp_status_identity(&request.account_id, request, &stored_raw)
            .await?;
        self.upsert_whatsapp_persona_identity_traces_for_status(request, &stored_raw)
            .await?;
        let status_feed_conversation_id = self
            .upsert_whatsapp_status_feed_conversation(
                &request.account_id,
                &channel_id,
                request,
                status_author_identity_id.as_deref(),
                &stored_raw,
            )
            .await?;
        let message = MessageProjectionStore::new(self.pool.clone())
            .upsert_channel_message(&NewProjectedMessage {
                message_id: whatsapp_status_message_id(
                    &request.account_id,
                    &request.provider_status_id,
                ),
                raw_record_id: stored_raw.raw_record_id.clone(),
                account_id: request.account_id.clone(),
                provider_record_id: request.provider_status_id.clone(),
                subject: "WhatsApp Status".to_owned(),
                sender: request.sender_id.clone(),
                recipients: vec![status_feed_conversation_id.clone()],
                body_text: request.text.clone(),
                occurred_at: Some(request.occurred_at),
                channel_kind: account_context.channel_kind.clone(),
                conversation_id: Some(status_feed_conversation_id),
                sender_display_name: Some(request.sender_display_name.clone()),
                delivery_state: "received".to_owned(),
                message_metadata: json!({
                    "communication_object_type": "status",
                    "provider_status_id": request.provider_status_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                    "status_author_identity_id": status_author_identity_id,
                    "status_author_identity_kind": request.sender_identity_kind,
                    "status_author_address": request.sender_address,
                    "status_author_push_name": request.sender_push_name,
                    "status_author_business_profile": request.sender_business_profile,
                    "status_author_profile_photo_ref": request.sender_profile_photo_ref,
                }),
            })
            .await?;
        let message_ids = vec![message.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        if let Some(projected_message) = MessageProjectionStore::new(self.pool.clone())
            .message(&message.message_id)
            .await?
        {
            let _ = refresh_message_people_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
            let _ = refresh_message_knowledge_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
        }
        let reconciled_commands = self
            .runtime
            .reconcile_fixture_status_commands(request)
            .await?;
        self.publish_whatsapp_command_reconciled_events(
            reconciled_commands.clone(),
            reconciliation_source,
        )
        .await?;
        self.publish_whatsapp_status_runtime_events(&reconciled_commands, reconciliation_source)
            .await?;

        Ok(WhatsappWebStatusIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
        })
    }

    async fn upsert_whatsapp_status_feed_conversation(
        &self,
        account_id: &str,
        channel_id: &str,
        request: &NewWhatsappWebStatus,
        status_author_identity_id: Option<&str>,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let conversation_id = whatsapp_status_feed_conversation_id(account_id);
        let metadata = json!({
            "provider": account_context.provider_kind,
            "chat_kind": "status_feed",
            "is_status_feed": true,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
            "provider_status_id": request.provider_status_id,
            "status_author_identity_id": status_author_identity_id,
            "status_author_identity_kind": request.sender_identity_kind,
            "status_author_address": request.sender_address,
            "status_author_push_name": request.sender_push_name,
            "status_author_business_profile": request.sender_business_profile,
            "status_author_profile_photo_ref": request.sender_profile_photo_ref,
        });
        sqlx::query(
            r#"
            INSERT INTO communication_conversations (
                conversation_id, account_id, channel_id, channel_kind,
                provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'status-feed', 'WhatsApp Status', $5, $6, now(), now())
            ON CONFLICT (conversation_id)
            DO UPDATE SET
                channel_id = EXCLUDED.channel_id,
                title = EXCLUDED.title,
                last_message_at = GREATEST(
                    COALESCE(communication_conversations.last_message_at, EXCLUDED.last_message_at),
                    EXCLUDED.last_message_at
                ),
                metadata = communication_conversations.metadata || EXCLUDED.metadata,
                updated_at = now()
            "#,
        )
        .bind(&conversation_id)
        .bind(account_id)
        .bind(channel_id)
        .bind(&account_context.channel_kind)
        .bind(request.occurred_at)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(conversation_id)
    }

    pub(crate) async fn ingest_status_view(
        &self,
        request: &NewWhatsappWebStatusView,
    ) -> Result<WhatsappWebStatusViewIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_view_with_observed_source(
            request,
            "provider_observed.fixture_status_view",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_status_view(
        &self,
        request: &NewWhatsappWebStatusView,
    ) -> Result<WhatsappWebStatusViewIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_view_with_observed_source(
            request,
            "provider_observed.runtime_bridge_status_view",
        )
        .await
    }

    async fn ingest_status_view_with_observed_source(
        &self,
        request: &NewWhatsappWebStatusView,
        observed_source: &str,
    ) -> Result<WhatsappWebStatusViewIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_status_view(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessageStore::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_status_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp status view target `{}` is not projected",
                    request.provider_status_id
                ))
            })?;
        let mut viewer_ids = message
            .message_metadata
            .get("status_viewer_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !viewer_ids
            .iter()
            .any(|value| value.as_str() == Some(request.viewer_id.as_str()))
        {
            viewer_ids.push(Value::String(request.viewer_id.clone()));
        }
        let updated_metadata = merged_whatsapp_message_metadata(
            &message.message_metadata,
            json!({
                "status_viewed": true,
                "status_last_viewed_at": request.observed_at,
                "status_view_count": viewer_ids.len(),
                "status_viewer_ids": viewer_ids,
                "status_last_viewer_id": request.viewer_id,
                "status_last_viewer_display_name": request.viewer_display_name,
                "status_view_observation_event_id": stored_raw.accepted_event_id,
            }),
        )?;
        let updated_message = MessageProjectionStore::new(self.pool.clone())
            .set_message_metadata_with_observation(
                &message.message_id,
                &updated_metadata,
                None,
                "status_view_observed",
                None,
            )
            .await?;

        Ok(WhatsappWebStatusViewIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: updated_message.message_id,
        })
    }

    pub(crate) async fn ingest_status_delete(
        &self,
        request: &NewWhatsappWebStatusDelete,
    ) -> Result<WhatsappWebStatusDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_delete_with_observed_source(
            request,
            "provider_observed.fixture_status_delete",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_status_delete(
        &self,
        request: &NewWhatsappWebStatusDelete,
    ) -> Result<WhatsappWebStatusDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_delete_with_observed_source(
            request,
            "provider_observed.runtime_bridge_status_delete",
        )
        .await
    }

    async fn ingest_status_delete_with_observed_source(
        &self,
        request: &NewWhatsappWebStatusDelete,
        observed_source: &str,
    ) -> Result<WhatsappWebStatusDeleteIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_status_delete(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessageStore::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_status_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp status delete target `{}` is not projected",
                    request.provider_status_id
                ))
            })?;
        let updated_metadata = merged_whatsapp_message_metadata(
            &message.message_metadata,
            json!({
                "status_deleted": true,
                "status_deleted_at": request.observed_at,
                "status_delete_actor_class": request.actor_class,
                "status_delete_reason_class": request.reason_class,
                "status_delete_observation_event_id": stored_raw.accepted_event_id,
            }),
        )?;
        MessageProjectionStore::new(self.pool.clone())
            .set_message_metadata_with_observation(
                &message.message_id,
                &updated_metadata,
                None,
                "status_delete_observed",
                None,
            )
            .await?;
        let tombstone_id = whatsapp_message_tombstone_id(&stored_raw.accepted_event_id);
        sqlx::query(
            r#"
            INSERT INTO communication_message_tombstones (
                tombstone_id, message_id, account_id, provider_message_id,
                provider_conversation_id, reason_class, actor_class, observed_at,
                source_event, is_provider_delete, is_local_visible, metadata, provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, TRUE, FALSE, $10, $11)
            ON CONFLICT (tombstone_id) DO NOTHING
            "#,
        )
        .bind(&tombstone_id)
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.conversation_id)
        .bind(&request.reason_class)
        .bind(&request.actor_class)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(json!({
            "provider": account_context.provider_kind,
            "communication_object_type": "status",
            "provider_status_id": request.provider_status_id,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .bind(json!({
            "provider": account_context.provider_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;

        Ok(WhatsappWebStatusDeleteIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            tombstone_id,
        })
    }

    pub(crate) async fn ingest_presence(
        &self,
        request: &NewWhatsappWebPresence,
    ) -> Result<WhatsappWebPresenceIngestResult, CommunicationFixtureIngestError> {
        self.ingest_presence_with_observed_source(request, "provider_observed.fixture_presence")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_presence(
        &self,
        request: &NewWhatsappWebPresence,
    ) -> Result<WhatsappWebPresenceIngestResult, CommunicationFixtureIngestError> {
        self.ingest_presence_with_observed_source(
            request,
            "provider_observed.runtime_bridge_presence",
        )
        .await
    }

    async fn ingest_presence_with_observed_source(
        &self,
        request: &NewWhatsappWebPresence,
        observed_source: &str,
    ) -> Result<WhatsappWebPresenceIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_presence(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let identity_row: Option<(String, Option<String>, Value)> = sqlx::query_as(
            r#"
            SELECT identity_id, display_name, metadata
            FROM communication_identities
            WHERE account_id = $1
              AND identity_kind = $2
              AND provider_identity_id = $3
            LIMIT 1
            "#,
        )
        .bind(&request.account_id)
        .bind(&request.identity_kind)
        .bind(&request.provider_identity_id)
        .fetch_optional(&self.pool)
        .await?;

        let identity_id =
            if let Some((identity_id, current_display_name, current_metadata)) = identity_row {
                let updated_metadata = merged_identity_display_name_metadata(
                    current_display_name.as_deref(),
                    &current_metadata,
                    Some(request.display_name.as_str()),
                    json!({
                        "presence_state": request.presence_state,
                        "presence_provider_chat_id": request.provider_chat_id,
                        "presence_observed_at": request.observed_at,
                        "last_seen_at": request.last_seen_at,
                        "presence_observation_event_id": stored_raw.accepted_event_id,
                    }),
                    request.observed_at,
                )?;
                sqlx::query(
                    r#"
                UPDATE communication_identities
                SET display_name = $2,
                    metadata = $3,
                    updated_at = now()
                WHERE identity_id = $1
                "#,
                )
                .bind(&identity_id)
                .bind(&request.display_name)
                .bind(updated_metadata)
                .execute(&self.pool)
                .await?;
                Some(identity_id)
            } else {
                None
            };

        Ok(WhatsappWebPresenceIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            identity_id,
        })
    }

    pub(crate) async fn ingest_call(
        &self,
        request: &NewWhatsappWebCall,
    ) -> Result<WhatsappWebCallIngestResult, CommunicationFixtureIngestError> {
        self.ingest_call_with_observed_source(request, "provider_observed.fixture_call")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_call(
        &self,
        request: &NewWhatsappWebCall,
    ) -> Result<WhatsappWebCallIngestResult, CommunicationFixtureIngestError> {
        self.ingest_call_with_observed_source(request, "provider_observed.runtime_bridge_call")
            .await
    }

    async fn ingest_call_with_observed_source(
        &self,
        request: &NewWhatsappWebCall,
        observed_source: &str,
    ) -> Result<WhatsappWebCallIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_call(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let call_id = whatsapp_call_id(&request.account_id, &request.provider_call_id);
        let direction = whatsapp_call_direction(&request.direction)?;
        let call_state = whatsapp_call_state(&request.call_state)?;
        let metadata = merged_object_metadata(
            &request.metadata,
            json!({
                "provider": account_context.provider_kind,
                "raw_record_id": stored_raw.raw_record_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
                "observed_at": request.observed_at,
            }),
        )?;
        CallIntelligenceStore::new(self.pool.clone())
            .upsert_call(&NewTelegramCall {
                call_id: call_id.clone(),
                account_id: request.account_id.clone(),
                provider_call_id: request.provider_call_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                direction,
                call_state,
                started_at: request.started_at,
                ended_at: request.ended_at,
                transcription_policy_id: None,
                metadata,
            })
            .await?;

        Ok(WhatsappWebCallIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            call_id,
        })
    }

    pub(crate) async fn ingest_runtime_event(
        &self,
        request: &NewWhatsappWebRuntimeEvent,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        self.ingest_runtime_event_with_observed_source(
            request,
            "provider_observed.fixture_runtime_event",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_runtime_event(
        &self,
        request: &NewWhatsappWebRuntimeEvent,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        self.ingest_runtime_event_with_observed_source(
            request,
            "provider_observed.runtime_bridge_runtime_event",
        )
        .await
    }

    async fn ingest_runtime_event_with_observed_source(
        &self,
        request: &NewWhatsappWebRuntimeEvent,
        observed_source: &str,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_runtime_event(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        Ok(WhatsappWebRuntimeEventIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: stored_raw.accepted_event_id,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn capture_runtime_lifecycle_event(
        &self,
        account_id: &str,
        provider_event_id: &str,
        runtime_event_kind: &str,
        runtime_status: Option<&str>,
        lifecycle_state: Option<&str>,
        severity: Option<&str>,
        metadata: Value,
        import_batch_id: &str,
        observed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let raw_record_id = whatsapp_runtime_event_raw_record_id(account_id, provider_event_id);
        let source_fingerprint = stable_whatsapp_id(
            "source_fingerprint:v5:whatsapp_web_runtime_event",
            &[
                account_id,
                provider_event_id,
                runtime_event_kind,
                runtime_status.unwrap_or(""),
                lifecycle_state.unwrap_or(""),
                severity.unwrap_or(""),
            ],
        );
        let observed_source = metadata
            .get("source")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(import_batch_id)
            .to_owned();
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            account_id,
            "whatsapp_web_runtime_event",
            provider_event_id,
            source_fingerprint,
            import_batch_id,
            json!({
                "provider_event_id": provider_event_id,
                "runtime_event_kind": runtime_event_kind,
                "runtime_status": runtime_status,
                "lifecycle_state": lifecycle_state,
                "severity": severity,
                "metadata": redact_secret_material(metadata),
            }),
        )
        .occurred_at(observed_at)
        .provenance(json!({
            "provider": account_context.provider_kind,
            "provider_kind": account_context.provider_kind,
            "account_id": account_id,
            "observed_source": observed_source,
            "captured_by": "application.communication_fixture_ingest.whatsapp_runtime_lifecycle",
        }));
        let stored_raw = self.record_and_accept_whatsapp_raw(&raw).await?;
        Ok(WhatsappWebRuntimeEventIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: stored_raw.accepted_event_id,
        })
    }

    pub(crate) async fn capture_media_lifecycle_event(
        &self,
        account_id: &str,
        command_id: &str,
        event_type: &str,
        metadata: Value,
        source: &str,
        observed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        let Some(runtime_event_kind) = whatsapp_media_runtime_event_kind(event_type) else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                format!("unsupported whatsapp media lifecycle event type `{event_type}`"),
            ));
        };
        let lifecycle_state = whatsapp_media_lifecycle_state(event_type);
        let has_runtime_blockers = metadata
            .get("runtime_blockers")
            .and_then(Value::as_array)
            .is_some_and(|blockers| !blockers.is_empty());
        let runtime_status = match lifecycle_state {
            "failed" if has_runtime_blockers => Some("blocked"),
            "failed" => Some("degraded"),
            _ => None,
        };
        let severity = match lifecycle_state {
            "failed" if has_runtime_blockers => Some("blocked"),
            "failed" => Some("warning"),
            _ => Some("info"),
        };
        let provider_event_id = format!(
            "{command_id}:{runtime_event_kind}:{}",
            observed_at.timestamp_micros()
        );
        self.capture_runtime_lifecycle_event(
            account_id,
            &provider_event_id,
            runtime_event_kind,
            runtime_status,
            Some(lifecycle_state),
            severity,
            merged_object_metadata(
                &metadata,
                json!({
                    "command_id": command_id,
                    "event_type": event_type,
                }),
            )?,
            source,
            observed_at,
        )
        .await
    }

    pub(crate) async fn ingest_dialog(
        &self,
        request: &NewWhatsappWebDialog,
    ) -> Result<WhatsappWebDialogIngestResult, CommunicationFixtureIngestError> {
        self.ingest_dialog_with_reconciliation_source(request, "provider_observed.fixture_dialog")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_dialog(
        &self,
        request: &NewWhatsappWebDialog,
    ) -> Result<WhatsappWebDialogIngestResult, CommunicationFixtureIngestError> {
        self.ingest_dialog_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_dialog",
        )
        .await
    }

    async fn ingest_dialog_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebDialog,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebDialogIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_dialog(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let channel_id = self.ensure_whatsapp_channel(&request.account_id).await?;
        let conversation_id = self
            .upsert_whatsapp_conversation(
                &request.account_id,
                &channel_id,
                &request.provider_chat_id,
                &request.chat_title,
                &request.chat_kind,
                request.is_archived,
                request.is_pinned,
                request.is_muted,
                request.is_unread,
                request.unread_count,
                request.participant_count,
                request.community_parent_chat_id.as_deref(),
                request.community_parent_title.as_deref(),
                request.invite_link.as_deref(),
                request.is_community_root,
                request.is_broadcast,
                request.is_newsletter,
                &request.avatar_metadata,
                &request.provider_labels,
                request.observed_at,
                &stored_raw,
            )
            .await?;
        let reconciled_commands = self
            .runtime
            .reconcile_fixture_dialog_commands(request)
            .await?;
        self.publish_whatsapp_command_reconciled_events(
            reconciled_commands.clone(),
            reconciliation_source,
        )
        .await?;
        self.publish_whatsapp_conversation_runtime_events(
            &reconciled_commands,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebDialogIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            channel_id,
            conversation_id,
        })
    }

    pub(crate) async fn ingest_participant(
        &self,
        request: &NewWhatsappWebParticipant,
    ) -> Result<WhatsappWebParticipantIngestResult, CommunicationFixtureIngestError> {
        self.ingest_participant_with_reconciliation_source(
            request,
            "provider_observed.fixture_participant",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_participant(
        &self,
        request: &NewWhatsappWebParticipant,
    ) -> Result<WhatsappWebParticipantIngestResult, CommunicationFixtureIngestError> {
        self.ingest_participant_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_participant",
        )
        .await
    }

    async fn ingest_participant_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebParticipant,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebParticipantIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_participant(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let channel_id = self.ensure_whatsapp_channel(&request.account_id).await?;
        let conversation_id =
            whatsapp_conversation_id(&request.account_id, &request.provider_chat_id);
        let previous_last_message_at = self
            .whatsapp_conversation_last_message_at(&conversation_id)
            .await?;
        let conversation_id = self
            .upsert_whatsapp_conversation(
                &request.account_id,
                &channel_id,
                &request.provider_chat_id,
                request.effective_chat_title(),
                request.effective_chat_kind(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &json!({}),
                &[],
                request.observed_at,
                &stored_raw,
            )
            .await?;
        self.restore_whatsapp_conversation_last_message_at(
            &request.account_id,
            &conversation_id,
            previous_last_message_at,
        )
        .await?;
        let identity_id = self
            .upsert_whatsapp_identity(&request.account_id, &channel_id, request, &stored_raw)
            .await?;
        self.upsert_whatsapp_persona_identity_traces_for_participant(request, &stored_raw)
            .await?;
        let participant_upsert = self
            .upsert_whatsapp_conversation_participant(
                &conversation_id,
                &identity_id,
                request,
                &stored_raw,
            )
            .await?;
        let reconciled_commands = self
            .runtime
            .reconcile_fixture_participant_commands(request)
            .await?;
        self.publish_whatsapp_command_reconciled_events(
            reconciled_commands.clone(),
            reconciliation_source,
        )
        .await?;
        self.publish_whatsapp_group_runtime_events(&reconciled_commands, reconciliation_source)
            .await?;

        Ok(WhatsappWebParticipantIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            conversation_id,
            participant_id: participant_upsert.participant_id,
            identity_id,
            previous_role: participant_upsert.previous_role,
            current_role: request.role.clone(),
            previous_status: participant_upsert.previous_status,
            current_status: request.status.clone(),
            role_changed: participant_upsert.role_changed,
            membership_changed: participant_upsert.membership_changed,
        })
    }

    async fn whatsapp_conversation_last_message_at(
        &self,
        conversation_id: &str,
    ) -> Result<Option<Option<chrono::DateTime<chrono::Utc>>>, CommunicationFixtureIngestError>
    {
        let last_message_at = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT last_message_at FROM communication_conversations WHERE conversation_id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(last_message_at)
    }

    async fn restore_whatsapp_conversation_last_message_at(
        &self,
        account_id: &str,
        conversation_id: &str,
        previous_last_message_at: Option<Option<chrono::DateTime<chrono::Utc>>>,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let Some(previous_last_message_at) = previous_last_message_at else {
            return Ok(());
        };
        sqlx::query(
            r#"
            UPDATE communication_conversations conversation
            SET last_message_at = $3,
                updated_at = now()
            WHERE conversation.conversation_id = $2
              AND conversation.account_id = $1
            "#,
        )
        .bind(account_id)
        .bind(conversation_id)
        .bind(previous_last_message_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn project_whatsapp_message_refs(
        &self,
        request: &NewWhatsappWebMessage,
        message_id: &str,
        raw_record_id: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let store = ProviderChannelMessageStore::new(self.pool.clone());

        if let Some(reply_to_provider_message_id) = request
            .reply_to_provider_message_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let target_message = store
                .message_by_provider_record_id(
                    &request.account_id,
                    reply_to_provider_message_id,
                    WHATSAPP_CHANNEL_KINDS,
                )
                .await?
                .ok_or_else(|| {
                    CommunicationFixtureIngestError::SignalControlBlocked(format!(
                        "whatsapp reply target `{reply_to_provider_message_id}` is not projected"
                    ))
                })?;
            let reply_ref_id = whatsapp_message_ref_id(
                &request.account_id,
                "reply",
                &request.provider_message_id,
                Some(reply_to_provider_message_id),
            );
            sqlx::query(
                r#"
                INSERT INTO communication_message_refs (
                    message_ref_id, ref_kind, source_message_id, target_message_id, account_id,
                    provider_conversation_id, source_provider_id, target_provider_id, depth,
                    metadata, provenance
                )
                VALUES ($1, 'reply', $2, $3, $4, $5, $6, $7, 1, $8, $9)
                ON CONFLICT (message_ref_id) DO NOTHING
                "#,
            )
            .bind(&reply_ref_id)
            .bind(message_id)
            .bind(&target_message.message_id)
            .bind(&request.account_id)
            .bind(&request.provider_chat_id)
            .bind(&request.provider_message_id)
            .bind(reply_to_provider_message_id)
            .bind(json!({
                "provider": account_context.provider_kind,
                "provider_chat_id": request.provider_chat_id,
                "raw_record_id": raw_record_id,
                "reply_to_provider_message_id": reply_to_provider_message_id,
            }))
            .bind(json!({
                "raw_record_id": raw_record_id,
                "relationship_kind": "reply",
            }))
            .execute(&self.pool)
            .await?;
        }

        let forward_origin_message_id = request
            .forward_origin_message_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let forward_origin_chat_id = request
            .forward_origin_chat_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if forward_origin_message_id.is_some() || forward_origin_chat_id.is_some() {
            let target_message = if let Some(origin_message_id) = forward_origin_message_id {
                store
                    .message_by_provider_record_id(
                        &request.account_id,
                        origin_message_id,
                        WHATSAPP_CHANNEL_KINDS,
                    )
                    .await?
            } else {
                None
            };
            let forward_ref_id = whatsapp_message_ref_id(
                &request.account_id,
                "forward",
                &request.provider_message_id,
                forward_origin_message_id,
            );
            sqlx::query(
                r#"
                INSERT INTO communication_message_refs (
                    message_ref_id, ref_kind, source_message_id, target_message_id, account_id,
                    provider_conversation_id, source_provider_id, target_provider_id, depth,
                    metadata, provenance
                )
                VALUES ($1, 'forward', $2, $3, $4, $5, $6, $7, 1, $8, $9)
                ON CONFLICT (message_ref_id) DO NOTHING
                "#,
            )
            .bind(&forward_ref_id)
            .bind(message_id)
            .bind(
                target_message
                    .as_ref()
                    .map(|message| message.message_id.as_str()),
            )
            .bind(&request.account_id)
            .bind(&request.provider_chat_id)
            .bind(&request.provider_message_id)
            .bind(forward_origin_message_id)
            .bind(json!({
                "provider": account_context.provider_kind,
                "provider_chat_id": request.provider_chat_id,
                "raw_record_id": raw_record_id,
                "forward_origin_chat_id": forward_origin_chat_id,
                "forward_origin_message_id": forward_origin_message_id,
                "forward_origin_sender_id": request.forward_origin_sender_id,
                "forward_origin_sender_name": request.forward_origin_sender_name,
                "forwarded_at": request.forwarded_at,
            }))
            .bind(json!({
                "raw_record_id": raw_record_id,
                "relationship_kind": "forward",
            }))
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn publish_whatsapp_command_reconciled_events(
        &self,
        commands: Vec<crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand>,
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            let payload = json!({
                "account_id": command.account_id,
                "command_id": command.command_id,
                "idempotency_key": command.idempotency_key,
                "command_kind": command.command_kind,
                "action": command.command_kind,
                "provider_chat_id": command.provider_chat_id,
                "provider_message_id": command.provider_message_id,
                "capability_state": command.capability_state,
                "action_class": command.action_class,
                "confirmation_decision": command.confirmation_decision,
                "status": command.status,
                "retry_count": command.retry_count,
                "max_retries": command.max_retries,
                "last_error": command.last_error,
                "result_payload": command.result_payload,
                "audit_metadata": command.audit_metadata,
                "provider_state": command.provider_state,
                "reconciliation_status": command.reconciliation_status,
                "next_attempt_at": command.next_attempt_at,
                "last_attempt_at": command.last_attempt_at,
                "provider_observed_at": command.provider_observed_at,
                "reconciled_at": command.reconciled_at,
                "dead_lettered_at": command.dead_lettered_at,
                "completed_at": command.completed_at,
                "source": source,
            });
            self.publish_whatsapp_command_event(
                whatsapp_event_types::COMMAND_STATUS_CHANGED,
                &command.command_id,
                &command.account_id,
                payload.clone(),
            )
            .await?;
            self.publish_whatsapp_command_event(
                whatsapp_event_types::COMMAND_RECONCILED,
                &command.command_id,
                &command.account_id,
                payload,
            )
            .await?;
        }
        Ok(())
    }

    async fn publish_whatsapp_status_runtime_events(
        &self,
        commands: &[crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand],
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            if command.command_kind != "publish_status" {
                continue;
            }
            let observed_at = command
                .provider_observed_at
                .or(command.completed_at)
                .unwrap_or_else(Utc::now);
            self.capture_runtime_lifecycle_event(
                &command.account_id,
                &format!(
                    "{}:status.publish.{}:{}",
                    command.command_id,
                    command.status,
                    observed_at.timestamp_micros()
                ),
                &format!("status.publish.{}", command.status),
                None,
                Some(command.status.as_str()),
                Some(if command.status == "failed" {
                    "warning"
                } else {
                    "info"
                }),
                json!({
                    "command_id": command.command_id,
                    "command_kind": command.command_kind,
                    "provider_chat_id": command.provider_chat_id,
                    "provider_status_id": command
                        .result_payload
                        .get("provider_status_id")
                        .cloned()
                        .or_else(|| command.provider_state.get("provider_status_id").cloned()),
                    "status": command.status,
                    "source": source,
                }),
                "status_publish_observed",
                observed_at,
            )
            .await?;
        }
        Ok(())
    }

    async fn publish_whatsapp_conversation_runtime_events(
        &self,
        commands: &[crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand],
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            let Some(runtime_event_kind) = (match command.command_kind.as_str() {
                "archive" => Some("conversation.archive.completed"),
                "unarchive" => Some("conversation.unarchive.completed"),
                "pin" => Some("conversation.pin.completed"),
                "unpin" => Some("conversation.unpin.completed"),
                "mute" => Some("conversation.mute.completed"),
                "unmute" => Some("conversation.unmute.completed"),
                "mark_read" => Some("conversation.mark_read.completed"),
                "mark_unread" => Some("conversation.mark_unread.completed"),
                _ => None,
            }) else {
                continue;
            };
            let observed_at = command
                .provider_observed_at
                .or(command.completed_at)
                .unwrap_or_else(Utc::now);
            self.capture_runtime_lifecycle_event(
                &command.account_id,
                &format!(
                    "{}:{}:{}",
                    command.command_id,
                    runtime_event_kind,
                    observed_at.timestamp_micros()
                ),
                runtime_event_kind,
                None,
                Some("completed"),
                Some("info"),
                json!({
                    "command_id": command.command_id,
                    "command_kind": command.command_kind,
                    "provider_chat_id": command.provider_chat_id,
                    "status": command.status,
                    "source": source,
                }),
                "conversation_command_observed",
                observed_at,
            )
            .await?;
        }
        Ok(())
    }

    async fn publish_whatsapp_group_runtime_events(
        &self,
        commands: &[crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand],
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            let Some(runtime_event_kind) = (match command.command_kind.as_str() {
                "join_group" => Some("group.join.completed"),
                "leave_group" => Some("group.leave.completed"),
                _ => None,
            }) else {
                continue;
            };
            let observed_at = command
                .provider_observed_at
                .or(command.completed_at)
                .unwrap_or_else(Utc::now);
            self.capture_runtime_lifecycle_event(
                &command.account_id,
                &format!(
                    "{}:{}:{}",
                    command.command_id,
                    runtime_event_kind,
                    observed_at.timestamp_micros()
                ),
                runtime_event_kind,
                None,
                Some("completed"),
                Some("info"),
                json!({
                    "command_id": command.command_id,
                    "command_kind": command.command_kind,
                    "provider_chat_id": command.provider_chat_id,
                    "status": command.status,
                    "source": source,
                }),
                "group_command_observed",
                observed_at,
            )
            .await?;
        }
        Ok(())
    }

    async fn publish_whatsapp_command_event(
        &self,
        event_type: &str,
        command_id: &str,
        account_id: &str,
        payload: Value,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let now = Utc::now();
        let source = payload
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("fixture_reconcile");
        let command_kind = payload
            .get("command_kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let status = payload
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let source_id = format!(
            "{}:{}:{}:{}:{}",
            command_id,
            command_kind,
            status,
            source,
            now.timestamp_micros()
        );
        let event = NewEventEnvelope::builder(
            whatsapp_fixture_event_id("command", command_id, now),
            event_type.to_owned(),
            now,
            json!({
                "channel": "whatsapp",
                "account_id": account_id,
                "actor_id": AUDIT_ACTOR_ID,
                "kind": "whatsapp_provider_commands",
                "source_id": source_id,
            }),
            json!({
                "id": command_id,
                "entity_id": command_id,
                "kind": "whatsapp_provider_command",
            }),
        )
        .payload(payload)
        .build()
        .expect("WhatsApp command reconciliation event envelope must be valid");
        self.event_store.append(&event).await?;
        let _ = self.event_bus.broadcast(event);
        Ok(())
    }

    async fn record_and_accept_whatsapp_raw(
        &self,
        raw: &hermes_communications_api::evidence::NewRawCommunicationRecord,
    ) -> Result<AcceptedWhatsappRawRecord, CommunicationFixtureIngestError> {
        let stored_raw = CommunicationIngestionStore::new(self.pool.clone())
            .record_raw_source(raw)
            .await?;
        self.ensure_canonical_communication_account(&stored_raw.account_id)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        Ok(AcceptedWhatsappRawRecord {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: accepted_event.event_id,
            observation_id: stored_raw.observation_id,
        })
    }

    async fn ensure_canonical_communication_account(
        &self,
        account_id: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        sqlx::query(
            r#"
            INSERT INTO communication_accounts (
                account_id, provider_kind, display_name, external_account_id,
                config, metadata, created_at, updated_at
            )
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                jsonb_build_object('source_table', 'communication_provider_accounts'),
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_whatsapp_channel(
        &self,
        account_id: &str,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let channel_id = whatsapp_channel_id(account_id);
        sqlx::query(
            r#"
            INSERT INTO communication_channels (
                channel_id, account_id, channel_kind, provider_kind, display_name,
                runtime_state, config, metadata, created_at, updated_at
            )
            SELECT
                $2,
                account_id,
                $3,
                provider_kind,
                display_name,
                'fixture',
                config,
                jsonb_build_object('source_table', 'communication_provider_accounts'),
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            ON CONFLICT (channel_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                provider_kind = EXCLUDED.provider_kind,
                runtime_state = EXCLUDED.runtime_state,
                config = EXCLUDED.config,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(account_id)
        .bind(&channel_id)
        .bind(&account_context.channel_kind)
        .execute(&self.pool)
        .await?;
        Ok(channel_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_whatsapp_conversation(
        &self,
        account_id: &str,
        channel_id: &str,
        provider_chat_id: &str,
        chat_title: &str,
        chat_kind: &str,
        is_archived: Option<bool>,
        is_pinned: Option<bool>,
        is_muted: Option<bool>,
        is_unread: Option<bool>,
        unread_count: Option<i64>,
        participant_count: Option<i64>,
        community_parent_chat_id: Option<&str>,
        community_parent_title: Option<&str>,
        invite_link: Option<&str>,
        is_community_root: Option<bool>,
        is_broadcast: Option<bool>,
        is_newsletter: Option<bool>,
        avatar_metadata: &Value,
        provider_labels: &[String],
        observed_at: chrono::DateTime<chrono::Utc>,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let conversation_id = whatsapp_conversation_id(account_id, provider_chat_id);
        let mut metadata = json!({
            "provider": account_context.provider_kind,
            "chat_kind": chat_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        });
        if let Some(is_archived) = is_archived {
            metadata["is_archived"] = json!(is_archived);
        }
        if let Some(is_pinned) = is_pinned {
            metadata["is_pinned"] = json!(is_pinned);
        }
        if let Some(is_muted) = is_muted {
            metadata["is_muted"] = json!(is_muted);
        }
        if let Some(is_unread) = is_unread {
            metadata["is_unread"] = json!(is_unread);
        }
        if let Some(community_parent_chat_id) = community_parent_chat_id {
            metadata["community_parent_chat_id"] = json!(community_parent_chat_id);
        }
        if let Some(community_parent_title) = community_parent_title {
            metadata["community_parent_title"] = json!(community_parent_title);
        }
        if let Some(invite_link) = invite_link {
            metadata["invite_link"] = json!(invite_link);
        }
        if let Some(is_community_root) = is_community_root {
            metadata["is_community_root"] = json!(is_community_root);
        }
        if let Some(is_broadcast) = is_broadcast {
            metadata["is_broadcast"] = json!(is_broadcast);
        }
        if let Some(is_newsletter) = is_newsletter {
            metadata["is_newsletter"] = json!(is_newsletter);
        }
        if let Some(unread_count) = unread_count {
            metadata["unread_count"] = json!(unread_count);
        }
        if let Some(participant_count) = participant_count {
            metadata["participant_count"] = json!(participant_count);
        }
        if avatar_metadata.is_object() && avatar_metadata != &json!({}) {
            metadata["avatar_metadata"] = avatar_metadata.clone();
        }
        if !provider_labels.is_empty() {
            metadata["provider_labels"] = json!(provider_labels);
        }
        sqlx::query(
            r#"
            INSERT INTO communication_conversations (
                conversation_id, account_id, channel_id, channel_kind,
                provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
            ON CONFLICT (conversation_id)
            DO UPDATE SET
                channel_id = EXCLUDED.channel_id,
                title = EXCLUDED.title,
                last_message_at = GREATEST(
                    COALESCE(communication_conversations.last_message_at, EXCLUDED.last_message_at),
                    EXCLUDED.last_message_at
                ),
                metadata = communication_conversations.metadata || EXCLUDED.metadata,
                updated_at = now()
        "#,
        )
        .bind(&conversation_id)
        .bind(account_id)
        .bind(channel_id)
        .bind(&account_context.channel_kind)
        .bind(provider_chat_id)
        .bind(chat_title)
        .bind(observed_at)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(conversation_id)
    }

    async fn upsert_whatsapp_identity(
        &self,
        account_id: &str,
        channel_id: &str,
        request: &NewWhatsappWebParticipant,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let identity_id = whatsapp_identity_id(
            account_id,
            &request.identity_kind,
            &request.provider_identity_id,
        );
        let existing_row: Option<(Option<String>, Value)> = sqlx::query_as(
            r#"
            SELECT display_name, metadata
            FROM communication_identities
            WHERE account_id = $1
              AND identity_kind = $2
              AND provider_identity_id = $3
            LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(&request.identity_kind)
        .bind(&request.provider_identity_id)
        .fetch_optional(&self.pool)
        .await?;
        let merged_metadata = if let Some((current_display_name, current_metadata)) = existing_row {
            merged_identity_display_name_metadata(
                current_display_name.as_deref(),
                &current_metadata,
                Some(request.display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "provider_member_id": request.provider_member_id,
                    "push_name": request.push_name,
                    "business_profile": request.business_profile,
                    "profile_photo_ref": request.profile_photo_ref,
                    "status": request.status,
                    "is_self": request.is_self,
                    "is_admin": request.is_admin,
                    "is_owner": request.is_owner,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.observed_at,
            )?
        } else {
            merged_identity_display_name_metadata(
                None,
                &json!({}),
                Some(request.display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "provider_member_id": request.provider_member_id,
                    "push_name": request.push_name,
                    "business_profile": request.business_profile,
                    "profile_photo_ref": request.profile_photo_ref,
                    "status": request.status,
                    "is_self": request.is_self,
                    "is_admin": request.is_admin,
                    "is_owner": request.is_owner,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.observed_at,
            )?
        };
        sqlx::query(
            r#"
            INSERT INTO communication_identities (
                identity_id, account_id, channel_id, identity_kind, provider_identity_id,
                display_name, address, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
            ON CONFLICT (account_id, identity_kind, provider_identity_id)
            DO UPDATE SET
                channel_id = EXCLUDED.channel_id,
                display_name = EXCLUDED.display_name,
                address = COALESCE(EXCLUDED.address, communication_identities.address),
                metadata = communication_identities.metadata || EXCLUDED.metadata,
                updated_at = now()
            "#,
        )
        .bind(&identity_id)
        .bind(account_id)
        .bind(channel_id)
        .bind(&request.identity_kind)
        .bind(&request.provider_identity_id)
        .bind(&request.display_name)
        .bind(&request.address)
        .bind(merged_metadata)
        .execute(&self.pool)
        .await?;
        Ok(identity_id)
    }

    async fn upsert_whatsapp_status_identity(
        &self,
        account_id: &str,
        request: &NewWhatsappWebStatus,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<Option<String>, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let Some(identity_kind) = request
            .sender_identity_kind
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };
        let channel_id = self.ensure_whatsapp_channel(account_id).await?;
        let identity_id = whatsapp_identity_id(account_id, identity_kind, &request.sender_id);
        let existing_row: Option<(Option<String>, Value)> = sqlx::query_as(
            r#"
            SELECT display_name, metadata
            FROM communication_identities
            WHERE account_id = $1
              AND identity_kind = $2
              AND provider_identity_id = $3
            LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(identity_kind)
        .bind(&request.sender_id)
        .fetch_optional(&self.pool)
        .await?;
        let merged_metadata = if let Some((current_display_name, current_metadata)) = existing_row {
            merged_identity_display_name_metadata(
                current_display_name.as_deref(),
                &current_metadata,
                Some(request.sender_display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "push_name": request.sender_push_name,
                    "business_profile": request.sender_business_profile,
                    "profile_photo_ref": request.sender_profile_photo_ref,
                    "status_author": true,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.occurred_at,
            )?
        } else {
            merged_identity_display_name_metadata(
                None,
                &json!({}),
                Some(request.sender_display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "push_name": request.sender_push_name,
                    "business_profile": request.sender_business_profile,
                    "profile_photo_ref": request.sender_profile_photo_ref,
                    "status_author": true,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.occurred_at,
            )?
        };
        sqlx::query(
            r#"
            INSERT INTO communication_identities (
                identity_id, account_id, channel_id, identity_kind, provider_identity_id,
                display_name, address, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
            ON CONFLICT (account_id, identity_kind, provider_identity_id)
            DO UPDATE SET
                channel_id = EXCLUDED.channel_id,
                display_name = EXCLUDED.display_name,
                address = COALESCE(EXCLUDED.address, communication_identities.address),
                metadata = communication_identities.metadata || EXCLUDED.metadata,
                updated_at = now()
            "#,
        )
        .bind(&identity_id)
        .bind(account_id)
        .bind(&channel_id)
        .bind(identity_kind)
        .bind(&request.sender_id)
        .bind(&request.sender_display_name)
        .bind(&request.sender_address)
        .bind(merged_metadata)
        .execute(&self.pool)
        .await?;
        Ok(Some(identity_id))
    }

    async fn upsert_whatsapp_persona_identity_traces_for_participant(
        &self,
        request: &NewWhatsappWebParticipant,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let participant_trace_metadata = json!({
            "whatsapp_participant_evidence": {
                "provider": account_context.provider_kind,
                "account_id": request.account_id,
                "provider_chat_id": request.provider_chat_id,
                "provider_member_id": request.effective_provider_member_id(),
                "provider_identity_id": request.provider_identity_id,
                "identity_kind": request.identity_kind,
                "display_name": request.display_name,
                "push_name": request.push_name,
                "address": request.address,
                "business_profile": request.business_profile,
                "profile_photo_ref": request.profile_photo_ref,
                "role": request.role,
                "status": request.status,
                "is_self": request.is_self,
                "is_admin": request.is_admin,
                "is_owner": request.is_owner,
                "raw_record_id": stored_raw.raw_record_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
            }
        });
        self.upsert_persona_identity_trace_with_metadata(
            "whatsapp",
            Some(request.provider_identity_id.as_str()),
            participant_trace_metadata.clone(),
            stored_raw,
        )
        .await?;
        self.upsert_persona_identity_trace_with_metadata(
            "phone",
            request.address.as_deref(),
            participant_trace_metadata.clone(),
            stored_raw,
        )
        .await?;
        let trace_value = format!(
            "whatsapp_participant:v1:{}:{}:{}",
            request.account_id,
            request.provider_chat_id,
            request.effective_provider_member_id()
        );
        self.upsert_persona_identity_trace_with_metadata(
            "message_participant",
            Some(trace_value.as_str()),
            participant_trace_metadata,
            stored_raw,
        )
        .await?;
        Ok(())
    }

    async fn upsert_whatsapp_persona_identity_traces_for_status(
        &self,
        request: &NewWhatsappWebStatus,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let status_trace_metadata = json!({
            "whatsapp_status_author_evidence": {
                "provider": account_context.provider_kind,
                "account_id": request.account_id,
                "provider_status_id": request.provider_status_id,
                "sender_id": request.sender_id,
                "sender_display_name": request.sender_display_name,
                "sender_identity_kind": request.sender_identity_kind,
                "sender_address": request.sender_address,
                "sender_push_name": request.sender_push_name,
                "sender_business_profile": request.sender_business_profile,
                "sender_profile_photo_ref": request.sender_profile_photo_ref,
                "raw_record_id": stored_raw.raw_record_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
            }
        });
        self.upsert_persona_identity_trace_with_metadata(
            "whatsapp",
            Some(request.sender_id.as_str()),
            status_trace_metadata.clone(),
            stored_raw,
        )
        .await?;
        self.upsert_persona_identity_trace_with_metadata(
            "phone",
            request.sender_address.as_deref(),
            status_trace_metadata,
            stored_raw,
        )
        .await?;
        Ok(())
    }

    async fn upsert_persona_identity_trace(
        &self,
        identity_type: &str,
        identity_value: Option<&str>,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        self.upsert_persona_identity_trace_with_metadata(
            identity_type,
            identity_value,
            json!({}),
            stored_raw,
        )
        .await
    }

    async fn upsert_persona_identity_trace_with_metadata(
        &self,
        identity_type: &str,
        identity_value: Option<&str>,
        metadata: Value,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let Some(identity_value) = identity_value
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        PersonaIdentityStore::new(self.pool.clone())
            .create_unattached_with_metadata_and_observation(
                identity_type,
                identity_value,
                "communication_projection",
                metadata,
                &stored_raw.observation_id,
            )
            .await?;
        Ok(())
    }

    async fn upsert_whatsapp_persona_identity_traces_for_message(
        &self,
        request: &NewWhatsappWebMessage,
        observation_id: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let Some(contact_card) = request.message_metadata.get("contact_card") else {
            return Ok(());
        };
        let Some(phones) = contact_card.get("phones").and_then(Value::as_array) else {
            return Ok(());
        };
        let store = PersonaIdentityStore::new(self.pool.clone());
        let contact_card_display_name = contact_card
            .get("display_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        for phone in phones
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            store
                .create_unattached_with_metadata_and_observation(
                    "phone",
                    phone,
                    "communication_projection",
                    json!({
                        "whatsapp_contact_card_evidence": {
                            "provider": account_context.provider_kind,
                            "account_id": request.account_id,
                            "provider_chat_id": request.provider_chat_id,
                            "provider_message_id": request.provider_message_id,
                            "sender_id": request.sender_id,
                            "sender_display_name": request.sender_display_name,
                            "contact_card": {
                                "display_name": contact_card_display_name.clone(),
                                "phones": phones,
                            }
                        }
                    }),
                    observation_id,
                )
                .await?;
        }
        Ok(())
    }

    async fn upsert_whatsapp_conversation_participant(
        &self,
        conversation_id: &str,
        identity_id: &str,
        request: &NewWhatsappWebParticipant,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<WhatsappParticipantUpsertOutcome, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let provider_member_id = request.effective_provider_member_id();
        let participant_id =
            whatsapp_conversation_participant_id(conversation_id, provider_member_id);
        let previous_row = sqlx::query(
            r#"
            SELECT role, metadata
            FROM communication_conversation_participants
            WHERE participant_id = $1
            "#,
        )
        .bind(&participant_id)
        .fetch_optional(&self.pool)
        .await?;
        let previous_role = previous_row
            .as_ref()
            .and_then(|row| row.try_get::<Option<String>, _>("role").ok())
            .flatten();
        let previous_metadata = previous_row
            .as_ref()
            .and_then(|row| row.try_get::<Option<Value>, _>("metadata").ok())
            .flatten()
            .unwrap_or_else(|| json!({}));
        let previous_status = previous_metadata
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let role_changed = previous_role
            .as_deref()
            .is_some_and(|previous| previous != request.role);
        let membership_changed = previous_status
            .as_deref()
            .is_some_and(|previous| previous != request.status);
        let mut metadata = json!({
            "provider": account_context.provider_kind,
            "provider_member_id": provider_member_id,
            "push_name": request.push_name,
            "business_profile": request.business_profile,
            "profile_photo_ref": request.profile_photo_ref,
            "status": request.status,
            "is_self": request.is_self,
            "is_admin": request.is_admin,
            "is_owner": request.is_owner,
            "role_observed_at": request.observed_at,
            "status_observed_at": request.observed_at,
            "last_membership_observed_at": request.observed_at,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        });
        if let Some(previous_role) = previous_role.as_deref() {
            metadata["previous_role"] = json!(previous_role);
        }
        if role_changed {
            metadata["last_role_change_at"] = json!(request.observed_at);
            metadata["role_changed"] = json!(true);
        }
        if let Some(previous_status) = previous_status.as_deref() {
            metadata["previous_status"] = json!(previous_status);
        }
        if membership_changed {
            metadata["last_membership_change_at"] = json!(request.observed_at);
            metadata["membership_changed"] = json!(true);
        }
        sqlx::query(
            r#"
            INSERT INTO communication_conversation_participants (
                participant_id, conversation_id, identity_id, persona_id, role,
                display_name, address, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, NULL, $4, $5, $6, $7, now(), now())
            ON CONFLICT (participant_id)
            DO UPDATE SET
                identity_id = EXCLUDED.identity_id,
                role = EXCLUDED.role,
                display_name = EXCLUDED.display_name,
                address = COALESCE(EXCLUDED.address, communication_conversation_participants.address),
                metadata = communication_conversation_participants.metadata || EXCLUDED.metadata,
                updated_at = now()
            "#,
        )
        .bind(&participant_id)
        .bind(conversation_id)
        .bind(identity_id)
        .bind(&request.role)
        .bind(&request.display_name)
        .bind(&request.address)
        .bind(metadata)
        .execute(&self.pool)
        .await?;
        Ok(WhatsappParticipantUpsertOutcome {
            participant_id,
            previous_role,
            previous_status,
            role_changed,
            membership_changed,
        })
    }

    async fn whatsapp_account_projection_context(
        &self,
        account_id: &str,
    ) -> Result<WhatsappAccountProjectionContext, CommunicationFixtureIngestError> {
        let provider_kind: String = sqlx::query_scalar(
            r#"
            SELECT provider_kind
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;
        let channel_kind = match provider_kind.as_str() {
            "whatsapp_web" | "whatsapp_business_cloud" => provider_kind.clone(),
            _ => "whatsapp_web".to_owned(),
        };
        Ok(WhatsappAccountProjectionContext {
            provider_kind,
            channel_kind,
        })
    }
}

struct WhatsappParticipantUpsertOutcome {
    participant_id: String,
    previous_role: Option<String>,
    previous_status: Option<String>,
    role_changed: bool,
    membership_changed: bool,
}

struct AcceptedWhatsappRawRecord {
    raw_record_id: String,
    accepted_event_id: String,
    observation_id: String,
}

struct WhatsappAccountProjectionContext {
    provider_kind: String,
    channel_kind: String,
}

#[derive(Debug, Error)]
pub(crate) enum CommunicationFixtureIngestError {
    #[error(transparent)]
    Telegram(#[from] TelegramError),

    #[error(transparent)]
    Whatsapp(#[from] WhatsappWebError),

    #[error(transparent)]
    Communication(#[from] hermes_communications_postgres::errors::CommunicationIngestionError),

    #[error(transparent)]
    ProviderMessage(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),

    #[error("{0}")]
    SignalControlBlocked(String),

    #[error(transparent)]
    Review(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Call(#[from] CallError),

    #[error(transparent)]
    PersonaCore(#[from] PersonaCoreError),
}

fn whatsapp_reaction_id(
    account_id: &str,
    provider_message_id: &str,
    provider_actor_id: &str,
    reaction: &str,
) -> String {
    stable_whatsapp_id(
        "reaction:v5:whatsapp_web",
        &[account_id, provider_message_id, provider_actor_id, reaction],
    )
}

fn whatsapp_status_message_id(account_id: &str, provider_status_id: &str) -> String {
    stable_whatsapp_id("message:v5:whatsapp_web", &[account_id, provider_status_id])
}

fn whatsapp_call_id(account_id: &str, provider_call_id: &str) -> String {
    stable_whatsapp_id("call:v5:whatsapp_web", &[account_id, provider_call_id])
}

fn whatsapp_channel_id(account_id: &str) -> String {
    stable_whatsapp_id("channel:v5:whatsapp_web", &[account_id])
}

fn whatsapp_conversation_id(account_id: &str, provider_chat_id: &str) -> String {
    stable_whatsapp_id(
        "conversation:v5:whatsapp_web",
        &[account_id, provider_chat_id],
    )
}

fn whatsapp_status_feed_conversation_id(account_id: &str) -> String {
    format!("whatsapp_status_feed:{account_id}")
}

fn annotate_whatsapp_raw_observed_source(
    raw: &hermes_communications_api::evidence::NewRawCommunicationRecord,
    observed_source: &str,
) -> Result<
    hermes_communications_api::evidence::NewRawCommunicationRecord,
    CommunicationFixtureIngestError,
> {
    let mut observed_raw = raw.clone();
    observed_raw.provenance = merged_object_metadata(
        &observed_raw.provenance,
        json!({
            "observed_source": observed_source,
        }),
    )?;
    Ok(observed_raw)
}

fn whatsapp_identity_id(
    account_id: &str,
    identity_kind: &str,
    provider_identity_id: &str,
) -> String {
    stable_whatsapp_id(
        "identity:v5:whatsapp_web",
        &[account_id, identity_kind, provider_identity_id],
    )
}

fn whatsapp_conversation_participant_id(conversation_id: &str, provider_member_id: &str) -> String {
    stable_whatsapp_id(
        "participant:v5:whatsapp_web",
        &[conversation_id, provider_member_id],
    )
}

fn whatsapp_message_version_id(accepted_event_id: &str) -> String {
    stable_whatsapp_id("message_version:v5:whatsapp_web", &[accepted_event_id])
}

fn whatsapp_message_tombstone_id(accepted_event_id: &str) -> String {
    stable_whatsapp_id("message_tombstone:v5:whatsapp_web", &[accepted_event_id])
}

fn whatsapp_message_ref_id(
    account_id: &str,
    ref_kind: &str,
    source_provider_message_id: &str,
    target_provider_message_id: Option<&str>,
) -> String {
    stable_whatsapp_id(
        "message_ref:v5:whatsapp_web",
        &[
            account_id,
            ref_kind,
            source_provider_message_id,
            target_provider_message_id.unwrap_or(""),
        ],
    )
}

fn whatsapp_runtime_event_raw_record_id(account_id: &str, provider_event_id: &str) -> String {
    stable_whatsapp_id(
        "raw:v5:whatsapp_web",
        &[account_id, "whatsapp_web_runtime_event", provider_event_id],
    )
}

fn whatsapp_media_runtime_event_kind(event_type: &str) -> Option<&'static str> {
    match event_type {
        "whatsapp.media.upload.requested" => Some("media.upload.requested"),
        "whatsapp.media.upload.started" => Some("media.upload.started"),
        "whatsapp.media.upload.progress" => Some("media.upload.progress"),
        "whatsapp.media.upload.completed" => Some("media.upload.completed"),
        "whatsapp.media.upload.failed" => Some("media.upload.failed"),
        "whatsapp.media.download.requested" => Some("media.download.requested"),
        "whatsapp.media.download.started" => Some("media.download.started"),
        "whatsapp.media.download.progress" => Some("media.download.progress"),
        "whatsapp.media.download.completed" => Some("media.download.completed"),
        "whatsapp.media.download.failed" => Some("media.download.failed"),
        _ => None,
    }
}

fn whatsapp_media_lifecycle_state(event_type: &str) -> &'static str {
    match event_type {
        "whatsapp.media.upload.requested" | "whatsapp.media.download.requested" => "requested",
        "whatsapp.media.upload.started" | "whatsapp.media.download.started" => "started",
        "whatsapp.media.upload.progress" | "whatsapp.media.download.progress" => "in_progress",
        "whatsapp.media.upload.completed" | "whatsapp.media.download.completed" => "completed",
        "whatsapp.media.upload.failed" | "whatsapp.media.download.failed" => "failed",
        _ => "observed",
    }
}

fn merged_whatsapp_message_metadata(
    current: &Value,
    patch: Value,
) -> Result<Value, CommunicationFixtureIngestError> {
    merged_object_metadata(current, patch)
}

fn merged_identity_display_name_metadata(
    current_display_name: Option<&str>,
    current_metadata: &Value,
    new_display_name: Option<&str>,
    patch: Value,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<Value, CommunicationFixtureIngestError> {
    let mut merged = merged_object_metadata(current_metadata, patch)?;
    let Some(new_display_name) = new_display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(merged);
    };

    let mut history = merged
        .get("display_name_history")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if let Some(previous_display_name) = current_display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        push_json_string_once(&mut history, previous_display_name);
        if previous_display_name != new_display_name {
            merged["previous_display_name"] = json!(previous_display_name);
            merged["display_name_changed_at"] = json!(observed_at);
        }
    }
    push_json_string_once(&mut history, new_display_name);

    merged["display_name_history"] = Value::Array(history);
    merged["display_name_observed_at"] = json!(observed_at);
    Ok(merged)
}

fn normalized_whatsapp_media_storage_kind(storage_kind: &str) -> String {
    match storage_kind.trim() {
        "local_blob" => "local_fs".to_owned(),
        value => value.to_owned(),
    }
}

fn normalized_whatsapp_media_sha256(sha256: &str) -> String {
    let value = sha256.trim();
    if value.starts_with("sha256:") {
        value.to_owned()
    } else {
        format!("sha256:{value}")
    }
}

fn whatsapp_call_direction(value: &str) -> Result<CallDirection, CommunicationFixtureIngestError> {
    match value.trim() {
        "incoming" => Ok(CallDirection::Incoming),
        "outgoing" => Ok(CallDirection::Outgoing),
        other => Err(CommunicationFixtureIngestError::SignalControlBlocked(
            format!("unsupported whatsapp call direction `{other}`"),
        )),
    }
}

fn whatsapp_call_state(value: &str) -> Result<CallState, CommunicationFixtureIngestError> {
    match value.trim() {
        "ringing" => Ok(CallState::Ringing),
        "active" => Ok(CallState::Active),
        "ended" => Ok(CallState::Ended),
        "missed" => Ok(CallState::Missed),
        "declined" => Ok(CallState::Declined),
        "failed" => Ok(CallState::Failed),
        other => Err(CommunicationFixtureIngestError::SignalControlBlocked(
            format!("unsupported whatsapp call state `{other}`"),
        )),
    }
}

fn merged_object_metadata(
    current: &Value,
    patch: Value,
) -> Result<Value, CommunicationFixtureIngestError> {
    let Value::Object(mut current_map) = current.clone() else {
        return Err(CommunicationFixtureIngestError::SignalControlBlocked(
            "metadata is not a JSON object".to_owned(),
        ));
    };
    let Value::Object(patch_map) = patch else {
        return Err(CommunicationFixtureIngestError::SignalControlBlocked(
            "metadata patch is not a JSON object".to_owned(),
        ));
    };
    current_map.extend(patch_map);
    Ok(Value::Object(current_map))
}

fn redact_secret_material(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_secret_like_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_material(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_material).collect())
        }
        other => other,
    }
}

fn is_secret_like_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "access_token"
            | "refresh_token"
            | "session_key"
            | "session_material"
            | "authorization"
            | "cookie"
            | "token"
            | "secret"
            | "secret_key"
            | "password"
    )
}

fn push_json_string_once(items: &mut Vec<Value>, value: &str) {
    if !items.iter().any(|item| item.as_str() == Some(value)) {
        items.push(Value::String(value.to_owned()));
    }
}

fn whatsapp_local_edit_diff(previous_text: Option<&str>, new_text: &str) -> Value {
    json!({
        "kind": "local_edit_diff",
        "previous_text": previous_text,
        "new_text": new_text,
    })
}

fn stable_whatsapp_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("{prefix}:{:x}", hasher.finalize())
}

fn whatsapp_fixture_event_id(
    scope: &str,
    subject: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> String {
    let seq = WHATSAPP_FIXTURE_EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_whatsapp_fixture_{}_{}_{}_{}",
        scope,
        subject.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        now.timestamp_nanos_opt().unwrap_or_default(),
        seq
    )
}

pub(crate) fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id, "actor_id": AUDIT_ACTOR_ID}),
        json!({"id": subject_id, "kind": "telegram_message"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

pub(crate) async fn telegram_message_snapshot_payload(
    store: &TelegramStore,
    message_id: &str,
    base_payload: serde_json::Value,
) -> Result<serde_json::Value, TelegramError> {
    let mut payload = match base_payload {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };

    if let Some(message) = store.message_by_id(message_id).await? {
        payload.insert("message".to_owned(), json!(message));
        if let Some(provider_chat_id) = message.provider_chat_id.as_deref() {
            let projected_chat = store
                .telegram_chat(&message.account_id, provider_chat_id)
                .await?;
            let resolved_chat_id = projected_chat
                .as_ref()
                .map(|chat| chat.telegram_chat_id.clone())
                .unwrap_or_else(|| telegram_chat_id(&message.account_id, provider_chat_id));
            payload.insert("telegram_chat_id".to_owned(), json!(resolved_chat_id));
            if let Some(chat) = projected_chat {
                payload.insert("chat".to_owned(), json!(chat));
            }
        }
    }

    Ok(serde_json::Value::Object(payload))
}
