import { getCommunicationsConnectClient } from '../../../../platform/connect/communicationsClient'
import type {
  CommunicationDraft,
  CommunicationOutboxItem,
  DraftDeleteResponse,
  DraftListResponse,
  DraftUpsertRequest,
  OutboxListResponse,
  RedirectMessageRequest,
  SendCommunicationRequest,
  SendCommunicationResponse,
  ThreadListResponse,
  ThreadMessagesResponse
} from '../../types/communications'
import type {
  AttachmentArchiveInspectionResponse,
  AttachmentExtractedTextResponse,
  AttachmentPreviewResponse,
  AttachmentSearchRequest,
  AttachmentSearchResponse,
  AttachmentTextExtractionResponse,
  AttachmentTranslationResponse
} from '../../types/attachments'
import type {
  CommunicationSavedSearch as DomainCommunicationSavedSearch,
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate
} from '../../types/savedSearches'
import type {
  CommunicationFolder,
  CommunicationFolderInput,
  CommunicationFolderListResponse,
  CommunicationFolderUpdate,
  FolderDeleteResponse,
  FolderMessageActionResponse,
  FolderMessageListResponse
} from '../../types/folders'
import type { ThreadTranslationResponse } from '../../types/multilingual'
import {
  emptyStringToNull,
  mapAttachment,
  mapDraftItem,
  mapFolderItem,
  mapFolderMessageActionResult,
  mapFolderMessageItem,
  mapOutboxItem,
  mapSavedSearchItem,
  normalizeAttachmentDisposition,
  normalizeAttachmentScanStatus,
  toNumber
} from './mapping'

export async function fetchCommunicationThreadsConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<ThreadListResponse> {
  const response = await getCommunicationsConnectClient().listThreads({
    accountId,
    cursor,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      thread_id: item.threadId,
      account_id: item.accountId,
      subject: item.subject,
      message_count: toNumber(item.messageCount),
      participant_count: toNumber(item.participantCount),
      first_message_at: item.firstMessageAt ?? null,
      last_message_at: item.lastMessageAt ?? null,
      last_activity_at: item.lastActivityAt,
      has_open_action: item.hasOpenAction,
      has_attachments: item.hasAttachments,
      dominant_workflow_state: item.dominantWorkflowState
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchCommunicationThreadMessagesConnect(
  accountId: string,
  subject: string,
  limit?: number
): Promise<ThreadMessagesResponse> {
  const response = await getCommunicationsConnectClient().listThreadMessages({
    accountId,
    subject,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      message_id: item.messageId,
      provider_record_id: item.providerRecordId,
      account_id: item.accountId,
      subject: item.subject,
      sender: item.sender,
      sender_display_name: item.senderDisplayName ?? null,
      body_text: item.bodyText,
      occurred_at: item.occurredAt ?? null,
      projected_at: item.projectedAt,
      workflow_state: item.workflowState,
      importance_score: item.importanceScore ?? null,
      ai_category: item.aiCategory ?? null,
      ai_summary: item.aiSummary ?? null,
      delivery_state: item.deliveryState,
      attachment_count: toNumber(item.attachmentCount),
      attachments: item.attachments.map(mapAttachment)
    }))
  }
}

export async function translateCommunicationThreadConnect(
  accountId: string,
  subject: string,
  targetLanguage: string,
  limit?: number
): Promise<ThreadTranslationResponse> {
  const response = await getCommunicationsConnectClient().translateThread({
    accountId,
    subject,
    targetLanguage,
    limit: limit ?? 0
  })

  return {
    account_id: response.accountId,
    subject: response.subject,
    target_language: response.targetLanguage,
    items: response.items.map((item) => ({
      message_id: item.messageId,
      original_language: item.originalLanguage,
      confidence: item.confidence,
      translated: item.translated,
      text: item.text ?? null,
      target: item.target,
      model: item.model ?? null,
      reason: item.reason ?? null
    }))
  }
}

export async function searchAttachmentsConnect(
  request: AttachmentSearchRequest = {}
): Promise<AttachmentSearchResponse> {
  const response = await getCommunicationsConnectClient().searchAttachments({
    accountId: request.account_id,
    query: request.q,
    contentType: request.content_type,
    scanStatus: request.scan_status,
    cursor: request.cursor ?? undefined,
    limit: request.limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      attachment_id: item.attachmentId,
      message_id: item.messageId,
      raw_record_id: item.rawRecordId,
      account_id: item.accountId,
      message_subject: item.messageSubject,
      sender: item.sender,
      occurred_at: item.occurredAt ?? null,
      blob_id: item.blobId,
      provider_attachment_id: item.providerAttachmentId,
      filename: item.filename ?? null,
      content_type: item.contentType,
      size_bytes: toNumber(item.sizeBytes),
      sha256: item.sha256,
      disposition: normalizeAttachmentDisposition(item.disposition),
      scan_status: normalizeAttachmentScanStatus(item.scanStatus),
      scan_engine: item.scanEngine ?? null,
      scan_checked_at: item.scanCheckedAt ?? null,
      scan_summary: item.scanSummary ?? null,
      storage_kind: item.storageKind,
      storage_path: item.storagePath,
      extracted_text_match: item.extractedTextMatch,
      created_at: item.createdAt,
      updated_at: item.updatedAt
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function inspectAttachmentArchiveConnect(
  attachmentId: string
): Promise<AttachmentArchiveInspectionResponse> {
  const response = await getCommunicationsConnectClient().getAttachmentArchiveInspection({
    attachmentId
  })

  return {
    attachment_id: response.attachmentId,
    message_id: response.messageId,
    filename: response.filename ?? null,
    content_type: response.contentType,
    scan_status: normalizeAttachmentScanStatus(response.scanStatus),
    report: {
      archive_kind: 'zip',
      entry_count: toNumber(response.report?.entryCount ?? 0),
      total_uncompressed_bytes: toNumber(response.report?.totalUncompressedBytes ?? 0),
      has_nested_archive: response.report?.hasNestedArchive ?? false,
      entries: (response.report?.entries ?? []).map((entry) => ({
        name: entry.name,
        normalized_path: entry.normalizedPath,
        compressed_size: toNumber(entry.compressedSize),
        uncompressed_size: toNumber(entry.uncompressedSize),
        is_dir: entry.isDir,
        is_nested_archive: entry.isNestedArchive
      }))
    }
  }
}

export async function previewAttachmentConnect(
  attachmentId: string
): Promise<AttachmentPreviewResponse> {
  const response = await getCommunicationsConnectClient().getAttachmentPreview({ attachmentId })
  return {
    attachment_id: response.attachmentId,
    message_id: response.messageId,
    filename: response.filename ?? null,
    content_type: response.contentType,
    scan_status: normalizeAttachmentScanStatus(response.scanStatus),
    preview_kind:
      response.previewKind === 'image'
        ? 'image'
        : response.previewKind === 'audio'
          ? 'audio'
          : response.previewKind === 'video'
            ? 'video'
            : response.previewKind === 'pdf'
              ? 'pdf'
              : 'text',
    text: response.text,
    data_url: response.dataUrl ?? null,
    truncated: response.truncated,
    byte_count: toNumber(response.byteCount),
    max_preview_bytes: toNumber(response.maxPreviewBytes)
  }
}

export async function extractAttachmentTextConnect(
  attachmentId: string
): Promise<AttachmentTextExtractionResponse> {
  const response = await getCommunicationsConnectClient().extractAttachmentText({ attachmentId })
  return {
    attachment_id: response.attachmentId,
    status: response.status === 'unsupported' ? 'unsupported' : 'completed',
    extracted_size_bytes:
      response.extractedSizeBytes === undefined ? null : toNumber(response.extractedSizeBytes)
  }
}

export async function fetchAttachmentExtractedTextConnect(
  attachmentId: string
): Promise<AttachmentExtractedTextResponse> {
  const response = await getCommunicationsConnectClient().getAttachmentExtractedText({ attachmentId })
  return {
    attachment_id: response.attachmentId,
    text: response.text,
    truncated: response.truncated,
    extracted_size_bytes: toNumber(response.extractedSizeBytes)
  }
}

export async function translateAttachmentConnect(
  attachmentId: string,
  targetLanguage: string
): Promise<AttachmentTranslationResponse> {
  const response = await getCommunicationsConnectClient().translateAttachment({
    attachmentId,
    targetLanguage
  })

  return {
    attachment_id: response.attachmentId,
    message_id: response.messageId,
    filename: response.filename ?? null,
    original_language: response.originalLanguage,
    confidence: response.confidence,
    translated: response.translated,
    text: response.text ?? null,
    target: response.target,
    model: response.model ?? null,
    reason: response.reason ?? null,
    source: 'durable_extracted_text'
  }
}

export async function fetchCommunicationDraftsConnect(
  accountId?: string,
  status?: string,
  limit?: number,
  cursor?: string
): Promise<DraftListResponse> {
  const response = await getCommunicationsConnectClient().listDrafts({
    accountId,
    status,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapDraftItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function fetchCommunicationSavedSearchesConnect(
  smartFolder?: boolean,
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<SavedSearchListResponse> {
  const response = await getCommunicationsConnectClient().listSavedSearches({
    accountId,
    smartFolder,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapSavedSearchItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function createCommunicationSavedSearchConnect(
  request: SavedSearchInput
): Promise<DomainCommunicationSavedSearch> {
  const response = await getCommunicationsConnectClient().createSavedSearch({
    name: request.name,
    description: request.description ?? undefined,
    accountId: request.account_id ?? undefined,
    query: request.query ?? undefined,
    workflowState: request.workflow_state ?? undefined,
    localState: request.local_state ?? undefined,
    channelKind: request.channel_kind ?? undefined,
    isSmartFolder: request.is_smart_folder ?? undefined,
    sortOrder: request.sort_order ?? undefined
  })
  return mapSavedSearchItem(response.item)
}

export async function updateCommunicationSavedSearchConnect(
  savedSearchId: string,
  request: SavedSearchUpdate
): Promise<DomainCommunicationSavedSearch> {
  const response = await getCommunicationsConnectClient().updateSavedSearch({
    savedSearchId,
    name: request.name ?? undefined,
    description: request.description ?? undefined,
    accountId: request.account_id ?? undefined,
    query: request.query ?? undefined,
    workflowState: request.workflow_state ?? undefined,
    localState: request.local_state ?? undefined,
    channelKind: request.channel_kind ?? undefined,
    isSmartFolder: request.is_smart_folder ?? undefined,
    sortOrder: request.sort_order ?? undefined
  })
  return mapSavedSearchItem(response.item)
}

export async function deleteCommunicationSavedSearchConnect(
  savedSearchId: string
): Promise<SavedSearchDeleteResponse> {
  const response = await getCommunicationsConnectClient().deleteSavedSearch({ savedSearchId })
  return { deleted: response.deleted }
}

export async function fetchCommunicationOutboxConnect(
  accountId?: string,
  status?: string,
  limit?: number,
  cursor?: string
): Promise<OutboxListResponse> {
  const response = await getCommunicationsConnectClient().listOutbox({
    accountId,
    status,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapOutboxItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function fetchCommunicationFoldersConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<CommunicationFolderListResponse> {
  const response = await getCommunicationsConnectClient().listFolders({
    accountId,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapFolderItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function createCommunicationFolderConnect(
  request: CommunicationFolderInput
): Promise<CommunicationFolder> {
  const response = await getCommunicationsConnectClient().createFolder({
    folderId: request.folder_id ?? undefined,
    accountId: request.account_id ?? undefined,
    name: request.name,
    description: request.description ?? undefined,
    color: request.color ?? undefined,
    sortOrder: request.sort_order ?? undefined
  })
  return mapFolderItem(response.item)
}

export async function updateCommunicationFolderConnect(
  folderId: string,
  request: CommunicationFolderUpdate
): Promise<CommunicationFolder> {
  const response = await getCommunicationsConnectClient().updateFolder({
    folderId,
    accountId: request.account_id ?? undefined,
    name: request.name ?? undefined,
    description: request.description ?? undefined,
    color: request.color ?? undefined,
    sortOrder: request.sort_order ?? undefined
  })
  return mapFolderItem(response.item)
}

export async function deleteCommunicationFolderConnect(folderId: string): Promise<FolderDeleteResponse> {
  const response = await getCommunicationsConnectClient().deleteFolder({ folderId })
  return { deleted: response.deleted }
}

export async function fetchFolderMessagesConnect(
  folderId: string,
  limit?: number,
  cursor?: string
): Promise<FolderMessageListResponse> {
  const response = await getCommunicationsConnectClient().listFolderMessages({
    folderId,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapFolderMessageItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function copyMessageToFolderConnect(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  const response = await getCommunicationsConnectClient().copyMessageToFolder({ folderId, messageId })
  return mapFolderMessageActionResult(response.item)
}

export async function moveMessageToFolderConnect(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  const response = await getCommunicationsConnectClient().moveMessageToFolder({ folderId, messageId })
  return mapFolderMessageActionResult(response.item)
}

export async function createCommunicationDraftConnect(
  request: DraftUpsertRequest
): Promise<CommunicationDraft> {
  const response = await getCommunicationsConnectClient().createDraft({
    draftId: request.draft_id,
    accountId: request.account_id,
    personaId: request.persona_id ?? undefined,
    toRecipients: request.to_recipients,
    ccRecipients: request.cc_recipients ?? [],
    bccRecipients: request.bcc_recipients ?? [],
    subject: request.subject,
    bodyText: request.body_text,
    bodyHtml: request.body_html ?? undefined,
    inReplyTo: request.in_reply_to ?? undefined,
    references: request.references ?? [],
    attachmentIds: request.attachment_ids ?? [],
    replaceAttachments: request.attachment_ids !== undefined,
    status: request.status ?? undefined,
    scheduledSendAt: request.scheduled_send_at ?? undefined,
    metadataJson: JSON.stringify(request.metadata ?? {})
  })
  return mapDraftItem(response.item)
}

export async function deleteCommunicationDraftConnect(draftId: string): Promise<DraftDeleteResponse> {
  const response = await getCommunicationsConnectClient().deleteDraft({ draftId })
  return { deleted: response.deleted }
}

export async function undoCommunicationOutboxItemConnect(outboxId: string): Promise<CommunicationOutboxItem> {
  const response = await getCommunicationsConnectClient().undoOutboxItem({ outboxId })
  return mapOutboxItem(response.item)
}

export async function sendCommunicationConnect(
  request: SendCommunicationRequest
): Promise<SendCommunicationResponse> {
  const response = await getCommunicationsConnectClient().sendMessage({
    accountId: request.account_id,
    toRecipients: request.to,
    ccRecipients: request.cc ?? [],
    bccRecipients: request.bcc ?? [],
    subject: request.subject,
    bodyText: request.body_text,
    bodyHtml: request.body_html ?? undefined,
    scheduledSendAt: request.scheduled_send_at ?? undefined,
    metadataJson: JSON.stringify({}),
    inReplyTo: request.in_reply_to ?? undefined,
    references: request.references ?? [],
    draftId: request.draft_id ?? undefined,
    undoSendSeconds: request.undo_send_seconds != null ? BigInt(request.undo_send_seconds) : undefined,
    confirmedProviderWrite: request.confirmed_provider_write
  })
  return {
    message_id: response.messageId,
    outbox_id: response.outboxId ?? null,
    accepted: response.accepted,
    accepted_recipients: response.acceptedRecipients,
    transport: response.transport,
    status: response.status,
    scheduled_send_at: response.scheduledSendAt ?? null,
    undo_deadline_at: response.undoDeadlineAt ?? null,
    failure_reason: response.failureReason ?? null
  }
}

export async function redirectMessageConnect(
  messageId: string,
  request: RedirectMessageRequest
): Promise<SendCommunicationResponse> {
  const response = await getCommunicationsConnectClient().redirectMessage({
    messageId,
    toRecipients: request.to,
    ccRecipients: request.cc ?? [],
    bccRecipients: request.bcc ?? [],
    confirmedProviderWrite: request.confirmed_provider_write ?? false
  })
  return {
    message_id: response.messageId,
    outbox_id: response.outboxId ?? null,
    accepted: response.accepted,
    accepted_recipients: response.acceptedRecipients,
    transport: response.transport,
    status: response.status,
    scheduled_send_at: response.scheduledSendAt ?? null,
    undo_deadline_at: response.undoDeadlineAt ?? null,
    failure_reason: response.failureReason ?? null
  }
}
