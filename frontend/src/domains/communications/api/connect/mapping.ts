import type {
  BulkMessageActionResponse,
  CommunicationAttachment,
  CommunicationDraft,
  CommunicationKnowledgeCandidate,
  CommunicationMessageSummary,
  CommunicationOutboxItem,
  CommunicationTemplate,
  DraftListResponse,
  MessageAnalyzeResponse,
} from "../../types/communications";
import type { CommunicationAiState } from "../../types/aiState";
import type {
  AttachmentScanStatus,
  AttachmentSearchResponse,
} from "../../types/attachments";
import type { CommunicationSavedSearch as DomainCommunicationSavedSearch } from "../../types/savedSearches";
import type {
  CommunicationFolder,
  FolderMessage,
  FolderMessageActionResponse,
} from "../../types/folders";

export function mapMessageSummary(item: {
  messageId: string;
  rawRecordId: string;
  observationId?: string;
  accountId: string;
  providerRecordId: string;
  subject: string;
  sender: string;
  recipients: string[];
  bodyText: string;
  occurredAt?: string;
  projectedAt: string;
  channelKind: string;
  conversationId?: string;
  senderDisplayName?: string;
  deliveryState: string;
  messageMetadataJson: string;
  workflowState: string;
  importanceScore?: number;
  aiCategory?: string;
  aiSummary?: string;
  aiSummaryGeneratedAt?: string;
  aiState?: string;
  localState: string;
  localStateChangedAt?: string;
  isRead: boolean;
  readChangedAt?: string;
  readOrigin: string;
  readSyncStatus: string;
  attachmentCount: number | bigint;
}): CommunicationMessageSummary {
  return {
    message_id: item.messageId,
    raw_record_id: item.rawRecordId,
    observation_id: item.observationId ?? null,
    account_id: item.accountId,
    provider_record_id: item.providerRecordId,
    subject: item.subject,
    sender: item.sender,
    recipients: item.recipients,
    body_text_preview: textPreview(item.bodyText, 240),
    occurred_at: item.occurredAt ?? null,
    projected_at: item.projectedAt,
    channel_kind: item.channelKind,
    conversation_id: item.conversationId ?? null,
    sender_display_name: item.senderDisplayName ?? null,
    delivery_state: item.deliveryState,
    workflow_state: normalizeWorkflowState(item.workflowState),
    importance_score: item.importanceScore ?? null,
    ai_category: item.aiCategory ?? null,
    ai_summary: item.aiSummary ?? null,
    ai_summary_generated_at: item.aiSummaryGeneratedAt ?? null,
    ai_state: normalizeAiState(item.aiState),
    message_metadata: parseJsonObject(item.messageMetadataJson),
    attachment_count: toNumber(item.attachmentCount),
    local_state: normalizeLocalState(item.localState),
    local_state_changed_at: item.localStateChangedAt ?? null,
    is_read: item.isRead,
    read_changed_at: item.readChangedAt ?? null,
    read_origin: item.readOrigin,
    read_sync_status: normalizeReadSyncStatus(item.readSyncStatus),
  };
}

function normalizeReadSyncStatus(value: string): CommunicationMessageSummary['read_sync_status'] {
  switch (value) {
    case 'queued':
    case 'syncing':
    case 'retrying':
    case 'failed':
    case 'awaiting_provider':
    case 'synced':
    case 'superseded':
      return value
    default:
      return 'synced'
  }
}

export function mapAttachment(item: {
  attachmentId: string;
  messageId: string;
  rawRecordId: string;
  blobId: string;
  providerAttachmentId: string;
  filename?: string;
  contentType: string;
  sizeBytes: number | bigint;
  sha256: string;
  disposition: string;
  scanStatus: string;
  scanEngine?: string;
  scanCheckedAt?: string;
  scanSummary?: string;
  scanMetadataJson: string;
  storageKind: string;
  storagePath: string;
  createdAt: string;
  updatedAt: string;
}): CommunicationAttachment {
  return {
    attachment_id: item.attachmentId,
    message_id: item.messageId,
    raw_record_id: item.rawRecordId,
    blob_id: item.blobId,
    provider_attachment_id: item.providerAttachmentId,
    filename: item.filename ?? null,
    content_type: item.contentType,
    size_bytes: toNumber(item.sizeBytes),
    sha256: item.sha256,
    disposition: normalizeDisposition(item.disposition),
    scan_status: normalizeScanStatus(item.scanStatus),
    scan_engine: item.scanEngine ?? null,
    scan_checked_at: item.scanCheckedAt ?? null,
    scan_summary: item.scanSummary ?? null,
    scan_metadata: parseJsonObject(item.scanMetadataJson),
    storage_kind: item.storageKind,
    storage_path: item.storagePath,
    created_at: item.createdAt,
    updated_at: item.updatedAt,
  };
}

export function mapOutboxItem(
  item:
    | {
        outboxId: string;
        accountId: string;
        draftId?: string;
        toRecipients: string[];
        ccRecipients: string[];
        bccRecipients: string[];
        subject: string;
        bodyText: string;
        bodyHtml?: string;
        status: string;
        scheduledSendAt?: string;
        undoDeadlineAt?: string;
        sendAttempts: number;
        claimedAt?: string;
        sentAt?: string;
        providerMessageId?: string;
        lastError?: string;
        metadataJson: string;
        createdAt: string;
        updatedAt: string;
      }
    | undefined
): CommunicationOutboxItem {
  if (!item) {
    throw new Error("CommunicationsService returned an empty outbox item");
  }
  return {
    outbox_id: item.outboxId,
    account_id: item.accountId,
    draft_id: item.draftId ?? null,
    to_recipients: item.toRecipients,
    cc_recipients: item.ccRecipients,
    bcc_recipients: item.bccRecipients,
    subject: item.subject,
    body_text: item.bodyText,
    body_html: item.bodyHtml ?? null,
    status: normalizeOutboxStatus(item.status),
    scheduled_send_at: item.scheduledSendAt ?? null,
    undo_deadline_at: item.undoDeadlineAt ?? null,
    send_attempts: item.sendAttempts,
    claimed_at: item.claimedAt ?? null,
    sent_at: item.sentAt ?? null,
    provider_message_id: item.providerMessageId ?? null,
    last_error: item.lastError ?? null,
    metadata: parseJsonObject(item.metadataJson),
    created_at: item.createdAt,
    updated_at: item.updatedAt,
  };
}

export function mapDraftItem(
  item:
    | {
        draftId: string;
        accountId: string;
        personaId?: string;
        toRecipients: string[];
        ccRecipients: string[];
        bccRecipients: string[];
        subject: string;
        bodyText: string;
        bodyHtml?: string;
        inReplyTo?: string;
        references: string[];
        attachmentIds: string[];
        attachments?: Array<{
          attachmentId: string;
          filename?: string;
          contentType: string;
          sizeBytes: number | bigint;
          scanStatus: string;
          scanEngine?: string;
          scanCheckedAt?: string;
          scanSummary?: string;
        }>;
        status: string;
        scheduledSendAt?: string;
        sendAttempts: number;
        lastError?: string;
        metadataJson: string;
        createdAt: string;
        updatedAt: string;
      }
    | undefined
): CommunicationDraft {
  if (!item) {
    throw new Error("CommunicationsService returned an empty draft item");
  }
  return {
    draft_id: item.draftId,
    account_id: item.accountId,
    persona_id: item.personaId ?? null,
    to_recipients: item.toRecipients,
    cc_recipients: item.ccRecipients,
    bcc_recipients: item.bccRecipients,
    subject: item.subject,
    body_text: item.bodyText,
    body_html: item.bodyHtml ?? null,
    in_reply_to: item.inReplyTo ?? null,
    references: item.references,
    attachment_ids: item.attachmentIds,
    attachments: (item.attachments ?? []).map((attachment) => ({
      attachment_id: attachment.attachmentId,
      filename: attachment.filename ?? null,
      content_type: attachment.contentType,
      size_bytes: toNumber(attachment.sizeBytes),
      scan_status: attachment.scanStatus,
      scan_engine: attachment.scanEngine ?? null,
      scan_checked_at: attachment.scanCheckedAt ?? null,
      scan_summary: attachment.scanSummary ?? null,
    })),
    status: normalizeDraftStatus(item.status),
    scheduled_send_at: item.scheduledSendAt ?? null,
    send_attempts: item.sendAttempts,
    last_error: item.lastError ?? null,
    metadata: parseJsonObject(item.metadataJson),
    created_at: item.createdAt,
    updated_at: item.updatedAt,
  };
}

export function mapSavedSearchItem(
  item:
    | {
        savedSearchId: string;
        name: string;
        description?: string;
        accountId?: string;
        query: string;
        workflowState?: string;
        localState: string;
        channelKind?: string;
        isSmartFolder: boolean;
        sortOrder: number;
        messageCount: number | bigint;
        createdAt: string;
        updatedAt: string;
      }
    | undefined
): DomainCommunicationSavedSearch {
  if (!item) {
    throw new Error(
      "CommunicationsService returned an empty saved search item"
    );
  }
  return {
    saved_search_id: item.savedSearchId,
    name: item.name,
    description: item.description ?? null,
    account_id: item.accountId ?? null,
    query: item.query,
    workflow_state: item.workflowState
      ? normalizeWorkflowState(item.workflowState)
      : null,
    local_state: normalizeLocalState(item.localState),
    channel_kind: item.channelKind ?? null,
    is_smart_folder: item.isSmartFolder,
    sort_order: item.sortOrder,
    message_count: toNumber(item.messageCount),
    created_at: item.createdAt,
    updated_at: item.updatedAt,
  };
}

export function mapFolderItem(
  item:
    | {
        folderId: string;
        accountId?: string;
        name: string;
        description?: string;
        color?: string;
        sortOrder: number;
        messageCount: number | bigint;
        createdAt: string;
        updatedAt: string;
      }
    | undefined
): CommunicationFolder {
  if (!item) {
    throw new Error("CommunicationsService returned an empty folder item");
  }
  return {
    folder_id: item.folderId,
    account_id: item.accountId ?? null,
    name: item.name,
    description: item.description ?? null,
    color: item.color ?? null,
    sort_order: item.sortOrder,
    message_count: toNumber(item.messageCount),
    created_at: item.createdAt,
    updated_at: item.updatedAt,
  };
}

export function mapFolderMessageItem(
  item:
    | {
        folderId: string;
        messageId: string;
        accountId: string;
        subject: string;
        sender: string;
        occurredAt?: string;
        projectedAt: string;
        workflowState: string;
        localState: string;
        addedAt: string;
        attachmentCount: number | bigint;
      }
    | undefined
): FolderMessage {
  if (!item) {
    throw new Error(
      "CommunicationsService returned an empty folder message item"
    );
  }
  return {
    folder_id: item.folderId,
    message_id: item.messageId,
    account_id: item.accountId,
    subject: item.subject,
    sender: item.sender,
    occurred_at: item.occurredAt ?? null,
    projected_at: item.projectedAt,
    workflow_state: normalizeWorkflowState(item.workflowState),
    local_state: normalizeLocalState(item.localState),
    added_at: item.addedAt,
    attachment_count: toNumber(item.attachmentCount),
  };
}

export function mapFolderMessageActionResult(
  item:
    | {
        operation: string;
        folderId: string;
        messageId: string;
        message?: {
          folderId: string;
          messageId: string;
          accountId: string;
          subject: string;
          sender: string;
          occurredAt?: string;
          projectedAt: string;
          workflowState: string;
          localState: string;
          addedAt: string;
          attachmentCount: number | bigint;
        };
      }
    | undefined
): FolderMessageActionResponse {
  if (!item) {
    throw new Error(
      "CommunicationsService returned an empty folder action result"
    );
  }
  return {
    operation: item.operation === "move" ? "move" : "copy",
    folder_id: item.folderId,
    message_id: item.messageId,
    message: mapFolderMessageItem(item.message),
  };
}

export function mapRichTemplate(
  item:
    | {
        templateId: string;
        name: string;
        subjectTemplate: string;
        bodyTemplate: string;
        variables: string[];
        placeholderVariables: string[];
        undeclaredVariables: string[];
        unusedVariables: string[];
        malformedPlaceholders: string[];
        language?: string;
        createdAt: string;
        updatedAt: string;
      }
    | undefined
): CommunicationTemplate {
  if (!item) {
    throw new Error(
      "CommunicationsService returned an empty rich template item"
    );
  }
  return {
    template_id: item.templateId,
    name: item.name,
    subject_template: item.subjectTemplate,
    body_template: item.bodyTemplate,
    variables: item.variables,
    placeholder_variables: item.placeholderVariables,
    undeclared_variables: item.undeclaredVariables,
    unused_variables: item.unusedVariables,
    malformed_placeholders: item.malformedPlaceholders,
    language: item.language ?? null,
    created_at: item.createdAt,
    updated_at: item.updatedAt,
  };
}

export function mapMessageSummaryContract(
  item:
    | {
        keyPoints: string[];
        actionItems: string[];
        risks: string[];
        deadlines: string[];
        eventCandidates: { title: string; evidence: string }[];
        personaCandidates: { title: string; evidence: string }[];
        organizationCandidates: { title: string; evidence: string }[];
        documentCandidates: { title: string; evidence: string }[];
        agreementCandidates: { title: string; evidence: string }[];
      }
    | undefined
): MessageAnalyzeResponse["summary_contract"] {
  return {
    key_points: item?.keyPoints ?? [],
    action_items: item?.actionItems ?? [],
    risks: item?.risks ?? [],
    deadlines: item?.deadlines ?? [],
    event_candidates: mapKnowledgeCandidates(item?.eventCandidates),
    persona_candidates: mapKnowledgeCandidates(item?.personaCandidates),
    organization_candidates: mapKnowledgeCandidates(
      item?.organizationCandidates
    ),
    document_candidates: mapKnowledgeCandidates(item?.documentCandidates),
    agreement_candidates: mapKnowledgeCandidates(item?.agreementCandidates),
  };
}

export function mapKnowledgeCandidates(
  items: { title: string; evidence: string }[] | undefined
): CommunicationKnowledgeCandidate[] {
  return (items ?? []).map((item) => ({
    title: item.title,
    evidence: item.evidence,
  }));
}

export function parseJsonObject(
  value: string | undefined
): Record<string, unknown> {
  if (!value || value.trim().length === 0) {
    return {};
  }
  try {
    const parsed: unknown = JSON.parse(value);
    return isJsonObject(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

function isJsonObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function textPreview(value: string, limit: number): string {
  const normalized = value.trim();
  if (normalized.length <= limit) {
    return normalized;
  }
  return `${normalized.slice(0, limit).trimEnd()}...`;
}

export function normalizeWorkflowState(
  value: string | undefined
): CommunicationMessageSummary["workflow_state"] {
  switch (value) {
    case "reviewed":
    case "needs_action":
    case "waiting":
    case "done":
    case "archived":
    case "muted":
    case "spam":
      return value;
    case "new":
    default:
      return "new";
  }
}

export function normalizeLocalState(
  value: string | undefined
): CommunicationMessageSummary["local_state"] {
  switch (value) {
    case "trash":
    case "all":
      return value;
    case "active":
    default:
      return "active";
  }
}

export function normalizeAiState(
  value: string | undefined
): CommunicationAiState | null {
  switch (value) {
    case "NEW":
    case "PROCESSING":
    case "PROCESSED":
    case "REVIEW_REQUIRED":
    case "FAILED":
    case "ARCHIVED":
      return value;
    default:
      return null;
  }
}

export function normalizeDraftStatus(
  value: string
): DraftListResponse["items"][number]["status"] {
  switch (value) {
    case "scheduled":
    case "sending":
    case "sent":
    case "failed":
      return value;
    case "draft":
    default:
      return "draft";
  }
}

export function normalizeOutboxStatus(
  value: string
): CommunicationOutboxItem["status"] {
  switch (value) {
    case "scheduled":
    case "sending":
    case "sent":
    case "failed":
    case "canceled":
      return value;
    case "queued":
    default:
      return "queued";
  }
}

export function normalizeDisposition(
  value: string
): CommunicationAttachment["disposition"] {
  switch (value) {
    case "attachment":
    case "inline":
      return value;
    default:
      return "unknown";
  }
}

export function normalizeAttachmentDisposition(
  value: string
): AttachmentSearchResponse["items"][number]["disposition"] {
  switch (value) {
    case "attachment":
    case "inline":
      return value;
    default:
      return "unknown";
  }
}

export function normalizeScanStatus(
  value: string
): CommunicationAttachment["scan_status"] {
  switch (value) {
    case "clean":
    case "suspicious":
    case "malicious":
    case "failed":
      return value;
    case "not_scanned":
    default:
      return "not_scanned";
  }
}

export function normalizeAttachmentScanStatus(
  value: string
): AttachmentScanStatus {
  switch (value) {
    case "clean":
    case "suspicious":
    case "malicious":
    case "failed":
      return value;
    case "not_scanned":
    default:
      return "not_scanned";
  }
}

export function normalizeBulkMessageAction(
  value: string
): BulkMessageActionResponse["action"] {
  switch (value) {
    case "mark_read":
    case "mark_unread":
    case "archive":
    case "trash":
    case "restore":
    case "pin":
    case "unpin":
    case "important":
    case "not_important":
    case "star":
    case "unstar":
    case "add_label":
    case "remove_label":
    case "snooze":
      return value;
    default:
      return "trash";
  }
}

export function emptyStringToNull(value: string | undefined): string | null {
  if (!value || value.length === 0) {
    return null;
  }
  return value;
}

export function toNumber(value: number | bigint): number {
  return typeof value === "bigint" ? Number(value) : value;
}
