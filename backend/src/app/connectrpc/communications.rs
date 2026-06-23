use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, Router as ConnectRouter, ServiceRequest,
    ServiceResult,
};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use crate::app::ApiError;
use crate::app::handlers::communications::{
    WorkflowActionInput as HandlerWorkflowActionInput,
    WorkflowActionKind as HandlerWorkflowActionKind,
    WorkflowActionRequest as HandlerWorkflowActionRequest,
    WorkflowActionResponse as HandlerWorkflowActionResponse,
    WorkflowActionSource as HandlerWorkflowActionSource,
    WorkflowActionStatus as HandlerWorkflowActionStatus,
    WorkflowActionTargetKind as HandlerWorkflowActionTargetKind, execute_workflow_action,
};
use crate::application::communication_send::{
    CommunicationSendDependencies, CommunicationSendError, CommunicationSendRequest, send_email,
};
use crate::contracts::hermes::common::v1::PageResponse;
use crate::contracts::hermes::communications::v1::{
    AddMessageLabelResponse, AiReplyRequest as ProtoAiReplyRequest,
    AiReplyResponse as ProtoAiReplyResponse, AiReplyVariantsRequest as ProtoAiReplyVariantsRequest,
    AiReplyVariantsResponse as ProtoAiReplyVariantsResponse, AnalyzeMessageRequest,
    AnalyzeMessageResponse, ArchiveInspectionEntry as ProtoArchiveInspectionEntry,
    ArchiveInspectionReport as ProtoArchiveInspectionReport,
    AttachmentSearchItem as ProtoAttachmentSearchItem, AttachmentSearchRequest,
    AttachmentSearchResponse, BulkMessageActionRequest as ProtoBulkMessageActionRequest,
    BulkMessageActionResponse as ProtoBulkMessageActionResponse,
    CommunicationArchitectureBlocker as ProtoCommunicationArchitectureBlocker,
    CommunicationDraft as ProtoCommunicationDraft, CommunicationFolder as ProtoCommunicationFolder,
    CommunicationMessage as ProtoCommunicationMessage,
    CommunicationMessageAttachment as ProtoCommunicationMessageAttachment,
    CommunicationOutboxItem as ProtoCommunicationOutboxItem,
    CommunicationPersona as ProtoCommunicationPersona,
    CommunicationSavedSearch as ProtoCommunicationSavedSearch,
    CommunicationSearchResult as ProtoCommunicationSearchResult,
    CommunicationThread as ProtoCommunicationThread, CommunicationsService,
    CommunicationsServiceExt, CopyMessageToFolderRequest, CopyMessageToFolderResponse,
    CreateDraftRequest, CreateDraftResponse, CreateFolderRequest, CreateFolderResponse,
    CreateSavedSearchRequest, CreateSavedSearchResponse, DeleteDraftRequest, DeleteDraftResponse,
    DeleteFolderRequest, DeleteFolderResponse, DeleteMessageFromProviderRequest,
    DeleteMessageFromProviderResponse, DeleteRichTemplateRequest, DeleteRichTemplateResponse,
    DeleteSavedSearchRequest, DeleteSavedSearchResponse, DetectMessageLanguageRequest,
    DetectMessageLanguageResponse, ExplainMessageRequest, ExplainMessageResponse,
    ExtractMessageNotesRequest, ExtractMessageNotesResponse, ExtractMessageTasksRequest,
    ExtractMessageTasksResponse, ExtractedNote as ProtoExtractedNote,
    ExtractedTask as ProtoExtractedTask, FolderMessage as ProtoFolderMessage,
    FolderMessageActionResult as ProtoFolderMessageActionResult,
    GetAttachmentArchiveInspectionRequest, GetAttachmentArchiveInspectionResponse,
    GetAttachmentPreviewRequest, GetAttachmentPreviewResponse, GetMailboxHealthRequest,
    GetMailboxHealthResponse, GetMessageAuthRequest, GetMessageAuthResponse,
    GetMessageExportRequest, GetMessageExportResponse, GetMessageRequest, GetMessageResponse,
    GetMessageSignatureRequest, GetMessageSignatureResponse, GetMessageSmartCcRequest,
    GetMessageSmartCcResponse, ListCommunicationBlockersRequest, ListCommunicationBlockersResponse,
    ListCommunicationPersonasRequest, ListCommunicationPersonasResponse, ListDraftsRequest,
    ListDraftsResponse, ListFolderMessagesRequest, ListFolderMessagesResponse, ListFoldersRequest,
    ListFoldersResponse, ListMessageWorkflowStateCountsRequest,
    ListMessageWorkflowStateCountsResponse, ListMessagesRequest, ListMessagesResponse,
    ListOutboxRequest, ListOutboxResponse, ListRichTemplatesRequest, ListRichTemplatesResponse,
    ListSavedSearchesRequest, ListSavedSearchesResponse, ListSubscriptionsRequest,
    ListSubscriptionsResponse, ListThreadMessagesRequest, ListThreadMessagesResponse,
    ListThreadsRequest, ListThreadsResponse, ListTopSendersRequest, ListTopSendersResponse,
    MailboxHealth as ProtoMailboxHealth, MarkMessageReadRequest, MarkMessageReadResponse,
    MessageAuthReport as ProtoMessageAuthReport, MessageAuthResult as ProtoMessageAuthResult,
    MessageAuthRiskReport as ProtoMessageAuthRiskReport,
    MessageKnowledgeCandidate as ProtoMessageKnowledgeCandidate,
    MessageSummaryContract as ProtoMessageSummaryContract, MessageToggleRequest,
    MoveMessageToFolderRequest, MoveMessageToFolderResponse, RedirectMessageRequest,
    RedirectMessageResponse, RemoveMessageLabelResponse,
    RenderedRichTemplate as ProtoRenderedRichTemplate, RichTemplate as ProtoRichTemplate,
    RichTemplateMailMergePreviewItem as ProtoRichTemplateMailMergePreviewItem,
    RichTemplateMailMergePreviewRequest, RichTemplateMailMergePreviewResponse,
    RichTemplateRenderRequest, RichTemplateRenderResponse, SearchMessagesRequest,
    SearchMessagesResponse, SendMessageRequest, SendMessageResponse,
    SenderStats as ProtoSenderStats, SnoozeMessageRequest, SnoozeMessageResponse,
    SubscriptionSource as ProtoSubscriptionSource, ThreadMessage as ProtoThreadMessage,
    ThreadTranslationItem as ProtoThreadTranslationItem, ToggleMessageImportantResponse,
    ToggleMessageMuteResponse, ToggleMessagePinResponse, TransitionMessageWorkflowStateRequest,
    TransitionMessageWorkflowStateResponse, TranslateAttachmentRequest,
    TranslateAttachmentResponse, TranslateMessageRequest, TranslateMessageResponse,
    TranslateThreadRequest, TranslateThreadResponse, UndoOutboxItemRequest, UndoOutboxItemResponse,
    UpdateFolderRequest, UpdateFolderResponse, UpdateMessageLabelRequest,
    UpdateMessageLocalStateRequest, UpdateMessageLocalStateResponse, UpdateSavedSearchRequest,
    UpdateSavedSearchResponse, UpsertRichTemplateRequest, UpsertRichTemplateResponse,
    WorkflowActionProvenance as ProtoWorkflowActionProvenance,
    WorkflowActionRequest as ProtoWorkflowActionRequest,
    WorkflowActionResponse as ProtoWorkflowActionResponse,
    WorkflowActionTarget as ProtoWorkflowActionTarget,
    WorkflowStateCount as ProtoWorkflowStateCount,
};
use crate::domains::communications::analytics::{
    EmailAnalyticsError, EmailAnalyticsStore, MailboxHealth, SenderStats,
};
use crate::domains::communications::archive_inspection::{
    ArchiveInspectionLimits, ArchiveInspectionReport, inspect_zip_bytes,
};
use crate::domains::communications::attachment_search::{
    AttachmentSearchError, AttachmentSearchPage, AttachmentSearchQuery, AttachmentSearchResult,
    AttachmentSearchStore,
};
use crate::domains::communications::bulk_actions::{
    BulkMessageAction, BulkMessageActionError, BulkMessageActionOutcome, BulkMessageActionStore,
};
use crate::domains::communications::drafts::{
    CommunicationDraft, CommunicationDraftError, CommunicationDraftStore, DraftStatus,
};
use crate::domains::communications::folders::{
    CommunicationFolder, CommunicationFolderError, CommunicationFolderListQuery,
    CommunicationFolderStore, FolderMessage, FolderMessageActionResponse, FolderMessageListQuery,
    NewCommunicationFolder, UpdateCommunicationFolder,
};
use crate::domains::communications::messages::{
    LocalMessageState, MessageProjectionError, MessageProjectionStore, MessageSearchMatchMode,
    ProjectedMessage, ProjectedMessagePageQuery, ProjectedMessageSummary, WorkflowState,
};
use crate::domains::communications::outbox::{
    CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStatus,
    CommunicationOutboxStore,
};
use crate::domains::communications::personas::{
    CommunicationPersona, CommunicationPersonaError, CommunicationPersonaStore,
};
use crate::domains::communications::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchError, CommunicationSavedSearchListQuery,
    CommunicationSavedSearchStore, NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use crate::domains::communications::service::{
    CommunicationCommandService, CommunicationDraftUpsertCommand,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::communications::subscriptions::{
    SubscriptionError, SubscriptionSource, SubscriptionStore,
};
use crate::domains::communications::templates::{
    CommunicationMergePreview, CommunicationMergePreviewItem, CommunicationMergePreviewRow,
    CommunicationTemplate, CommunicationTemplateError, CommunicationTemplateStore,
    NewCommunicationTemplate, RenderedTemplate,
};
use crate::domains::communications::threads::{
    CommunicationThread, CommunicationThreadError, CommunicationThreadStore, ThreadMessage,
    ThreadMessageAttachment,
};
use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::omniroute::client::{
    OmniRouteClient, OmniRouteClientConfig, OmniRouteError,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};
use crate::platform::config::{AiRuntimeProvider, AppConfig};
use crate::platform::settings::ApplicationSettingsStore;

const AI_REQUEST_RUNTIME: &str = "ai_request_runtime";
const ATTACHMENT_TRANSLATION_SOURCE: &str = "caller_provided_extracted_text";
const MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS: usize = 64_000;
const MAX_TEXT_PREVIEW_BYTES: usize = 64 * 1024;
const MAX_IMAGE_PREVIEW_BYTES: usize = 5 * 1024 * 1024;

pub(crate) fn register(
    router: ConnectRouter,
    pool: Option<PgPool>,
    config: AppConfig,
) -> ConnectRouter {
    let Some(pool) = pool else {
        return router;
    };

    Arc::new(CommunicationsConnectService::new(pool, config)).register(router)
}

struct CommunicationsConnectService {
    config: AppConfig,
    pool: PgPool,
    message_store: MessageProjectionStore,
    analytics_store: EmailAnalyticsStore,
    subscription_store: SubscriptionStore,
    persona_store: CommunicationPersonaStore,
    template_store: CommunicationTemplateStore,
    thread_store: CommunicationThreadStore,
    draft_store: CommunicationDraftStore,
    outbox_store: CommunicationOutboxStore,
    storage_store: CommunicationStorageStore,
    attachment_search_store: AttachmentSearchStore,
    saved_search_store: CommunicationSavedSearchStore,
    folder_store: CommunicationFolderStore,
    audit_log: ApiAuditLog,
}

impl CommunicationsConnectService {
    fn new(pool: PgPool, config: AppConfig) -> Self {
        Self {
            config,
            pool: pool.clone(),
            message_store: MessageProjectionStore::new(pool.clone()),
            analytics_store: EmailAnalyticsStore::new(pool.clone()),
            subscription_store: SubscriptionStore::new(pool.clone()),
            persona_store: CommunicationPersonaStore::new(pool.clone()),
            template_store: CommunicationTemplateStore::new(pool.clone()),
            thread_store: CommunicationThreadStore::new(pool.clone()),
            draft_store: CommunicationDraftStore::new(pool.clone()),
            outbox_store: CommunicationOutboxStore::new(pool.clone()),
            storage_store: CommunicationStorageStore::new(pool.clone()),
            attachment_search_store: AttachmentSearchStore::new(pool.clone()),
            saved_search_store: CommunicationSavedSearchStore::new(pool.clone()),
            folder_store: CommunicationFolderStore::new(pool.clone()),
            audit_log: ApiAuditLog::new(pool),
        }
    }
}

#[allow(refining_impl_trait)]
impl CommunicationsService for CommunicationsConnectService {
    async fn list_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListMessagesRequest>,
    ) -> ServiceResult<ListMessagesResponse> {
        let req = req.to_owned_message();
        let workflow_state = req
            .workflow_state
            .as_deref()
            .map(parse_workflow_state)
            .transpose()?;
        let local_state = req
            .local_state
            .as_deref()
            .map(parse_local_state)
            .transpose()?
            .unwrap_or(LocalMessageState::Active);
        let match_mode = parse_match_mode(req.match_mode.as_deref())?;
        let limit = normalize_limit(req.limit, 5000, 5000);
        let page = self
            .message_store
            .list_messages_page(ProjectedMessagePageQuery {
                account_id: req.account_id.as_deref(),
                workflow_state,
                channel_kind: req.channel_kind.as_deref(),
                conversation_id: req.conversation_id.as_deref(),
                query: req.query.as_deref(),
                match_mode,
                search: Default::default(),
                local_state,
                cursor: req.cursor.as_deref(),
                limit,
            })
            .await
            .map_err(message_connect_error)?;

        Response::ok(ListMessagesResponse {
            items: page.items.into_iter().map(proto_message_summary).collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn get_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageRequest>,
    ) -> ServiceResult<GetMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let Some(message) = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
        else {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "communication message was not found",
            ));
        };
        let attachments = self
            .storage_store
            .attachments_for_message(&req.message_id)
            .await
            .map_err(storage_connect_error)?;

        Response::ok(GetMessageResponse {
            item: Some(proto_message(message, attachments.len() as i64)).into(),
            attachments: attachments
                .into_iter()
                .map(proto_attachment_from_storage)
                .collect(),
            ..Default::default()
        })
    }

    async fn transition_message_workflow_state(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TransitionMessageWorkflowStateRequest>,
    ) -> ServiceResult<TransitionMessageWorkflowStateResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let new_state = parse_workflow_state(&req.workflow_state)?;
        let actor_id = "hermes-frontend";

        self.audit_log
            .record(&NewApiAuditRecord::message_workflow_state_set(
                actor_id,
                &req.message_id,
            ))
            .await
            .map_err(audit_connect_error)?;

        let result = CommunicationCommandService::new(self.pool.clone())
            .transition_message_workflow_state(&req.message_id, new_state, actor_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(TransitionMessageWorkflowStateResponse {
            message_id: result.updated.message_id,
            workflow_state: result.updated.workflow_state.as_str().to_owned(),
            previous_state: result.previous_state,
            ..Default::default()
        })
    }

    async fn trash_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLocalStateRequest>,
    ) -> ServiceResult<UpdateMessageLocalStateResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .move_message_to_local_trash(&req.message_id, "message_trash", "user_deleted")
            .await
            .map_err(command_connect_error)?;

        Response::ok(UpdateMessageLocalStateResponse {
            message_id: updated.message_id,
            local_state: updated.local_state.as_str().to_owned(),
            provider_deleted: Some(false),
            ..Default::default()
        })
    }

    async fn restore_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLocalStateRequest>,
    ) -> ServiceResult<UpdateMessageLocalStateResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .restore_message_from_local_trash(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(UpdateMessageLocalStateResponse {
            message_id: updated.message_id,
            local_state: updated.local_state.as_str().to_owned(),
            provider_deleted: None,
            ..Default::default()
        })
    }

    async fn mark_message_read(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MarkMessageReadRequest>,
    ) -> ServiceResult<MarkMessageReadResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .mark_message_imap_read(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(MarkMessageReadResponse {
            message_id: updated.message_id,
            marked_read: true,
            workflow_state: updated.workflow_state.as_str().to_owned(),
            ..Default::default()
        })
    }

    async fn delete_message_from_provider(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteMessageFromProviderRequest>,
    ) -> ServiceResult<DeleteMessageFromProviderResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .move_message_to_local_trash(&req.message_id, "imap_delete_alias", "imap-delete-alias")
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteMessageFromProviderResponse {
            message_id: updated.message_id,
            deleted: true,
            local_state: updated.local_state.as_str().to_owned(),
            provider_deleted: Some(false),
            ..Default::default()
        })
    }

    async fn bulk_message_action(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoBulkMessageActionRequest>,
    ) -> ServiceResult<ProtoBulkMessageActionResponse> {
        let req = req.to_owned_message();
        let action = parse_bulk_action_request(&req)?;
        let outcome = BulkMessageActionStore::new(self.pool.clone())
            .apply(req.message_ids, action)
            .await
            .map_err(bulk_action_connect_error)?;

        Response::ok(proto_bulk_message_action_outcome(outcome))
    }

    async fn toggle_message_pin(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MessageToggleRequest>,
    ) -> ServiceResult<ToggleMessagePinResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let pinned = CommunicationCommandService::new(self.pool.clone())
            .toggle_message_pin(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(ToggleMessagePinResponse {
            message_id: req.message_id,
            pinned,
            ..Default::default()
        })
    }

    async fn toggle_message_important(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MessageToggleRequest>,
    ) -> ServiceResult<ToggleMessageImportantResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let important = CommunicationCommandService::new(self.pool.clone())
            .toggle_message_important(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(ToggleMessageImportantResponse {
            message_id: req.message_id,
            important,
            ..Default::default()
        })
    }

    async fn toggle_message_mute(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MessageToggleRequest>,
    ) -> ServiceResult<ToggleMessageMuteResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let muted = CommunicationCommandService::new(self.pool.clone())
            .toggle_message_mute(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(ToggleMessageMuteResponse {
            message_id: req.message_id,
            muted,
            ..Default::default()
        })
    }

    async fn snooze_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, SnoozeMessageRequest>,
    ) -> ServiceResult<SnoozeMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let until = parse_timestamp(&req.until)?;
        CommunicationCommandService::new(self.pool.clone())
            .snooze_message(&req.message_id, until)
            .await
            .map_err(command_connect_error)?;

        Response::ok(SnoozeMessageResponse {
            snoozed: true,
            ..Default::default()
        })
    }

    async fn add_message_label(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLabelRequest>,
    ) -> ServiceResult<AddMessageLabelResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if req.label.trim().is_empty() {
            return Err(invalid_argument_error("label must not be empty"));
        }
        CommunicationCommandService::new(self.pool.clone())
            .add_message_label(&req.message_id, &req.label)
            .await
            .map_err(command_connect_error)?;

        Response::ok(AddMessageLabelResponse {
            labeled: true,
            ..Default::default()
        })
    }

    async fn remove_message_label(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLabelRequest>,
    ) -> ServiceResult<RemoveMessageLabelResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if req.label.trim().is_empty() {
            return Err(invalid_argument_error("label must not be empty"));
        }
        CommunicationCommandService::new(self.pool.clone())
            .remove_message_label(&req.message_id, &req.label)
            .await
            .map_err(command_connect_error)?;

        Response::ok(RemoveMessageLabelResponse {
            removed: true,
            ..Default::default()
        })
    }

    async fn analyze_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, AnalyzeMessageRequest>,
    ) -> ServiceResult<AnalyzeMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }

        let ai_state_store =
            crate::domains::communications::ai_state::CommunicationAiStateStore::new(
                self.pool.clone(),
            );
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;

        let _ = ai_state_store
            .transition(
                &req.message_id,
                crate::domains::communications::ai_state::CommunicationAiStateTransitionRequest {
                    ai_state:
                        crate::domains::communications::ai_state::CommunicationAiState::Processing,
                    review_reason: None,
                    last_error: None,
                },
            )
            .await
            .map_err(ai_state_connect_error)?;

        let heuristic_score =
            crate::workflows::email_intelligence::EmailIntelligenceService::heuristic_score(
                &message,
            );
        let heuristic_category =
            crate::workflows::email_intelligence::EmailIntelligenceService::heuristic_category(
                &message,
            );
        let summary_contract =
            crate::workflows::email_intelligence::EmailIntelligenceService::heuristic_structured_summary(&message);

        self.message_store
            .set_ai_analysis(
                &req.message_id,
                heuristic_category.as_deref(),
                None,
                Some(heuristic_score),
            )
            .await
            .map_err(message_connect_error)?;
        let mut metadata = message.message_metadata.clone();
        metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract)
            .map_err(|_| invalid_argument_error("summary contract serialization failed"))?;
        self.message_store
            .set_message_metadata(&req.message_id, &metadata)
            .await
            .map_err(message_connect_error)?;

        if heuristic_score >= 75 && message.workflow_state.as_str() == "new" {
            let _ = self
                .message_store
                .transition_workflow_state(&req.message_id, WorkflowState::NeedsAction)
                .await;
        }

        let _ = ai_state_store
            .transition(
                &req.message_id,
                crate::domains::communications::ai_state::CommunicationAiStateTransitionRequest {
                    ai_state:
                        crate::domains::communications::ai_state::CommunicationAiState::Processed,
                    review_reason: None,
                    last_error: None,
                },
            )
            .await
            .map_err(ai_state_connect_error)?;

        let updated = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let _ = crate::application::refresh_message_knowledge_candidates_into_review(
            &self.pool,
            std::slice::from_ref(&updated),
        )
        .await
        .map_err(|_| invalid_argument_error("message knowledge candidate review sync failed"))?;
        let evidence =
            crate::domains::communications::explain::explain_importance(&updated).reasons;

        Response::ok(AnalyzeMessageResponse {
            message_id: updated.message_id,
            analyzed: true,
            category: updated.ai_category,
            summary: updated.ai_summary,
            summary_contract: Some(proto_message_summary_contract(summary_contract)).into(),
            importance_score: updated.importance_score.map(i32::from),
            workflow_state: updated.workflow_state.as_str().to_owned(),
            source: "local_heuristic".to_owned(),
            confidence: None,
            evidence,
            ..Default::default()
        })
    }

    async fn run_workflow_action(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoWorkflowActionRequest>,
    ) -> ServiceResult<ProtoWorkflowActionResponse> {
        let req = req.to_owned_message();
        let request = proto_workflow_action_request(req)?;
        let response = execute_workflow_action(&self.pool, "hermes-frontend", request)
            .await
            .map_err(api_error_connect_error)?;
        Response::ok(proto_workflow_action_response(response))
    }

    async fn get_message_explain(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExplainMessageRequest>,
    ) -> ServiceResult<ExplainMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let context = crate::domains::communications::explain::explain_importance(&message);

        Response::ok(ExplainMessageResponse {
            reasons: context.reasons,
            ..Default::default()
        })
    }

    async fn get_message_smart_cc(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageSmartCcRequest>,
    ) -> ServiceResult<GetMessageSmartCcResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let suggestions = crate::domains::communications::explain::smart_cc_suggestions(&message);

        Response::ok(GetMessageSmartCcResponse {
            suggestions,
            ..Default::default()
        })
    }

    async fn get_message_export(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageExportRequest>,
    ) -> ServiceResult<GetMessageExportResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let format = match req.format.trim() {
            "eml" => crate::domains::communications::export::ExportFormat::Eml,
            "json" => crate::domains::communications::export::ExportFormat::Json,
            "" | "md" | "markdown" => {
                crate::domains::communications::export::ExportFormat::Markdown
            }
            _ => return Err(invalid_argument_error("invalid export format")),
        };
        let export = crate::domains::communications::export::export_message(
            &self.message_store,
            &self.storage_store,
            &req.message_id,
            format,
        )
        .await
        .map_err(export_connect_error)?;

        Response::ok(GetMessageExportResponse {
            content_type: export.format.content_type().to_owned(),
            content: export.content,
            filename: format!(
                "message_{}.{}",
                &req.message_id[..8.min(req.message_id.len())],
                export.format.extension()
            ),
            ..Default::default()
        })
    }

    async fn get_message_auth(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageAuthRequest>,
    ) -> ServiceResult<GetMessageAuthResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let auth = crate::domains::communications::spf_dkim::parse_auth_headers(&message.body_text);
        let risk = crate::domains::communications::spf_dkim::assess_auth_risk(&auth);

        Response::ok(GetMessageAuthResponse {
            auth: Some(proto_message_auth_report(auth)).into(),
            risk: Some(proto_message_auth_risk_report(risk)).into(),
            ..Default::default()
        })
    }

    async fn get_message_signature(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageSignatureRequest>,
    ) -> ServiceResult<GetMessageSignatureResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let detection =
            crate::domains::communications::signatures::SignatureDetector::detect_in_message(
                &message.body_text,
                "",
            );

        Response::ok(GetMessageSignatureResponse {
            has_signature: detection.has_signature,
            signature_type: detection
                .signature_type
                .map(|value| value.as_str().to_owned()),
            signer_info: detection.signer_info,
            is_valid: detection.is_valid,
            cert_expiry_warning: detection.cert_expiry_warning,
            ..Default::default()
        })
    }

    async fn generate_ai_reply(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoAiReplyRequest>,
    ) -> ServiceResult<ProtoAiReplyResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let runtime = thread_ai_runtime_port_optional(&self.pool, &self.config)
            .await
            .map_err(api_error_connect_error)?;
        let service = crate::domains::communications::ai_reply::AiReplyService::new(runtime);
        let opts = crate::domains::communications::ai_reply::AiReplyOptions {
            tone: req.tone,
            language: req.language,
            context: req.context,
        };

        match service.generate_reply(&message, &opts).await {
            Ok(Some(draft)) => {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    self.pool.clone(),
                    "reply_drafting",
                    &req.message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": req.message_id,
                        "operation": "reply_drafting",
                    }),
                    serde_json::json!({
                        "tone": draft.tone,
                        "language": draft.language,
                    }),
                    serde_json::json!({
                        "source": "communication_message_ai_reply",
                        "message_id": req.message_id,
                    }),
                    None,
                )
                .await
                .map_err(signal_hub_connect_error)?;

                Response::ok(ProtoAiReplyResponse {
                    subject: Some(draft.subject),
                    body: Some(draft.body),
                    tone: Some(draft.tone),
                    language: Some(draft.language),
                    generated: Some(true),
                    reason: None,
                    ..Default::default()
                })
            }
            Ok(None) => Response::ok(ProtoAiReplyResponse {
                generated: Some(false),
                reason: Some("no LLM configured".to_owned()),
                ..Default::default()
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %req.message_id,
                    "ai reply generation failed"
                );
                Response::ok(ProtoAiReplyResponse {
                    generated: Some(false),
                    reason: Some("ai reply runtime unavailable".to_owned()),
                    ..Default::default()
                })
            }
        }
    }

    async fn generate_ai_reply_variants(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoAiReplyVariantsRequest>,
    ) -> ServiceResult<ProtoAiReplyVariantsResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let runtime = thread_ai_runtime_port_optional(&self.pool, &self.config)
            .await
            .map_err(api_error_connect_error)?;
        let service = crate::domains::communications::ai_reply::AiReplyService::new(runtime);
        let languages = if req.languages.is_empty() {
            vec!["en".to_owned(), "es".to_owned(), "ru".to_owned()]
        } else {
            req.languages
        };
        let tones = if req.tones.is_empty() {
            vec!["professional".to_owned(), "friendly".to_owned()]
        } else {
            req.tones
        };
        let variants = match service
            .generate_reply_variants(&message, &languages, &tones)
            .await
        {
            Ok(variants) => variants,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %req.message_id,
                    "ai reply variants generation failed"
                );
                Vec::new()
            }
        };

        if !variants.is_empty() {
            crate::domains::signal_hub::dispatch_ai_helper_signal(
                self.pool.clone(),
                "reply_variant_generation",
                &req.message_id,
                serde_json::json!({
                    "kind": "communication_message",
                    "source_code": "ai",
                    "message_id": req.message_id,
                    "operation": "reply_variant_generation",
                }),
                serde_json::json!({
                    "variant_count": variants.len(),
                    "language_count": languages.len(),
                    "tone_count": tones.len(),
                }),
                serde_json::json!({
                    "source": "communication_message_ai_reply_variants",
                    "message_id": req.message_id,
                }),
                None,
            )
            .await
            .map_err(signal_hub_connect_error)?;
        }

        Response::ok(ProtoAiReplyVariantsResponse {
            variants: variants
                .into_iter()
                .map(|draft| ProtoAiReplyResponse {
                    subject: Some(draft.subject),
                    body: Some(draft.body),
                    tone: Some(draft.tone),
                    language: Some(draft.language),
                    generated: Some(true),
                    reason: None,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn list_message_workflow_state_counts(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListMessageWorkflowStateCountsRequest>,
    ) -> ServiceResult<ListMessageWorkflowStateCountsResponse> {
        let req = req.to_owned_message();
        let local_state = req
            .local_state
            .as_deref()
            .unwrap_or("active")
            .parse::<LocalMessageState>()
            .map_err(|_| invalid_argument_error("invalid local_state value"))?;
        let counts = self
            .message_store
            .count_messages_by_state_with_local_state(req.account_id.as_deref(), local_state)
            .await
            .map_err(message_connect_error)?;

        Response::ok(ListMessageWorkflowStateCountsResponse {
            counts: counts.into_iter().map(proto_workflow_state_count).collect(),
            ..Default::default()
        })
    }

    async fn list_subscriptions(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListSubscriptionsRequest>,
    ) -> ServiceResult<ListSubscriptionsResponse> {
        let req = req.to_owned_message();
        let page = self
            .subscription_store
            .detect_subscriptions_page(
                req.account_id.as_deref(),
                normalize_limit(req.limit, 50, 100),
                req.cursor.as_deref(),
            )
            .await
            .map_err(subscription_connect_error)?;

        Response::ok(ListSubscriptionsResponse {
            items: page
                .items
                .into_iter()
                .map(proto_subscription_source)
                .collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn get_mailbox_health(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMailboxHealthRequest>,
    ) -> ServiceResult<GetMailboxHealthResponse> {
        let req = req.to_owned_message();
        let item = self
            .analytics_store
            .mailbox_health(req.account_id.as_deref())
            .await
            .map_err(analytics_connect_error)?;

        Response::ok(GetMailboxHealthResponse {
            item: Some(proto_mailbox_health(item)).into(),
            ..Default::default()
        })
    }

    async fn list_top_senders(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListTopSendersRequest>,
    ) -> ServiceResult<ListTopSendersResponse> {
        let req = req.to_owned_message();
        let page = self
            .analytics_store
            .top_senders_page(
                req.account_id.as_deref(),
                normalize_limit(req.limit, 20, 50),
                req.cursor.as_deref(),
            )
            .await
            .map_err(analytics_connect_error)?;

        Response::ok(ListTopSendersResponse {
            items: page.items.into_iter().map(proto_sender_stats).collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn list_communication_blockers(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListCommunicationBlockersRequest>,
    ) -> ServiceResult<ListCommunicationBlockersResponse> {
        Response::ok(ListCommunicationBlockersResponse {
            items: crate::domains::communications::blockers::list_blockers()
                .into_iter()
                .map(proto_communication_architecture_blocker)
                .collect(),
            ..Default::default()
        })
    }

    async fn list_communication_personas(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListCommunicationPersonasRequest>,
    ) -> ServiceResult<ListCommunicationPersonasResponse> {
        let items = self
            .persona_store
            .list()
            .await
            .map_err(persona_connect_error)?;
        Response::ok(ListCommunicationPersonasResponse {
            items: items.into_iter().map(proto_communication_persona).collect(),
            ..Default::default()
        })
    }

    async fn list_rich_templates(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListRichTemplatesRequest>,
    ) -> ServiceResult<ListRichTemplatesResponse> {
        let templates = self
            .template_store
            .list()
            .await
            .map_err(template_connect_error)?;
        Response::ok(ListRichTemplatesResponse {
            templates: templates.into_iter().map(proto_rich_template).collect(),
            ..Default::default()
        })
    }

    async fn upsert_rich_template(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpsertRichTemplateRequest>,
    ) -> ServiceResult<UpsertRichTemplateResponse> {
        let req = req.to_owned_message();
        let template = self
            .template_store
            .upsert(&NewCommunicationTemplate {
                template_id: req
                    .template_id
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| {
                        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
                        format!("mail_template:{timestamp}")
                    }),
                name: req.name,
                subject_template: req.subject_template,
                body_template: req.body_template,
                variables: req.variables,
                language: req.language,
            })
            .await
            .map_err(template_connect_error)?;
        Response::ok(UpsertRichTemplateResponse {
            saved: true,
            template: Some(proto_rich_template(template)).into(),
            ..Default::default()
        })
    }

    async fn delete_rich_template(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteRichTemplateRequest>,
    ) -> ServiceResult<DeleteRichTemplateResponse> {
        let req = req.to_owned_message();
        let template_id = req.template_id.trim();
        if template_id.is_empty() {
            return Err(invalid_argument_error("template_id is required"));
        }
        let deleted = self
            .template_store
            .delete(template_id)
            .await
            .map_err(template_connect_error)?;
        if !deleted {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "rich template was not found",
            ));
        }
        Response::ok(DeleteRichTemplateResponse {
            template_id: template_id.to_owned(),
            deleted: true,
            ..Default::default()
        })
    }

    async fn render_rich_template(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RichTemplateRenderRequest>,
    ) -> ServiceResult<RichTemplateRenderResponse> {
        let req = req.to_owned_message();
        let template_id = req.template_id.trim();
        if template_id.is_empty() {
            return Err(invalid_argument_error("template_id is required"));
        }
        let Some(template) = self
            .template_store
            .get(template_id)
            .await
            .map_err(template_connect_error)?
        else {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "rich template was not found",
            ));
        };
        let rendered = self
            .template_store
            .render(&template, &req.variables)
            .map_err(template_connect_error)?;
        Response::ok(RichTemplateRenderResponse {
            template_id: template.template_id,
            variables: req.variables,
            rendered: Some(proto_rendered_rich_template(rendered)).into(),
            ..Default::default()
        })
    }

    async fn preview_rich_template_mail_merge(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RichTemplateMailMergePreviewRequest>,
    ) -> ServiceResult<RichTemplateMailMergePreviewResponse> {
        let req = req.to_owned_message();
        let template_id = req.template_id.trim();
        if template_id.is_empty() {
            return Err(invalid_argument_error("template_id is required"));
        }
        if req.rows.is_empty() {
            return Err(invalid_argument_error(
                "mail merge preview rows are required",
            ));
        }
        let Some(template) = self
            .template_store
            .get(template_id)
            .await
            .map_err(template_connect_error)?
        else {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "rich template was not found",
            ));
        };
        let rows = req
            .rows
            .into_iter()
            .map(|row| {
                let row_id = row.row_id.trim().to_owned();
                if row_id.is_empty() {
                    return Err(invalid_argument_error("row_id is required"));
                }
                Ok(CommunicationMergePreviewRow {
                    row_id,
                    variables: row.variables,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let preview = self
            .template_store
            .render_mail_merge_preview(&template, rows)
            .map_err(template_connect_error)?;
        Response::ok(proto_rich_template_mail_merge_preview(preview))
    }

    async fn search_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, SearchMessagesRequest>,
    ) -> ServiceResult<SearchMessagesResponse> {
        let req = req.to_owned_message();
        if req.query.trim().is_empty() {
            return Err(invalid_argument_error("query must not be empty"));
        }

        let Some(path) = std::env::var("HERMES_SEARCH_INDEX_PATH").ok() else {
            return Response::ok(SearchMessagesResponse {
                results: Vec::new(),
                ..Default::default()
            });
        };

        let index =
            crate::engines::search::SearchIndex::open_or_create(std::path::Path::new(&path))
                .map_err(search_engine_connect_error)?;
        let limit = normalize_limit(req.limit, 20, 100) as usize;
        let _ = crate::domains::communications::search::index_messages(
            &index,
            &self.message_store,
            100,
        )
        .await
        .map_err(search_connect_error)?;
        let results =
            crate::domains::communications::search::search_emails(&index, &req.query, limit)
                .map_err(search_connect_error)?;

        Response::ok(SearchMessagesResponse {
            results: results.into_iter().map(proto_search_result).collect(),
            ..Default::default()
        })
    }

    async fn detect_message_language(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DetectMessageLanguageRequest>,
    ) -> ServiceResult<DetectMessageLanguageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );

        Response::ok(DetectMessageLanguageResponse {
            language: detection.language,
            confidence: detection.confidence,
            script: detection.script,
            ..Default::default()
        })
    }

    async fn translate_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TranslateMessageRequest>,
    ) -> ServiceResult<TranslateMessageResponse> {
        let req = req.to_owned_message();
        let message_id = req.message_id.trim();
        let target_language = req.target_language.trim();
        if message_id.is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if target_language.is_empty() {
            return Err(invalid_argument_error("target_language is required"));
        }

        let message = self
            .message_store
            .message(message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let service = thread_multilingual_service(&self.pool, &self.config)
            .await
            .map_err(api_error_connect_error)?;
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );

        match service.translate(&message.body_text, target_language).await {
            Ok(Some(translation)) => {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    self.pool.clone(),
                    "message_translation",
                    message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": message_id,
                        "operation": "translation",
                    }),
                    serde_json::json!({
                        "target_language": translation.target_language,
                        "original_language": detection.language,
                        "model": translation.model,
                    }),
                    serde_json::json!({
                        "source": "communication_message_translation",
                        "message_id": message_id,
                    }),
                    None,
                )
                .await
                .map_err(signal_hub_connect_error)?;

                Response::ok(TranslateMessageResponse {
                    translated: true,
                    text: Some(translation.translated_text),
                    target: Some(translation.target_language),
                    model: Some(translation.model),
                    reason: None,
                    ..Default::default()
                })
            }
            Ok(None) => Response::ok(TranslateMessageResponse {
                translated: false,
                text: None,
                target: Some(target_language.to_owned()),
                model: None,
                reason: Some("no LLM configured".to_owned()),
                ..Default::default()
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %message.message_id,
                    "message translation failed"
                );
                Response::ok(TranslateMessageResponse {
                    translated: false,
                    text: None,
                    target: Some(target_language.to_owned()),
                    model: None,
                    reason: Some("translation runtime unavailable".to_owned()),
                    ..Default::default()
                })
            }
        }
    }

    async fn extract_message_tasks(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExtractMessageTasksRequest>,
    ) -> ServiceResult<ExtractMessageTasksResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let service = crate::domains::communications::extract::EmailExtractService::new(
            thread_ai_runtime_port_optional(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?,
        );
        let tasks = service
            .extract_tasks(&message)
            .await
            .map_err(extract_connect_error)?;
        let llm_task_count = tasks.iter().filter(|task| task.source == "llm").count();
        if llm_task_count > 0 {
            crate::domains::signal_hub::dispatch_ai_helper_signal(
                self.pool.clone(),
                "message_task_extraction",
                &req.message_id,
                serde_json::json!({
                    "kind": "communication_message",
                    "source_code": "ai",
                    "message_id": req.message_id,
                    "operation": "task_extraction",
                }),
                serde_json::json!({
                    "task_count": tasks.len(),
                    "llm_task_count": llm_task_count,
                }),
                serde_json::json!({
                    "source": "communication_message_task_extraction",
                    "message_id": req.message_id,
                }),
                None,
            )
            .await
            .map_err(signal_hub_connect_error)?;
        }

        Response::ok(ExtractMessageTasksResponse {
            tasks: tasks.into_iter().map(proto_extracted_task).collect(),
            ..Default::default()
        })
    }

    async fn extract_message_notes(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExtractMessageNotesRequest>,
    ) -> ServiceResult<ExtractMessageNotesResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let service = crate::domains::communications::extract::EmailExtractService::new(None);
        let notes = service
            .extract_notes(&message)
            .await
            .map_err(extract_connect_error)?;

        Response::ok(ExtractMessageNotesResponse {
            notes: notes.into_iter().map(proto_extracted_note).collect(),
            ..Default::default()
        })
    }

    async fn list_threads(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListThreadsRequest>,
    ) -> ServiceResult<ListThreadsResponse> {
        let req = req.to_owned_message();
        let page = self
            .thread_store
            .list_threads_page(
                req.account_id.as_deref(),
                req.cursor.as_deref(),
                normalize_limit(req.limit, 50, 100),
            )
            .await
            .map_err(thread_connect_error)?;
        Response::ok(ListThreadsResponse {
            items: page.items.into_iter().map(proto_thread).collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn list_thread_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListThreadMessagesRequest>,
    ) -> ServiceResult<ListThreadMessagesResponse> {
        let req = req.to_owned_message();
        let account_id = req.account_id.as_deref().ok_or_else(|| {
            invalid_argument_error("account_id is required for thread message lookup")
        })?;
        if req.subject.trim().is_empty() {
            return Err(invalid_argument_error("subject must not be empty"));
        }
        let items = self
            .thread_store
            .thread_messages(
                account_id,
                &req.subject,
                normalize_limit(req.limit, 50, 100),
            )
            .await
            .map_err(thread_connect_error)?;
        Response::ok(ListThreadMessagesResponse {
            items: items.into_iter().map(proto_thread_message).collect(),
            ..Default::default()
        })
    }

    async fn translate_thread(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TranslateThreadRequest>,
    ) -> ServiceResult<TranslateThreadResponse> {
        let req = req.to_owned_message();
        let account_id = req.account_id.trim();
        let subject = req.subject.trim();
        let target_language = req.target_language.trim();
        if account_id.is_empty() {
            return Err(invalid_argument_error("account_id is required"));
        }
        if subject.is_empty() {
            return Err(invalid_argument_error("subject is required"));
        }
        if target_language.is_empty() {
            return Err(invalid_argument_error("target_language is required"));
        }

        let messages = self
            .thread_store
            .thread_messages(account_id, subject, normalize_limit(req.limit, 50, 100))
            .await
            .map_err(thread_connect_error)?;
        let service = thread_multilingual_service(&self.pool, &self.config)
            .await
            .map_err(api_error_connect_error)?;
        let mut items = Vec::with_capacity(messages.len());

        for message in messages {
            let detection =
                crate::domains::communications::multilingual::MultilingualService::detect_language(
                    &message.body_text,
                );
            match service.translate(&message.body_text, target_language).await {
                Ok(Some(translation)) => {
                    crate::domains::signal_hub::dispatch_ai_helper_signal(
                        self.pool.clone(),
                        "thread_message_translation",
                        &message.message_id,
                        serde_json::json!({
                            "kind": "communication_message",
                            "source_code": "ai",
                            "message_id": message.message_id,
                            "operation": "thread_message_translation",
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        serde_json::json!({
                            "target_language": translation.target_language,
                            "original_language": detection.language,
                            "model": translation.model,
                        }),
                        serde_json::json!({
                            "source": "communication_thread_message_translation",
                            "message_id": message.message_id,
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        None,
                    )
                    .await
                    .map_err(signal_hub_connect_error)?;

                    items.push(ProtoThreadTranslationItem {
                        message_id: message.message_id,
                        original_language: detection.language,
                        confidence: detection.confidence,
                        translated: true,
                        text: Some(translation.translated_text),
                        target: translation.target_language,
                        model: Some(translation.model),
                        reason: None,
                        ..Default::default()
                    });
                }
                Ok(None) => items.push(ProtoThreadTranslationItem {
                    message_id: message.message_id,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: false,
                    text: None,
                    target: target_language.to_owned(),
                    model: None,
                    reason: Some("no LLM configured".to_owned()),
                    ..Default::default()
                }),
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        message_id = %message.message_id,
                        "thread message translation failed"
                    );
                    items.push(ProtoThreadTranslationItem {
                        message_id: message.message_id,
                        original_language: detection.language,
                        confidence: detection.confidence,
                        translated: false,
                        text: None,
                        target: target_language.to_owned(),
                        model: None,
                        reason: Some("translation runtime unavailable".to_owned()),
                        ..Default::default()
                    });
                }
            }
        }

        Response::ok(TranslateThreadResponse {
            account_id: account_id.to_owned(),
            subject: subject.to_owned(),
            target_language: target_language.to_owned(),
            items,
            ..Default::default()
        })
    }

    async fn list_drafts(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListDraftsRequest>,
    ) -> ServiceResult<ListDraftsResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let status = req.status.as_deref().map(parse_draft_status).transpose()?;
        let page = self
            .draft_store
            .list_page(
                req.account_id.as_deref(),
                status,
                page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                normalize_limit(page_request.map(|page| page.limit).unwrap_or(100), 100, 100),
            )
            .await
            .map_err(draft_connect_error)?;
        Response::ok(ListDraftsResponse {
            items: page.items.into_iter().map(proto_draft).collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn list_saved_searches(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListSavedSearchesRequest>,
    ) -> ServiceResult<ListSavedSearchesResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let page = self
            .saved_search_store
            .list(CommunicationSavedSearchListQuery {
                account_id: req.account_id.as_deref(),
                is_smart_folder: req.smart_folder,
                cursor: page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                limit: normalize_limit(
                    page_request.map(|page| page.limit).unwrap_or(500),
                    500,
                    1000,
                ),
            })
            .await
            .map_err(saved_search_connect_error)?;
        Response::ok(ListSavedSearchesResponse {
            items: page.items.into_iter().map(proto_saved_search).collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn list_folders(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListFoldersRequest>,
    ) -> ServiceResult<ListFoldersResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let page = self
            .folder_store
            .list(CommunicationFolderListQuery {
                account_id: req.account_id.as_deref(),
                cursor: page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                limit: normalize_limit(
                    page_request.map(|page| page.limit).unwrap_or(500),
                    500,
                    1000,
                ),
            })
            .await
            .map_err(folder_connect_error)?;
        Response::ok(ListFoldersResponse {
            items: page.items.into_iter().map(proto_folder).collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn list_outbox(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListOutboxRequest>,
    ) -> ServiceResult<ListOutboxResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let status = req.status.as_deref().map(parse_outbox_status).transpose()?;
        let page = self
            .outbox_store
            .list_page(
                req.account_id.as_deref(),
                status,
                page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                normalize_limit(page_request.map(|page| page.limit).unwrap_or(100), 100, 100),
            )
            .await
            .map_err(outbox_connect_error)?;
        Response::ok(ListOutboxResponse {
            items: page.items.into_iter().map(proto_outbox_item).collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn create_draft(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateDraftRequest>,
    ) -> ServiceResult<CreateDraftResponse> {
        let req = req.to_owned_message();
        let metadata = parse_json_object(req.metadata_json.as_str(), "metadata_json")?;
        let scheduled_send_at = req
            .scheduled_send_at
            .as_deref()
            .map(parse_timestamp)
            .transpose()?;
        let draft = CommunicationCommandService::new(self.pool.clone())
            .upsert_draft(CommunicationDraftUpsertCommand {
                draft_id: req.draft_id,
                account_id: req.account_id,
                persona_id: req.persona_id,
                to_recipients: req.to_recipients,
                cc_recipients: Some(req.cc_recipients),
                bcc_recipients: Some(req.bcc_recipients),
                subject: req.subject,
                body_text: req.body_text,
                body_html: req.body_html,
                in_reply_to: req.in_reply_to,
                references: Some(req.references),
                status: req.status,
                scheduled_send_at,
                metadata: Some(metadata),
            })
            .await
            .map_err(command_connect_error)?;

        Response::ok(CreateDraftResponse {
            item: Some(proto_draft(draft)).into(),
            ..Default::default()
        })
    }

    async fn delete_draft(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteDraftRequest>,
    ) -> ServiceResult<DeleteDraftResponse> {
        let req = req.to_owned_message();
        if req.draft_id.trim().is_empty() {
            return Err(invalid_argument_error("draft_id must not be empty"));
        }
        let deleted = CommunicationCommandService::new(self.pool.clone())
            .delete_draft(&req.draft_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteDraftResponse {
            deleted,
            ..Default::default()
        })
    }

    async fn create_saved_search(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateSavedSearchRequest>,
    ) -> ServiceResult<CreateSavedSearchResponse> {
        let req = req.to_owned_message();
        let saved_search = CommunicationCommandService::new(self.pool.clone())
            .create_saved_search(NewCommunicationSavedSearch {
                saved_search_id: req.saved_search_id,
                name: req.name,
                description: req.description,
                account_id: req.account_id,
                query: req.query,
                workflow_state: req
                    .workflow_state
                    .as_deref()
                    .map(parse_workflow_state)
                    .transpose()?,
                local_state: req
                    .local_state
                    .as_deref()
                    .map(parse_local_state)
                    .transpose()?,
                channel_kind: req.channel_kind,
                is_smart_folder: req.is_smart_folder,
                sort_order: req.sort_order,
            })
            .await
            .map_err(command_connect_error)?;

        Response::ok(CreateSavedSearchResponse {
            item: Some(proto_saved_search(saved_search)).into(),
            ..Default::default()
        })
    }

    async fn update_saved_search(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateSavedSearchRequest>,
    ) -> ServiceResult<UpdateSavedSearchResponse> {
        let req = req.to_owned_message();
        if req.saved_search_id.trim().is_empty() {
            return Err(invalid_argument_error("saved_search_id must not be empty"));
        }
        let saved_search = CommunicationCommandService::new(self.pool.clone())
            .update_saved_search(
                &req.saved_search_id,
                UpdateCommunicationSavedSearch {
                    name: req.name,
                    description: req.description,
                    account_id: req.account_id,
                    query: req.query,
                    workflow_state: req
                        .workflow_state
                        .as_deref()
                        .map(parse_workflow_state)
                        .transpose()?,
                    local_state: req
                        .local_state
                        .as_deref()
                        .map(parse_local_state)
                        .transpose()?,
                    channel_kind: req.channel_kind,
                    is_smart_folder: req.is_smart_folder,
                    sort_order: req.sort_order,
                },
            )
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "saved search was not found"))?;

        Response::ok(UpdateSavedSearchResponse {
            item: Some(proto_saved_search(saved_search)).into(),
            ..Default::default()
        })
    }

    async fn delete_saved_search(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteSavedSearchRequest>,
    ) -> ServiceResult<DeleteSavedSearchResponse> {
        let req = req.to_owned_message();
        if req.saved_search_id.trim().is_empty() {
            return Err(invalid_argument_error("saved_search_id must not be empty"));
        }
        let deleted = CommunicationCommandService::new(self.pool.clone())
            .delete_saved_search(&req.saved_search_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteSavedSearchResponse {
            deleted,
            ..Default::default()
        })
    }

    async fn create_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateFolderRequest>,
    ) -> ServiceResult<CreateFolderResponse> {
        let req = req.to_owned_message();
        let folder = CommunicationCommandService::new(self.pool.clone())
            .create_folder(NewCommunicationFolder {
                folder_id: req.folder_id,
                account_id: req.account_id,
                name: req.name,
                description: req.description,
                color: req.color,
                sort_order: req.sort_order,
            })
            .await
            .map_err(command_connect_error)?;

        Response::ok(CreateFolderResponse {
            item: Some(proto_folder(folder)).into(),
            ..Default::default()
        })
    }

    async fn update_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateFolderRequest>,
    ) -> ServiceResult<UpdateFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        let folder = CommunicationCommandService::new(self.pool.clone())
            .update_folder(
                &req.folder_id,
                UpdateCommunicationFolder {
                    account_id: req.account_id,
                    name: req.name,
                    description: req.description,
                    color: req.color,
                    sort_order: req.sort_order,
                },
            )
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "folder was not found"))?;

        Response::ok(UpdateFolderResponse {
            item: Some(proto_folder(folder)).into(),
            ..Default::default()
        })
    }

    async fn delete_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteFolderRequest>,
    ) -> ServiceResult<DeleteFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        let deleted = CommunicationCommandService::new(self.pool.clone())
            .delete_folder(&req.folder_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteFolderResponse {
            deleted,
            ..Default::default()
        })
    }

    async fn list_folder_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListFolderMessagesRequest>,
    ) -> ServiceResult<ListFolderMessagesResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        let page_request = req.page.as_option();
        let page = self
            .folder_store
            .list_messages(FolderMessageListQuery {
                folder_id: &req.folder_id,
                cursor: page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                limit: normalize_limit(
                    page_request.map(|page| page.limit).unwrap_or(250),
                    250,
                    1000,
                ),
            })
            .await
            .map_err(folder_connect_error)?;
        Response::ok(ListFolderMessagesResponse {
            items: page.items.into_iter().map(proto_folder_message).collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn copy_message_to_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CopyMessageToFolderRequest>,
    ) -> ServiceResult<CopyMessageToFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let response = CommunicationCommandService::new(self.pool.clone())
            .copy_message_to_folder(&req.folder_id, &req.message_id)
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(
                    ErrorCode::NotFound,
                    "folder message copy target was not found",
                )
            })?;

        Response::ok(CopyMessageToFolderResponse {
            item: Some(proto_folder_message_action(response)).into(),
            ..Default::default()
        })
    }

    async fn move_message_to_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MoveMessageToFolderRequest>,
    ) -> ServiceResult<MoveMessageToFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let response = CommunicationCommandService::new(self.pool.clone())
            .move_message_to_folder(&req.folder_id, &req.message_id)
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(
                    ErrorCode::NotFound,
                    "folder message move target was not found",
                )
            })?;

        Response::ok(MoveMessageToFolderResponse {
            item: Some(proto_folder_message_action(response)).into(),
            ..Default::default()
        })
    }

    async fn send_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, SendMessageRequest>,
    ) -> ServiceResult<SendMessageResponse> {
        let req = req.to_owned_message();
        if !req.confirmed_provider_write {
            return Err(invalid_argument_error(
                "provider write confirmation is required",
            ));
        }
        let metadata = parse_json_object(req.metadata_json.as_str(), "metadata_json")?;
        let scheduled_send_at = req
            .scheduled_send_at
            .as_deref()
            .map(parse_timestamp)
            .transpose()?;
        let result = send_email(
            &CommunicationSendDependencies::new(self.pool.clone(), self.audit_log.clone()),
            CommunicationSendRequest {
                account_id: req.account_id,
                to: req.to_recipients,
                cc: req.cc_recipients,
                bcc: req.bcc_recipients,
                subject: req.subject,
                body_text: req.body_text,
                body_html: req.body_html,
                in_reply_to: req.in_reply_to,
                references: req.references,
                draft_id: req.draft_id,
                scheduled_send_at,
                undo_send_seconds: req.undo_send_seconds,
                metadata,
            },
        )
        .await
        .map_err(send_connect_error)?;
        let outbox_id = result.outbox_id.clone().ok_or_else(|| {
            ConnectError::new(
                ErrorCode::Internal,
                "send_email did not return an outbox item identifier",
            )
        })?;
        let item = self
            .outbox_store
            .get(&outbox_id)
            .await
            .map_err(outbox_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::Internal, "queued outbox item was not found")
            })?;

        Response::ok(SendMessageResponse {
            item: Some(proto_outbox_item(item)).into(),
            message_id: result.message_id,
            outbox_id: result.outbox_id,
            accepted: result.accepted,
            accepted_recipients: result.accepted_recipients,
            transport: result.transport,
            status: result.status,
            scheduled_send_at: result.scheduled_send_at.map(timestamp_string),
            undo_deadline_at: result.undo_deadline_at.map(timestamp_string),
            failure_reason: result.failure_reason,
            ..Default::default()
        })
    }

    async fn redirect_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RedirectMessageRequest>,
    ) -> ServiceResult<RedirectMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if !req.confirmed_provider_write {
            return Err(invalid_argument_error(
                "provider write confirmation is required",
            ));
        }

        let to = trim_non_empty_recipients(req.to_recipients);
        let cc = trim_non_empty_recipients(req.cc_recipients);
        let bcc = trim_non_empty_recipients(req.bcc_recipients);
        if to.is_empty() && cc.is_empty() && bcc.is_empty() {
            return Err(invalid_argument_error("at least one recipient is required"));
        }

        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let recipient_count = to.len() + cc.len() + bcc.len();
        let outbox = CommunicationCommandService::new(self.pool.clone())
            .enqueue_redirect_message(&message.message_id, to.clone(), cc, bcc)
            .await
            .map_err(command_connect_error)?;

        self.audit_log
            .record(&NewApiAuditRecord::communication_email_send(
                "hermes-frontend",
                &outbox.account_id,
                recipient_count,
            ))
            .await
            .map_err(audit_connect_error)?;

        Response::ok(RedirectMessageResponse {
            message_id: outbox.outbox_id.clone(),
            outbox_id: Some(outbox.outbox_id),
            accepted: outbox.to_recipients.clone(),
            accepted_recipients: outbox.to_recipients,
            transport: "outbox".to_owned(),
            status: outbox.status.as_str().to_owned(),
            scheduled_send_at: outbox.scheduled_send_at.map(timestamp_string),
            undo_deadline_at: outbox.undo_deadline_at.map(timestamp_string),
            failure_reason: None,
            ..Default::default()
        })
    }

    async fn search_attachments(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, AttachmentSearchRequest>,
    ) -> ServiceResult<AttachmentSearchResponse> {
        let req = req.to_owned_message();
        let page = self
            .attachment_search_store
            .search(AttachmentSearchQuery {
                account_id: req.account_id.as_deref(),
                query: req.query.as_deref(),
                content_type: req.content_type.as_deref(),
                scan_status: req.scan_status.as_deref(),
                cursor: req.cursor.as_deref(),
                limit: normalize_limit(req.limit, 100, 250),
            })
            .await
            .map_err(attachment_search_connect_error)?;

        Response::ok(proto_attachment_search_page(page))
    }

    async fn get_attachment_preview(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetAttachmentPreviewRequest>,
    ) -> ServiceResult<GetAttachmentPreviewResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let attachment = self
            .storage_store
            .attachment_by_id(&req.attachment_id)
            .await
            .map_err(storage_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "attachment was not found"))?;

        if attachment.storage_kind != "local_fs" {
            return Err(invalid_argument_error(
                "attachment preview requires a local blob",
            ));
        }
        if !is_preview_allowed_by_scan_status(&attachment) {
            return Err(invalid_argument_error(
                "attachment preview is blocked by attachment scan status",
            ));
        }
        let preview_kind = attachment_preview_kind(&attachment).ok_or_else(|| {
            invalid_argument_error("attachment preview supports text and image attachments only")
        })?;

        let bytes = crate::app::api_support::communication_blob_store()
            .read_blob(&attachment.storage_path)
            .await
            .map_err(storage_connect_error)?;
        let byte_count = bytes.len();

        match preview_kind {
            AttachmentPreviewKind::Text => {
                text_attachment_preview_proto(attachment, bytes, byte_count)
            }
            AttachmentPreviewKind::Image => {
                image_attachment_preview_proto(attachment, bytes, byte_count)
            }
        }
    }

    async fn get_attachment_archive_inspection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetAttachmentArchiveInspectionRequest>,
    ) -> ServiceResult<GetAttachmentArchiveInspectionResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let attachment = self
            .storage_store
            .attachment_by_id(&req.attachment_id)
            .await
            .map_err(storage_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "attachment was not found"))?;

        if attachment.storage_kind != "local_fs" {
            return Err(invalid_argument_error(
                "attachment archive inspection requires a local blob",
            ));
        }
        if !is_zip_attachment(&attachment) {
            return Err(invalid_argument_error(
                "attachment archive inspection supports ZIP attachments only",
            ));
        }

        let bytes = crate::app::api_support::communication_blob_store()
            .read_blob(&attachment.storage_path)
            .await
            .map_err(storage_connect_error)?;
        let report =
            inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).map_err(|error| {
                tracing::warn!(
                    attachment_id = %attachment.attachment.attachment_id,
                    error = %error,
                    "attachment archive inspection rejected archive"
                );
                invalid_argument_error("attachment archive inspection failed")
            })?;

        Response::ok(GetAttachmentArchiveInspectionResponse {
            attachment_id: attachment.attachment.attachment_id,
            message_id: attachment.attachment.message_id,
            filename: attachment.attachment.filename,
            content_type: attachment.attachment.content_type,
            scan_status: attachment.attachment.scan_status.as_str().to_owned(),
            report: Some(proto_archive_inspection_report(report)).into(),
            ..Default::default()
        })
    }

    async fn translate_attachment(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TranslateAttachmentRequest>,
    ) -> ServiceResult<TranslateAttachmentResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let target_language = req.target_language.trim();
        if target_language.is_empty() {
            return Err(invalid_argument_error("target_language is required"));
        }
        let source_text = req.source_text.trim();
        if source_text.is_empty() {
            return Err(invalid_argument_error("source_text is required"));
        }
        if source_text.chars().count() > MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS {
            return Err(invalid_argument_error("source_text exceeds max length"));
        }

        let attachment = self
            .storage_store
            .attachment_by_id(&req.attachment_id)
            .await
            .map_err(storage_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "attachment was not found"))?;
        let service = thread_multilingual_service(&self.pool, &self.config)
            .await
            .map_err(api_error_connect_error)?;
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                source_text,
            );

        match service.translate(source_text, target_language).await {
            Ok(Some(translation)) => {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    self.pool.clone(),
                    "attachment_translation",
                    &attachment.attachment.message_id,
                    serde_json::json!({
                        "kind": "communication_attachment",
                        "source_code": "ai",
                        "message_id": attachment.attachment.message_id,
                        "attachment_id": attachment.attachment.attachment_id,
                        "operation": "attachment_translation",
                    }),
                    serde_json::json!({
                        "target_language": translation.target_language,
                        "original_language": detection.language,
                        "model": translation.model,
                    }),
                    serde_json::json!({
                        "source": "communication_attachment_translation",
                        "attachment_id": attachment.attachment.attachment_id,
                        "message_id": attachment.attachment.message_id,
                    }),
                    None,
                )
                .await
                .map_err(signal_hub_connect_error)?;

                Response::ok(TranslateAttachmentResponse {
                    attachment_id: attachment.attachment.attachment_id,
                    message_id: attachment.attachment.message_id,
                    filename: attachment.attachment.filename,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: true,
                    text: Some(translation.translated_text),
                    target: translation.target_language,
                    model: Some(translation.model),
                    reason: None,
                    source: ATTACHMENT_TRANSLATION_SOURCE.to_owned(),
                    ..Default::default()
                })
            }
            Ok(None) => Response::ok(TranslateAttachmentResponse {
                attachment_id: attachment.attachment.attachment_id,
                message_id: attachment.attachment.message_id,
                filename: attachment.attachment.filename,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: false,
                text: None,
                target: target_language.to_owned(),
                model: None,
                reason: Some("no LLM configured".to_owned()),
                source: ATTACHMENT_TRANSLATION_SOURCE.to_owned(),
                ..Default::default()
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    attachment_id = %attachment.attachment.attachment_id,
                    "attachment translation failed"
                );
                Response::ok(TranslateAttachmentResponse {
                    attachment_id: attachment.attachment.attachment_id,
                    message_id: attachment.attachment.message_id,
                    filename: attachment.attachment.filename,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: false,
                    text: None,
                    target: target_language.to_owned(),
                    model: None,
                    reason: Some("translation runtime unavailable".to_owned()),
                    source: ATTACHMENT_TRANSLATION_SOURCE.to_owned(),
                    ..Default::default()
                })
            }
        }
    }

    async fn undo_outbox_item(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UndoOutboxItemRequest>,
    ) -> ServiceResult<UndoOutboxItemResponse> {
        let req = req.to_owned_message();
        if req.outbox_id.trim().is_empty() {
            return Err(invalid_argument_error("outbox_id must not be empty"));
        }
        let item = CommunicationCommandService::new(self.pool.clone())
            .undo_outbox(&req.outbox_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(UndoOutboxItemResponse {
            item: Some(proto_outbox_item(item)).into(),
            ..Default::default()
        })
    }
}

fn proto_message_summary(summary: ProjectedMessageSummary) -> ProtoCommunicationMessage {
    proto_message(summary.message, summary.attachment_count)
}

fn proto_message(message: ProjectedMessage, attachment_count: i64) -> ProtoCommunicationMessage {
    ProtoCommunicationMessage {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        observation_id: message.observation_id,
        account_id: message.account_id,
        provider_record_id: message.provider_record_id,
        subject: message.subject,
        sender: message.sender,
        recipients: message.recipients,
        body_text: message.body_text,
        occurred_at: message.occurred_at.map(timestamp_string),
        projected_at: timestamp_string(message.projected_at),
        channel_kind: message.channel_kind,
        conversation_id: message.conversation_id,
        sender_display_name: message.sender_display_name,
        delivery_state: message.delivery_state,
        message_metadata_json: json_string(&message.message_metadata),
        workflow_state: message.workflow_state.as_str().to_owned(),
        importance_score: message.importance_score.map(i32::from),
        ai_category: message.ai_category,
        ai_summary: message.ai_summary,
        ai_summary_generated_at: message.ai_summary_generated_at.map(timestamp_string),
        local_state: message.local_state.as_str().to_owned(),
        local_state_changed_at: message.local_state_changed_at.map(timestamp_string),
        local_state_reason: message.local_state_reason,
        attachment_count,
        ..Default::default()
    }
}

fn proto_attachment_from_storage(
    attachment: StoredCommunicationAttachmentWithBlob,
) -> ProtoCommunicationMessageAttachment {
    let attachment_record = attachment.attachment;
    ProtoCommunicationMessageAttachment {
        attachment_id: attachment_record.attachment_id,
        message_id: attachment_record.message_id,
        raw_record_id: attachment_record.raw_record_id,
        blob_id: attachment_record.blob_id,
        provider_attachment_id: attachment_record.provider_attachment_id,
        filename: attachment_record.filename,
        content_type: attachment_record.content_type,
        size_bytes: attachment_record.size_bytes,
        sha256: attachment_record.sha256,
        disposition: attachment_record.disposition.as_str().to_owned(),
        scan_status: attachment_record.scan_status.as_str().to_owned(),
        scan_engine: attachment_record.scan_engine,
        scan_checked_at: attachment_record.scan_checked_at.map(timestamp_string),
        scan_summary: attachment_record.scan_summary,
        scan_metadata_json: json_string(&attachment_record.scan_metadata),
        storage_kind: attachment.storage_kind,
        storage_path: attachment.storage_path,
        created_at: timestamp_string(attachment_record.created_at),
        updated_at: timestamp_string(attachment_record.updated_at),
        ..Default::default()
    }
}

fn proto_attachment_from_thread(
    attachment: ThreadMessageAttachment,
) -> ProtoCommunicationMessageAttachment {
    ProtoCommunicationMessageAttachment {
        attachment_id: attachment.attachment_id,
        message_id: attachment.message_id,
        raw_record_id: attachment.raw_record_id,
        blob_id: attachment.blob_id,
        provider_attachment_id: attachment.provider_attachment_id,
        filename: attachment.filename,
        content_type: attachment.content_type,
        size_bytes: attachment.size_bytes,
        sha256: attachment.sha256,
        disposition: attachment.disposition,
        scan_status: attachment.scan_status,
        scan_engine: attachment.scan_engine,
        scan_checked_at: attachment.scan_checked_at.map(timestamp_string),
        scan_summary: attachment.scan_summary,
        scan_metadata_json: json_string(&attachment.scan_metadata),
        storage_kind: attachment.storage_kind,
        storage_path: attachment.storage_path,
        created_at: timestamp_string(attachment.created_at),
        updated_at: timestamp_string(attachment.updated_at),
        ..Default::default()
    }
}

fn proto_thread(item: CommunicationThread) -> ProtoCommunicationThread {
    ProtoCommunicationThread {
        thread_id: item.thread_id,
        account_id: item.account_id,
        subject: item.subject,
        message_count: item.message_count,
        participant_count: item.participant_count,
        first_message_at: item.first_message_at.map(timestamp_string),
        last_message_at: item.last_message_at.map(timestamp_string),
        last_activity_at: timestamp_string(item.last_activity_at),
        has_open_action: item.has_open_action,
        has_attachments: item.has_attachments,
        dominant_workflow_state: item.dominant_workflow_state,
        ..Default::default()
    }
}

fn proto_workflow_state_count(
    item: crate::domains::communications::messages::WorkflowStateCount,
) -> ProtoWorkflowStateCount {
    ProtoWorkflowStateCount {
        state: item.state.as_str().to_owned(),
        count: item.count,
        ..Default::default()
    }
}

fn proto_subscription_source(item: SubscriptionSource) -> ProtoSubscriptionSource {
    ProtoSubscriptionSource {
        sender: item.sender,
        message_count: item.message_count,
        first_seen: item.first_seen,
        last_seen: item.last_seen,
        is_newsletter: item.is_newsletter,
        has_unsubscribe: item.has_unsubscribe,
        ..Default::default()
    }
}

fn proto_mailbox_health(item: MailboxHealth) -> ProtoMailboxHealth {
    ProtoMailboxHealth {
        total_messages: item.total_messages,
        unread: item.unread,
        needs_action: item.needs_action,
        waiting: item.waiting,
        done: item.done,
        archived: item.archived,
        spam: item.spam,
        important: item.important,
        with_attachments: item.with_attachments,
        average_importance: item.average_importance,
        oldest_message_days: item.oldest_message_days,
        ..Default::default()
    }
}

fn proto_sender_stats(item: SenderStats) -> ProtoSenderStats {
    ProtoSenderStats {
        sender: item.sender,
        message_count: item.message_count,
        avg_importance: item.avg_importance,
        last_message_days: item.last_message_days,
        ..Default::default()
    }
}

fn proto_communication_architecture_blocker(
    item: crate::domains::communications::blockers::ArchitectureBlocker,
) -> ProtoCommunicationArchitectureBlocker {
    ProtoCommunicationArchitectureBlocker {
        section: item.section,
        feature: item.feature,
        reason: item.reason,
        resolution: item.resolution,
        ..Default::default()
    }
}

fn proto_communication_persona(item: CommunicationPersona) -> ProtoCommunicationPersona {
    ProtoCommunicationPersona {
        persona_id: item.persona_id,
        account_id: item.account_id,
        name: item.name,
        display_name: item.display_name,
        signature: item.signature,
        default_language: item.default_language,
        default_tone: item.default_tone,
        is_default: item.is_default,
        metadata_json: json_string(&item.metadata),
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn proto_rich_template(item: CommunicationTemplate) -> ProtoRichTemplate {
    ProtoRichTemplate {
        template_id: item.template_id,
        name: item.name,
        subject_template: item.subject_template,
        body_template: item.body_template,
        variables: item.variables,
        placeholder_variables: item.placeholder_variables,
        undeclared_variables: item.undeclared_variables,
        unused_variables: item.unused_variables,
        malformed_placeholders: item.malformed_placeholders,
        language: item.language,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn proto_rendered_rich_template(item: RenderedTemplate) -> ProtoRenderedRichTemplate {
    ProtoRenderedRichTemplate {
        subject: item.subject,
        body: item.body,
        missing_variables: item.missing_variables,
        unresolved_variables: item.unresolved_variables,
        malformed_placeholders: item.malformed_placeholders,
        ..Default::default()
    }
}

fn proto_rich_template_mail_merge_preview(
    item: CommunicationMergePreview,
) -> RichTemplateMailMergePreviewResponse {
    RichTemplateMailMergePreviewResponse {
        template_id: item.template_id,
        row_count: item.row_count as u32,
        ready_count: item.ready_count as u32,
        blocked_count: item.blocked_count as u32,
        items: item
            .items
            .into_iter()
            .map(proto_rich_template_mail_merge_preview_item)
            .collect(),
        ..Default::default()
    }
}

fn proto_rich_template_mail_merge_preview_item(
    item: CommunicationMergePreviewItem,
) -> ProtoRichTemplateMailMergePreviewItem {
    ProtoRichTemplateMailMergePreviewItem {
        row_id: item.row_id,
        ready: item.ready,
        rendered: Some(proto_rendered_rich_template(item.rendered)).into(),
        ..Default::default()
    }
}

fn proto_search_result(
    item: crate::engines::search::SearchResult,
) -> ProtoCommunicationSearchResult {
    ProtoCommunicationSearchResult {
        object_id: item.object_id,
        object_kind: item.object_kind,
        title: item.title,
        ..Default::default()
    }
}

fn proto_bulk_message_action_outcome(
    outcome: BulkMessageActionOutcome,
) -> ProtoBulkMessageActionResponse {
    ProtoBulkMessageActionResponse {
        action: outcome.action,
        requested_count: outcome.requested_count as u32,
        matched_count: outcome.matched_count as u32,
        updated_count: outcome.updated_count as u32,
        not_found: outcome.not_found,
        ..Default::default()
    }
}

fn proto_extracted_task(
    item: crate::domains::communications::extract::ExtractedTask,
) -> ProtoExtractedTask {
    ProtoExtractedTask {
        title: item.title,
        due_date: item.due_date,
        assignee: item.assignee,
        priority: item.priority,
        source: item.source,
        ..Default::default()
    }
}

fn proto_extracted_note(
    item: crate::domains::communications::extract::ExtractedNote,
) -> ProtoExtractedNote {
    ProtoExtractedNote {
        title: item.title,
        content: item.content,
        tags: item.tags,
        source: item.source,
        ..Default::default()
    }
}

fn proto_message_summary_contract(
    item: crate::workflows::email_intelligence::EmailSummaryContract,
) -> ProtoMessageSummaryContract {
    ProtoMessageSummaryContract {
        key_points: item.key_points,
        action_items: item.action_items,
        risks: item.risks,
        deadlines: item.deadlines,
        event_candidates: item
            .event_candidates
            .into_iter()
            .map(proto_message_knowledge_candidate)
            .collect(),
        persona_candidates: item
            .persona_candidates
            .into_iter()
            .map(proto_message_knowledge_candidate)
            .collect(),
        organization_candidates: item
            .organization_candidates
            .into_iter()
            .map(proto_message_knowledge_candidate)
            .collect(),
        document_candidates: item
            .document_candidates
            .into_iter()
            .map(proto_message_knowledge_candidate)
            .collect(),
        agreement_candidates: item
            .agreement_candidates
            .into_iter()
            .map(proto_message_knowledge_candidate)
            .collect(),
        ..Default::default()
    }
}

fn proto_message_knowledge_candidate(
    item: crate::workflows::email_intelligence::EmailKnowledgeCandidate,
) -> ProtoMessageKnowledgeCandidate {
    ProtoMessageKnowledgeCandidate {
        title: item.title,
        evidence: item.evidence,
        ..Default::default()
    }
}

fn proto_workflow_action_request(
    req: ProtoWorkflowActionRequest,
) -> Result<HandlerWorkflowActionRequest, ConnectError> {
    Ok(HandlerWorkflowActionRequest {
        command_id: req.command_id,
        action: parse_workflow_action_kind(req.action.as_str())?,
        source: req
            .source
            .as_option()
            .map(|source| HandlerWorkflowActionSource {
                kind: source.kind.clone(),
                id: source.id.clone(),
            }),
        input: req
            .input
            .as_option()
            .map(|input| {
                Ok::<_, ConnectError>(HandlerWorkflowActionInput {
                    title: input.title.clone(),
                    body: input.body.clone(),
                    email: input.email.clone(),
                    display_name: input.display_name.clone(),
                    starts_at: input
                        .starts_at
                        .as_deref()
                        .map(parse_timestamp)
                        .transpose()?,
                    ends_at: input.ends_at.as_deref().map(parse_timestamp).transpose()?,
                    due_at: input.due_at.as_deref().map(parse_timestamp).transpose()?,
                    document_id: input.document_id.clone(),
                })
            })
            .transpose()?,
    })
}

fn proto_workflow_action_response(
    item: HandlerWorkflowActionResponse,
) -> ProtoWorkflowActionResponse {
    ProtoWorkflowActionResponse {
        command_id: item.command_id,
        event_id: item.event_id,
        action: workflow_action_kind_str(&item.action).to_owned(),
        status: workflow_action_status_str(&item.status).to_owned(),
        target: Some(ProtoWorkflowActionTarget {
            kind: workflow_action_target_kind_str(&item.target.kind).to_owned(),
            id: item.target.id,
            ..Default::default()
        })
        .into(),
        provenance: Some(ProtoWorkflowActionProvenance {
            source_kind: item.provenance.source_kind,
            source_id: item.provenance.source_id,
            confidence: item.provenance.confidence,
            evidence: item.provenance.evidence,
            ..Default::default()
        })
        .into(),
        ..Default::default()
    }
}

fn proto_message_auth_result(
    result: &str,
    domain: Option<String>,
    ip: Option<String>,
    selector: Option<String>,
    policy: Option<String>,
) -> ProtoMessageAuthResult {
    ProtoMessageAuthResult {
        result: result.to_owned(),
        domain,
        ip,
        selector,
        policy,
        ..Default::default()
    }
}

fn proto_message_auth_report(
    item: crate::domains::communications::spf_dkim::AuthResults,
) -> ProtoMessageAuthReport {
    ProtoMessageAuthReport {
        spf: item
            .spf
            .map(|value| {
                proto_message_auth_result(&value.result, value.domain, value.ip, None, None)
            })
            .map(|value| Some(value).into())
            .unwrap_or_default(),
        dkim: item
            .dkim
            .map(|value| {
                proto_message_auth_result(&value.result, value.domain, None, value.selector, None)
            })
            .map(|value| Some(value).into())
            .unwrap_or_default(),
        dmarc: item
            .dmarc
            .map(|value| {
                proto_message_auth_result(&value.result, value.domain, None, None, value.policy)
            })
            .map(|value| Some(value).into())
            .unwrap_or_default(),
        raw_headers: item.raw_headers,
        ..Default::default()
    }
}

fn proto_message_auth_risk_report(
    item: crate::domains::communications::spf_dkim::SpfDkimReport,
) -> ProtoMessageAuthRiskReport {
    ProtoMessageAuthRiskReport {
        has_spf: item.has_spf,
        spf_pass: item.spf_pass,
        has_dkim: item.has_dkim,
        dkim_pass: item.dkim_pass,
        has_dmarc: item.has_dmarc,
        dmarc_pass: item.dmarc_pass,
        is_spoofed: item.is_spoofed,
        risk_summary: item.risk_summary,
        ..Default::default()
    }
}

fn proto_attachment_search_page(page: AttachmentSearchPage) -> AttachmentSearchResponse {
    AttachmentSearchResponse {
        items: page
            .items
            .into_iter()
            .map(proto_attachment_search_item)
            .collect(),
        next_cursor: page.next_cursor,
        has_more: page.has_more,
        ..Default::default()
    }
}

fn proto_attachment_search_item(item: AttachmentSearchResult) -> ProtoAttachmentSearchItem {
    ProtoAttachmentSearchItem {
        attachment_id: item.attachment_id,
        message_id: item.message_id,
        raw_record_id: item.raw_record_id,
        account_id: item.account_id,
        message_subject: item.message_subject,
        sender: item.sender,
        occurred_at: item.occurred_at.map(timestamp_string),
        blob_id: item.blob_id,
        provider_attachment_id: item.provider_attachment_id,
        filename: item.filename,
        content_type: item.content_type,
        size_bytes: item.size_bytes,
        sha256: item.sha256,
        disposition: serde_json::to_string(&item.disposition)
            .unwrap_or_else(|_| "\"unknown\"".to_owned())
            .trim_matches('"')
            .to_owned(),
        scan_status: serde_json::to_string(&item.scan_status)
            .unwrap_or_else(|_| "\"failed\"".to_owned())
            .trim_matches('"')
            .to_owned(),
        scan_engine: item.scan_engine,
        scan_checked_at: item.scan_checked_at.map(timestamp_string),
        scan_summary: item.scan_summary,
        storage_kind: item.storage_kind,
        storage_path: item.storage_path,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn proto_archive_inspection_report(
    report: ArchiveInspectionReport,
) -> ProtoArchiveInspectionReport {
    ProtoArchiveInspectionReport {
        archive_kind: report.archive_kind,
        entry_count: report.entry_count as u32,
        total_uncompressed_bytes: report.total_uncompressed_bytes,
        has_nested_archive: report.has_nested_archive,
        entries: report
            .entries
            .into_iter()
            .map(|entry| ProtoArchiveInspectionEntry {
                name: entry.name,
                normalized_path: entry.normalized_path,
                compressed_size: entry.compressed_size,
                uncompressed_size: entry.uncompressed_size,
                is_dir: entry.is_dir,
                is_nested_archive: entry.is_nested_archive,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

fn proto_thread_message(item: ThreadMessage) -> ProtoThreadMessage {
    ProtoThreadMessage {
        message_id: item.message_id,
        provider_record_id: item.provider_record_id,
        account_id: item.account_id,
        subject: item.subject,
        sender: item.sender,
        sender_display_name: item.sender_display_name,
        body_text: item.body_text,
        occurred_at: item.occurred_at.map(timestamp_string),
        projected_at: timestamp_string(item.projected_at),
        workflow_state: item.workflow_state,
        importance_score: item.importance_score.map(i32::from),
        ai_category: item.ai_category,
        ai_summary: item.ai_summary,
        delivery_state: item.delivery_state,
        attachment_count: item.attachment_count,
        attachments: item
            .attachments
            .into_iter()
            .map(proto_attachment_from_thread)
            .collect(),
        ..Default::default()
    }
}

fn proto_draft(item: CommunicationDraft) -> ProtoCommunicationDraft {
    ProtoCommunicationDraft {
        draft_id: item.draft_id,
        account_id: item.account_id,
        persona_id: item.persona_id,
        to_recipients: item.to_recipients,
        cc_recipients: item.cc_recipients,
        bcc_recipients: item.bcc_recipients,
        subject: item.subject,
        body_text: item.body_text,
        body_html: item.body_html,
        in_reply_to: item.in_reply_to,
        references: item.references,
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at.map(timestamp_string),
        send_attempts: item.send_attempts,
        last_error: item.last_error,
        metadata_json: json_string(&item.metadata),
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn proto_saved_search(item: CommunicationSavedSearch) -> ProtoCommunicationSavedSearch {
    ProtoCommunicationSavedSearch {
        saved_search_id: item.saved_search_id,
        name: item.name,
        description: item.description,
        account_id: item.account_id,
        query: item.query,
        workflow_state: item.workflow_state.map(|state| state.as_str().to_owned()),
        local_state: item.local_state.as_str().to_owned(),
        channel_kind: item.channel_kind,
        is_smart_folder: item.is_smart_folder,
        sort_order: item.sort_order,
        message_count: item.message_count,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn proto_folder(item: CommunicationFolder) -> ProtoCommunicationFolder {
    ProtoCommunicationFolder {
        folder_id: item.folder_id,
        account_id: item.account_id,
        name: item.name,
        description: item.description,
        color: item.color,
        sort_order: item.sort_order,
        message_count: item.message_count,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn proto_folder_message(item: FolderMessage) -> ProtoFolderMessage {
    ProtoFolderMessage {
        folder_id: item.folder_id,
        message_id: item.message_id,
        account_id: item.account_id,
        subject: item.subject,
        sender: item.sender,
        occurred_at: item.occurred_at.map(timestamp_string),
        projected_at: timestamp_string(item.projected_at),
        workflow_state: item.workflow_state.as_str().to_owned(),
        local_state: item.local_state.as_str().to_owned(),
        added_at: timestamp_string(item.added_at),
        attachment_count: item.attachment_count,
        ..Default::default()
    }
}

fn proto_folder_message_action(
    item: FolderMessageActionResponse,
) -> ProtoFolderMessageActionResult {
    ProtoFolderMessageActionResult {
        operation: item.operation.as_str().to_owned(),
        folder_id: item.folder_id,
        message_id: item.message_id,
        message: Some(proto_folder_message(item.message)).into(),
        ..Default::default()
    }
}

fn proto_outbox_item(item: CommunicationOutboxItem) -> ProtoCommunicationOutboxItem {
    ProtoCommunicationOutboxItem {
        outbox_id: item.outbox_id,
        account_id: item.account_id,
        draft_id: item.draft_id,
        to_recipients: item.to_recipients,
        cc_recipients: item.cc_recipients,
        bcc_recipients: item.bcc_recipients,
        subject: item.subject,
        body_text: item.body_text,
        body_html: item.body_html,
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at.map(timestamp_string),
        undo_deadline_at: item.undo_deadline_at.map(timestamp_string),
        send_attempts: item.send_attempts,
        claimed_at: item.claimed_at.map(timestamp_string),
        sent_at: item.sent_at.map(timestamp_string),
        provider_message_id: item.provider_message_id,
        last_error: item.last_error,
        metadata_json: json_string(&item.metadata),
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}

fn parse_workflow_state(value: &str) -> Result<WorkflowState, ConnectError> {
    value
        .parse()
        .map_err(|_| invalid_argument_error(format!("invalid workflow_state: {value}")))
}

fn parse_local_state(value: &str) -> Result<LocalMessageState, ConnectError> {
    value
        .parse()
        .map_err(|_| invalid_argument_error(format!("invalid local_state: {value}")))
}

fn parse_match_mode(value: Option<&str>) -> Result<MessageSearchMatchMode, ConnectError> {
    match value.unwrap_or("all").trim() {
        "" | "all" => Ok(MessageSearchMatchMode::All),
        "any" => Ok(MessageSearchMatchMode::Any),
        other => Err(invalid_argument_error(format!(
            "invalid match_mode: {other}"
        ))),
    }
}

fn parse_workflow_action_kind(value: &str) -> Result<HandlerWorkflowActionKind, ConnectError> {
    match value.trim() {
        "reply" => Ok(HandlerWorkflowActionKind::Reply),
        "create_task" => Ok(HandlerWorkflowActionKind::CreateTask),
        "create_note" => Ok(HandlerWorkflowActionKind::CreateNote),
        "create_document" => Ok(HandlerWorkflowActionKind::CreateDocument),
        "create_event" => Ok(HandlerWorkflowActionKind::CreateEvent),
        "link_document" => Ok(HandlerWorkflowActionKind::LinkDocument),
        "create_contact" => Ok(HandlerWorkflowActionKind::CreateContact),
        "archive" => Ok(HandlerWorkflowActionKind::Archive),
        _ => Err(invalid_argument_error(format!(
            "invalid workflow action: {value}"
        ))),
    }
}

fn workflow_action_kind_str(value: &HandlerWorkflowActionKind) -> &'static str {
    match value {
        HandlerWorkflowActionKind::Reply => "reply",
        HandlerWorkflowActionKind::CreateTask => "create_task",
        HandlerWorkflowActionKind::CreateNote => "create_note",
        HandlerWorkflowActionKind::CreateDocument => "create_document",
        HandlerWorkflowActionKind::CreateEvent => "create_event",
        HandlerWorkflowActionKind::LinkDocument => "link_document",
        HandlerWorkflowActionKind::CreateContact => "create_contact",
        HandlerWorkflowActionKind::Archive => "archive",
    }
}

fn workflow_action_status_str(value: &HandlerWorkflowActionStatus) -> &'static str {
    match value {
        HandlerWorkflowActionStatus::Created => "created",
        HandlerWorkflowActionStatus::Updated => "updated",
        HandlerWorkflowActionStatus::Linked => "linked",
        HandlerWorkflowActionStatus::Opened => "opened",
        HandlerWorkflowActionStatus::Archived => "archived",
        HandlerWorkflowActionStatus::Noop => "noop",
    }
}

fn workflow_action_target_kind_str(value: &HandlerWorkflowActionTargetKind) -> &'static str {
    match value {
        HandlerWorkflowActionTargetKind::Compose => "compose",
        HandlerWorkflowActionTargetKind::Message => "message",
        HandlerWorkflowActionTargetKind::Task => "task",
        HandlerWorkflowActionTargetKind::Document => "document",
        HandlerWorkflowActionTargetKind::CalendarEvent => "calendar_event",
        HandlerWorkflowActionTargetKind::Person => "person",
    }
}

fn parse_bulk_action_request(
    request: &ProtoBulkMessageActionRequest,
) -> Result<BulkMessageAction, ConnectError> {
    match request.action.trim() {
        "mark_read" => Ok(BulkMessageAction::MarkRead),
        "mark_unread" => Ok(BulkMessageAction::MarkUnread),
        "archive" => Ok(BulkMessageAction::Archive),
        "trash" => Ok(BulkMessageAction::Trash),
        "restore" => Ok(BulkMessageAction::Restore),
        "pin" => Ok(BulkMessageAction::Pin),
        "unpin" => Ok(BulkMessageAction::Unpin),
        "important" => Ok(BulkMessageAction::Important),
        "not_important" => Ok(BulkMessageAction::NotImportant),
        "add_label" => request
            .label
            .clone()
            .map(BulkMessageAction::AddLabel)
            .ok_or_else(|| invalid_argument_error("label is required for add_label")),
        "remove_label" => request
            .label
            .clone()
            .map(BulkMessageAction::RemoveLabel)
            .ok_or_else(|| invalid_argument_error("label is required for remove_label")),
        "snooze" => {
            let until = request
                .snooze_until
                .as_deref()
                .ok_or_else(|| invalid_argument_error("snooze_until is required for snooze"))?;
            Ok(BulkMessageAction::Snooze(parse_timestamp(until)?))
        }
        _ => Err(invalid_argument_error("invalid bulk message action")),
    }
}

fn parse_draft_status(value: &str) -> Result<DraftStatus, ConnectError> {
    DraftStatus::parse(value)
        .ok_or_else(|| invalid_argument_error(format!("invalid draft status: {value}")))
}

fn parse_outbox_status(value: &str) -> Result<CommunicationOutboxStatus, ConnectError> {
    CommunicationOutboxStatus::parse(value)
        .ok_or_else(|| invalid_argument_error(format!("invalid outbox status: {value}")))
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, ConnectError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| invalid_argument_error(format!("invalid RFC3339 timestamp: {value}")))
}

fn parse_json_object(value: &str, field_name: &str) -> Result<Value, ConnectError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(json!({}));
    }
    let parsed: Value = serde_json::from_str(trimmed)
        .map_err(|_| invalid_argument_error(format!("{field_name} must contain valid JSON")))?;
    if !parsed.is_object() {
        return Err(invalid_argument_error(format!(
            "{field_name} must contain a JSON object"
        )));
    }
    Ok(parsed)
}

fn normalize_limit(limit: u32, default: i64, max: i64) -> i64 {
    match limit {
        0 => default,
        value => i64::from(value).clamp(1, max),
    }
}

fn trim_non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    recipients
        .into_iter()
        .map(|recipient| recipient.trim().to_owned())
        .filter(|recipient| !recipient.is_empty())
        .collect()
}

fn is_preview_allowed_by_scan_status(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    matches!(
        attachment.attachment.scan_status.as_str(),
        "not_scanned" | "clean"
    )
}

enum AttachmentPreviewKind {
    Text,
    Image,
}

fn attachment_preview_kind(
    attachment: &StoredCommunicationAttachmentWithBlob,
) -> Option<AttachmentPreviewKind> {
    if is_previewable_text_attachment(attachment) {
        return Some(AttachmentPreviewKind::Text);
    }
    if is_previewable_image_attachment(attachment) {
        return Some(AttachmentPreviewKind::Image);
    }
    None
}

fn is_previewable_text_attachment(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    if content_type.starts_with("text/") {
        return true;
    }
    matches!(
        content_type.as_str(),
        "application/json" | "application/xml" | "application/yaml" | "application/x-yaml"
    ) || attachment
        .attachment
        .filename
        .as_deref()
        .map(|filename| {
            let filename = filename.trim().to_ascii_lowercase();
            filename.ends_with(".txt")
                || filename.ends_with(".md")
                || filename.ends_with(".csv")
                || filename.ends_with(".json")
                || filename.ends_with(".xml")
                || filename.ends_with(".yaml")
                || filename.ends_with(".yml")
        })
        .unwrap_or(false)
}

fn is_previewable_image_attachment(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase()
        .starts_with("image/")
}

fn preview_image_content_type(attachment: &StoredCommunicationAttachmentWithBlob) -> Option<&str> {
    let content_type = attachment.attachment.content_type.trim();
    if content_type.is_empty() || !content_type.to_ascii_lowercase().starts_with("image/") {
        return None;
    }
    Some(content_type)
}

fn is_zip_attachment(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    content_type == "application/zip"
        || attachment
            .attachment
            .filename
            .as_deref()
            .map(|filename| filename.trim().to_ascii_lowercase().ends_with(".zip"))
            .unwrap_or(false)
}

fn text_attachment_preview_proto(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    let truncated = byte_count > MAX_TEXT_PREVIEW_BYTES;
    let preview_bytes = if truncated {
        &bytes[..MAX_TEXT_PREVIEW_BYTES]
    } else {
        &bytes
    };
    let text = String::from_utf8_lossy(preview_bytes).into_owned();

    Response::ok(GetAttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "text".to_owned(),
        text,
        data_url: None,
        truncated,
        byte_count: byte_count as u64,
        max_preview_bytes: MAX_TEXT_PREVIEW_BYTES as u64,
        ..Default::default()
    })
}

fn image_attachment_preview_proto(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    if byte_count > MAX_IMAGE_PREVIEW_BYTES {
        return Err(invalid_argument_error(
            "attachment image preview exceeds size limit",
        ));
    }
    let content_type = preview_image_content_type(&attachment).unwrap_or("image/png");
    let data_url = format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    );

    Response::ok(GetAttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "image".to_owned(),
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count: byte_count as u64,
        max_preview_bytes: MAX_IMAGE_PREVIEW_BYTES as u64,
        ..Default::default()
    })
}

async fn thread_multilingual_service(
    pool: &PgPool,
    config: &AppConfig,
) -> Result<crate::domains::communications::multilingual::MultilingualService, ApiError> {
    Ok(
        crate::domains::communications::multilingual::MultilingualService::new(
            thread_ai_runtime_port_optional(pool, config).await?,
        ),
    )
}

async fn thread_ai_runtime_port_optional(
    pool: &PgPool,
    config: &AppConfig,
) -> Result<Option<crate::platform::ai_runtime::SharedAiRuntimePort>, ApiError> {
    if !thread_ai_requests_allowed(pool).await? {
        return Ok(None);
    }

    let settings = ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(config)
        .await?;
    Ok(thread_ai_runtime_port(config, &settings))
}

async fn thread_ai_requests_allowed(pool: &PgPool) -> Result<bool, ApiError> {
    crate::domains::signal_hub::SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await?;
    crate::platform::events::runtime_allows_processing(
        pool,
        "ai",
        AI_REQUEST_RUNTIME,
        &serde_json::json!({
            "label": "AI request runtime",
            "scope": "runtime",
        }),
    )
    .await
    .map_err(crate::domains::signal_hub::SignalHubError::from)
    .map_err(ApiError::from)
}

fn thread_ai_runtime_port(
    config: &AppConfig,
    settings: &crate::platform::settings::AiRuntimeSettings,
) -> Option<crate::platform::ai_runtime::SharedAiRuntimePort> {
    thread_ai_runtime_client(config, settings)
        .ok()
        .map(|runtime| Arc::new(runtime) as crate::platform::ai_runtime::SharedAiRuntimePort)
}

fn thread_ai_runtime_client(
    config: &AppConfig,
    settings: &crate::platform::settings::AiRuntimeSettings,
) -> Result<AiRuntimeClient, ApiError> {
    match settings.provider {
        AiRuntimeProvider::Ollama => Ok(AiRuntimeClient::Ollama(OllamaClient::new(
            OllamaClientConfig::new(
                &settings.base_url,
                &settings.chat_model,
                &settings.embedding_model,
            )
            .with_timeout_seconds(settings.timeout_seconds),
        )?)),
        AiRuntimeProvider::OmniRoute => {
            let api_key = config.omniroute_api_key().cloned().ok_or_else(|| {
                ApiError::Ai(crate::ai::core::AiError::Runtime(
                    AiRuntimeError::OmniRoute(OmniRouteError::MissingApiKey),
                ))
            })?;
            Ok(AiRuntimeClient::OmniRoute(OmniRouteClient::new(
                OmniRouteClientConfig::new(
                    &settings.base_url,
                    &settings.chat_model,
                    &settings.embedding_model,
                    api_key,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )?))
        }
    }
}

fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn json_string(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_owned())
}

fn message_connect_error(error: MessageProjectionError) -> ConnectError {
    match error {
        MessageProjectionError::MessageNotFound => {
            ConnectError::new(ErrorCode::NotFound, error.to_string())
        }
        MessageProjectionError::InvalidLimit(_)
        | MessageProjectionError::InvalidCursor
        | MessageProjectionError::InvalidWorkflowState(_)
        | MessageProjectionError::InvalidLocalState(_)
        | MessageProjectionError::EmptyField(_)
        | MessageProjectionError::MissingPayloadField(_)
        | MessageProjectionError::InvalidMessageMetadata
        | MessageProjectionError::InvalidStoredRecipients
        | MessageProjectionError::UnsupportedRawBlobStorageKind(_)
        | MessageProjectionError::RawRecordTupleMismatch { .. }
        | MessageProjectionError::RawRecordNotFound(_)
        | MessageProjectionError::InvalidImportanceScore(_) => {
            invalid_argument_error(error.to_string())
        }
        MessageProjectionError::Sqlx(_)
        | MessageProjectionError::CommunicationStorage(_)
        | MessageProjectionError::Rfc822(_)
        | MessageProjectionError::ObservationStore(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn export_connect_error(
    error: crate::domains::communications::export::CommunicationExportError,
) -> ConnectError {
    match error {
        crate::domains::communications::export::CommunicationExportError::NotFound => {
            ConnectError::new(ErrorCode::NotFound, error.to_string())
        }
        crate::domains::communications::export::CommunicationExportError::MessageProjection(
            err,
        ) => message_connect_error(err),
        crate::domains::communications::export::CommunicationExportError::CommunicationStorage(
            err,
        ) => storage_connect_error(err),
    }
}

fn thread_connect_error(error: CommunicationThreadError) -> ConnectError {
    match error {
        CommunicationThreadError::InvalidCursor => invalid_argument_error(error.to_string()),
        CommunicationThreadError::Sqlx(_) | CommunicationThreadError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn draft_connect_error(error: CommunicationDraftError) -> ConnectError {
    match error {
        CommunicationDraftError::Invalid(_) | CommunicationDraftError::InvalidCursor => {
            invalid_argument_error(error.to_string())
        }
        CommunicationDraftError::Sqlx(_)
        | CommunicationDraftError::Observation(_)
        | CommunicationDraftError::EventStore(_)
        | CommunicationDraftError::EventEnvelope(_)
        | CommunicationDraftError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn saved_search_connect_error(error: CommunicationSavedSearchError) -> ConnectError {
    match error {
        CommunicationSavedSearchError::Invalid(_)
        | CommunicationSavedSearchError::InvalidCursor => invalid_argument_error(error.to_string()),
        CommunicationSavedSearchError::Sqlx(_)
        | CommunicationSavedSearchError::Observation(_)
        | CommunicationSavedSearchError::Serde(_)
        | CommunicationSavedSearchError::EventStore(_)
        | CommunicationSavedSearchError::EventEnvelope(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn attachment_search_connect_error(error: AttachmentSearchError) -> ConnectError {
    match error {
        AttachmentSearchError::InvalidCursor => invalid_argument_error(error.to_string()),
        AttachmentSearchError::Sqlx(_) | AttachmentSearchError::CommunicationStorage(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn bulk_action_connect_error(error: BulkMessageActionError) -> ConnectError {
    match error {
        BulkMessageActionError::Invalid(message) => invalid_argument_error(message),
        BulkMessageActionError::Sqlx(_)
        | BulkMessageActionError::ObservationStore(_)
        | BulkMessageActionError::EventStore(_)
        | BulkMessageActionError::EventEnvelope(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn folder_connect_error(error: CommunicationFolderError) -> ConnectError {
    match error {
        CommunicationFolderError::Invalid(_) | CommunicationFolderError::InvalidCursor => {
            invalid_argument_error(error.to_string())
        }
        CommunicationFolderError::Sqlx(_)
        | CommunicationFolderError::Observation(_)
        | CommunicationFolderError::Serde(_)
        | CommunicationFolderError::EventStore(_)
        | CommunicationFolderError::EventEnvelope(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn outbox_connect_error(error: CommunicationOutboxError) -> ConnectError {
    match error {
        CommunicationOutboxError::Invalid(_)
        | CommunicationOutboxError::InvalidCursor
        | CommunicationOutboxError::UndoUnavailable => invalid_argument_error(error.to_string()),
        CommunicationOutboxError::Sqlx(_)
        | CommunicationOutboxError::Serde(_)
        | CommunicationOutboxError::EventStore(_)
        | CommunicationOutboxError::EventEnvelope(_)
        | CommunicationOutboxError::ObservationStore(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn storage_connect_error(error: CommunicationStorageError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn audit_connect_error(error: crate::platform::audit::ApiAuditError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn search_connect_error(
    error: crate::domains::communications::search::IndexEmailError,
) -> ConnectError {
    match error {
        crate::domains::communications::search::IndexEmailError::Messages(message_error) => {
            message_connect_error(message_error)
        }
        crate::domains::communications::search::IndexEmailError::Search(search_error) => {
            ConnectError::new(ErrorCode::Internal, search_error.to_string())
        }
    }
}

fn search_engine_connect_error(error: crate::engines::search::SearchError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn extract_connect_error(
    error: crate::domains::communications::extract::ExtractError,
) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn ai_reply_connect_error(
    error: crate::domains::communications::ai_reply::AiReplyError,
) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn analytics_connect_error(error: EmailAnalyticsError) -> ConnectError {
    match error {
        EmailAnalyticsError::InvalidCursor => invalid_argument_error(error.to_string()),
        EmailAnalyticsError::Sqlx(_) | EmailAnalyticsError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn subscription_connect_error(error: SubscriptionError) -> ConnectError {
    match error {
        SubscriptionError::InvalidCursor => invalid_argument_error(error.to_string()),
        SubscriptionError::Sqlx(_) | SubscriptionError::Serde(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn persona_connect_error(error: CommunicationPersonaError) -> ConnectError {
    match error {
        CommunicationPersonaError::Invalid(_) => invalid_argument_error(error.to_string()),
        CommunicationPersonaError::Sqlx(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn template_connect_error(error: CommunicationTemplateError) -> ConnectError {
    match error {
        CommunicationTemplateError::InvalidTemplate(_) => invalid_argument_error(error.to_string()),
        CommunicationTemplateError::Sqlx(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn signal_hub_connect_error(error: crate::domains::signal_hub::SignalHubError) -> ConnectError {
    ConnectError::new(ErrorCode::Internal, error.to_string())
}

fn ai_state_connect_error(
    error: crate::domains::communications::ai_state::CommunicationAiStateError,
) -> ConnectError {
    match error {
        crate::domains::communications::ai_state::CommunicationAiStateError::Invalid(_) => {
            invalid_argument_error(error.to_string())
        }
        other => ConnectError::new(ErrorCode::Internal, other.to_string()),
    }
}

fn api_error_connect_error(error: crate::app::error::ApiError) -> ConnectError {
    let _ = error;
    ConnectError::new(ErrorCode::Internal, "internal API error")
}

fn send_connect_error(error: CommunicationSendError) -> ConnectError {
    match error {
        CommunicationSendError::InvalidRequest(_)
        | CommunicationSendError::ProviderAccountNotFound => {
            invalid_argument_error(error.to_string())
        }
        CommunicationSendError::CommunicationIngestion(_)
        | CommunicationSendError::Command(_)
        | CommunicationSendError::Audit(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
    }
}

fn command_connect_error(
    error: crate::domains::communications::service::CommunicationCommandServiceError,
) -> ConnectError {
    match error {
        crate::domains::communications::service::CommunicationCommandServiceError::Draft(
            draft_error,
        ) => draft_connect_error(draft_error),
        crate::domains::communications::service::CommunicationCommandServiceError::Outbox(
            outbox_error,
        ) => outbox_connect_error(outbox_error),
        crate::domains::communications::service::CommunicationCommandServiceError::SavedSearch(
            saved_search_error,
        ) => saved_search_connect_error(saved_search_error),
        crate::domains::communications::service::CommunicationCommandServiceError::Folder(
            folder_error,
        ) => folder_connect_error(folder_error),
        crate::domains::communications::service::CommunicationCommandServiceError::MessageProjection(
            message_error,
        ) => message_connect_error(message_error),
        other => ConnectError::new(ErrorCode::Internal, other.to_string()),
    }
}

fn invalid_argument_error(message: impl Into<String>) -> ConnectError {
    ConnectError::new(ErrorCode::InvalidArgument, message.into())
}
