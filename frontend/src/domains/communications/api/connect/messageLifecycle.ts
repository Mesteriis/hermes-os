import { getCommunicationsConnectClient } from "../../../../platform/connect/communicationsClient";
import type {
  AiReplyResponse,
  AiReplyVariantsResponse,
  BulkMessageActionRequest,
  BulkMessageActionResponse,
  CommunicationMessageDetailResponse,
  CommunicationMessagesResponse,
  ExtractNotesResponse,
  ExtractTasksResponse,
  LocalMessageStateResponse,
  MessageAnalyzeResponse,
  MessageAuthCheckResponse,
  MessageExplainResponse,
  MessageExportResponse,
  MessageImportantToggleResponse,
  MessagePinToggleResponse,
  SignatureDetection,
  SmartCcResponse,
  WorkflowActionRequest,
  WorkflowActionResponse,
  WorkflowState,
  WorkflowStateTransitionResponse,
} from "../../types/communications";
import type { MailReadSyncStatus } from "../../types/mailSync";
import {
  normalizeAiState,
  mapAttachment,
  mapMessageSummary,
  mapMessageSummaryContract,
  normalizeBulkMessageAction,
  normalizeLocalState,
  normalizeWorkflowState,
  parseJsonObject,
} from "./mapping";
import { postCommunicationsConnectJson } from "./shared";

export type ConnectCommunicationMessagesRequest = {
  account_id?: string;
  workflow_state?: string;
  is_read?: boolean;
  channel_kind?: string;
  conversation_id?: string;
  query?: string;
  match_mode?: "all" | "any";
  local_state?: string;
  cursor?: string;
  limit?: number;
};

export async function fetchCommunicationMessagesConnect(
  request: ConnectCommunicationMessagesRequest = {}
): Promise<CommunicationMessagesResponse> {
  const response = await getCommunicationsConnectClient().listMessages({
    accountId: request.account_id,
    workflowState: request.workflow_state,
    isRead: request.is_read,
    channelKind: request.channel_kind,
    conversationId: request.conversation_id,
    query: request.query,
    matchMode: request.match_mode,
    localState: request.local_state,
    cursor: request.cursor,
    limit: request.limit ?? 0,
  });

  return {
    items: response.items.map(mapMessageSummary),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore,
  };
}

export async function fetchCommunicationMessageConnect(
  messageId: string
): Promise<CommunicationMessageDetailResponse> {
  const response = await getCommunicationsConnectClient().getMessage({
    messageId,
  });

  return {
    message: {
      message_id: response.item?.messageId ?? messageId,
      raw_record_id: response.item?.rawRecordId ?? "",
      observation_id: response.item?.observationId ?? null,
      account_id: response.item?.accountId ?? "",
      provider_record_id: response.item?.providerRecordId ?? "",
      subject: response.item?.subject ?? "",
      sender: response.item?.sender ?? "",
      recipients: response.item?.recipients ?? [],
      body_text: response.item?.bodyText ?? "",
      body_html: response.item?.bodyHtml ?? null,
      occurred_at: response.item?.occurredAt ?? null,
      projected_at: response.item?.projectedAt ?? "",
      channel_kind: response.item?.channelKind ?? "",
      conversation_id: response.item?.conversationId ?? null,
      sender_display_name: response.item?.senderDisplayName ?? null,
      delivery_state: response.item?.deliveryState ?? "",
      workflow_state: normalizeWorkflowState(response.item?.workflowState),
      importance_score: response.item?.importanceScore ?? null,
      ai_category: response.item?.aiCategory ?? null,
      ai_summary: response.item?.aiSummary ?? null,
      ai_summary_generated_at: response.item?.aiSummaryGeneratedAt ?? null,
      ai_state: normalizeAiState(response.item?.aiState),
      message_metadata: parseJsonObject(response.item?.messageMetadataJson),
      local_state: normalizeLocalState(response.item?.localState),
      local_state_changed_at: response.item?.localStateChangedAt ?? null,
      local_state_reason: response.item?.localStateReason ?? null,
      is_read: response.item?.isRead ?? false,
      read_changed_at: response.item?.readChangedAt ?? null,
      read_origin: response.item?.readOrigin ?? "migration_inferred",
      read_sync_status: normalizeReadSyncStatus(response.item?.readSyncStatus),
    },
    attachments: response.attachments.map(mapAttachment),
  };
}

function normalizeReadSyncStatus(value?: string): MailReadSyncStatus {
  switch (value) {
    case 'queued':
    case 'syncing':
    case 'retrying':
    case 'failed':
    case 'awaiting_provider':
    case 'superseded':
      return value
    default:
      return 'synced'
  }
}

export async function transitionMessageWorkflowStateConnect(
  messageId: string,
  workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
  const response =
    await getCommunicationsConnectClient().transitionMessageWorkflowState({
      messageId,
      workflowState,
    });

  return {
    message_id: response.messageId,
    workflow_state: normalizeWorkflowState(response.workflowState),
    previous_state: response.previousState,
  };
}

export async function trashMessageConnect(
  messageId: string
): Promise<LocalMessageStateResponse> {
  const response = await getCommunicationsConnectClient().trashMessage({
    messageId,
  });
  return {
    message_id: response.messageId,
    local_state: normalizeLocalState(response.localState),
    provider_deleted: response.providerDeleted ?? undefined,
  };
}

export async function restoreMessageConnect(
  messageId: string
): Promise<LocalMessageStateResponse> {
  const response = await getCommunicationsConnectClient().restoreMessage({
    messageId,
  });
  return {
    message_id: response.messageId,
    local_state: normalizeLocalState(response.localState),
    provider_deleted: response.providerDeleted ?? undefined,
  };
}

export async function markMessageReadConnect(
  messageId: string
): Promise<Record<string, unknown>> {
  const response = await getCommunicationsConnectClient().markMessageRead({
    messageId,
  });
  return {
    message_id: response.messageId,
    marked_read: response.markedRead,
    workflow_state: normalizeWorkflowState(response.workflowState),
  };
}

export async function deleteMessageFromProviderConnect(
  messageId: string
): Promise<LocalMessageStateResponse> {
  const response =
    await getCommunicationsConnectClient().deleteMessageFromProvider({
      messageId,
    });
  return {
    message_id: response.messageId,
    local_state: normalizeLocalState(response.localState),
    provider_deleted: response.providerDeleted ?? undefined,
  };
}

export async function bulkMessageActionConnect(
  request: BulkMessageActionRequest
): Promise<BulkMessageActionResponse> {
  const response = await getCommunicationsConnectClient().bulkMessageAction({
    action: request.action,
    messageIds: request.message_ids,
    label: request.label ?? undefined,
    snoozeUntil: request.snooze_until ?? undefined,
  });

  return {
    action: normalizeBulkMessageAction(response.action),
    requested_count: Number(response.requestedCount),
    matched_count: Number(response.matchedCount),
    updated_count: Number(response.updatedCount),
    not_found: response.notFound,
  };
}

export async function toggleMessagePinConnect(
  messageId: string
): Promise<MessagePinToggleResponse> {
  const response = await getCommunicationsConnectClient().toggleMessagePin({
    messageId,
  });
  return { message_id: response.messageId, pinned: response.pinned };
}

export async function toggleMessageImportantConnect(
  messageId: string
): Promise<MessageImportantToggleResponse> {
  const response =
    await getCommunicationsConnectClient().toggleMessageImportant({
      messageId,
    });
  return { message_id: response.messageId, important: response.important };
}

export async function toggleMessageMuteConnect(
  messageId: string
): Promise<MessagePinToggleResponse> {
  const response = await getCommunicationsConnectClient().toggleMessageMute({
    messageId,
  });
  return {
    message_id: response.messageId,
    pinned: response.muted,
  };
}

export async function snoozeMessageConnect(
  messageId: string,
  snoozeUntil: string
): Promise<Record<string, unknown>> {
  return postCommunicationsConnectJson<Record<string, unknown>>(
    "SnoozeMessage",
    {
      messageId,
      until: snoozeUntil,
    }
  );
}

export async function addMessageLabelConnect(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return postCommunicationsConnectJson<Record<string, unknown>>(
    "AddMessageLabel",
    {
      messageId,
      label,
    }
  );
}

export async function removeMessageLabelConnect(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return postCommunicationsConnectJson<Record<string, unknown>>(
    "RemoveMessageLabel",
    {
      messageId,
      label,
    }
  );
}

export async function analyzeMessageConnect(
  messageId: string
): Promise<MessageAnalyzeResponse> {
  const response = await getCommunicationsConnectClient().analyzeMessage({
    messageId,
  });
  return {
    message_id: response.messageId,
    analyzed: response.analyzed,
    category: response.category ?? null,
    summary: response.summary ?? null,
    summary_contract: mapMessageSummaryContract(response.summaryContract),
    importance_score: response.importanceScore ?? null,
    workflow_state: response.workflowState,
    source: response.source,
    confidence: response.confidence ?? null,
    evidence: response.evidence,
  };
}

export async function runWorkflowActionConnect(
  request: WorkflowActionRequest
): Promise<WorkflowActionResponse> {
  const response = await postCommunicationsConnectJson<{
    commandId: string;
    eventId: string;
    action: string;
    status: "created" | "updated" | "linked" | "opened" | "archived" | "noop";
    target?: {
      kind:
        | "compose"
        | "message"
        | "task"
        | "document"
        | "calendar_event"
        | "persona";
      id?: string;
    };
    provenance?: {
      sourceKind?: string;
      sourceId?: string;
      confidence?: number;
      evidence: string[];
    };
  }>("RunWorkflowAction", {
    commandId: request.command_id,
    action: request.action,
    source: request.source
      ? { kind: request.source.kind, id: request.source.id }
      : undefined,
    input: request.input
      ? {
          title: request.input.title,
          body: request.input.body,
          email: request.input.email,
          displayName: request.input.display_name,
          startsAt: request.input.starts_at,
          endsAt: request.input.ends_at,
          dueAt: request.input.due_at,
          documentId: request.input.document_id,
        }
      : undefined,
  });

  return {
    command_id: response.commandId,
    event_id: response.eventId,
    action: response.action as WorkflowActionResponse["action"],
    status: response.status,
    target: {
      kind: response.target?.kind ?? "message",
      id: response.target?.id ?? null,
    },
    provenance: {
      source_kind: response.provenance?.sourceKind,
      source_id: response.provenance?.sourceId,
      confidence: response.provenance?.confidence ?? null,
      evidence: response.provenance?.evidence ?? [],
    },
  };
}

export async function fetchMessageExplainConnect(
  messageId: string
): Promise<MessageExplainResponse> {
  const response = await getCommunicationsConnectClient().getMessageExplain({
    messageId,
  });
  return { reasons: response.reasons };
}

export async function fetchMessageSmartCcConnect(
  messageId: string
): Promise<SmartCcResponse> {
  const response = await getCommunicationsConnectClient().getMessageSmartCc({
    messageId,
  });
  return { suggestions: response.suggestions };
}

export async function exportMessageConnect(
  messageId: string,
  format: "md" | "eml" | "json"
): Promise<MessageExportResponse> {
  const response = await getCommunicationsConnectClient().getMessageExport({
    messageId,
    format,
  });
  return {
    content_type: response.contentType,
    content: response.content,
    filename: response.filename,
  };
}

export async function fetchMessageAuthConnect(
  messageId: string
): Promise<MessageAuthCheckResponse> {
  const response = await getCommunicationsConnectClient().getMessageAuth({
    messageId,
  });
  return {
    auth: {
      spf: response.auth?.spf
        ? {
            result: response.auth.spf.result,
            domain: response.auth.spf.domain ?? null,
            ip: response.auth.spf.ip ?? null,
            selector: response.auth.spf.selector ?? null,
            policy: response.auth.spf.policy ?? null,
          }
        : null,
      dkim: response.auth?.dkim
        ? {
            result: response.auth.dkim.result,
            domain: response.auth.dkim.domain ?? null,
            ip: response.auth.dkim.ip ?? null,
            selector: response.auth.dkim.selector ?? null,
            policy: response.auth.dkim.policy ?? null,
          }
        : null,
      dmarc: response.auth?.dmarc
        ? {
            result: response.auth.dmarc.result,
            domain: response.auth.dmarc.domain ?? null,
            ip: response.auth.dmarc.ip ?? null,
            selector: response.auth.dmarc.selector ?? null,
            policy: response.auth.dmarc.policy ?? null,
          }
        : null,
      raw_headers: response.auth?.rawHeaders ?? [],
    },
    risk: {
      has_spf: response.risk?.hasSpf ?? false,
      spf_pass: response.risk?.spfPass ?? false,
      has_dkim: response.risk?.hasDkim ?? false,
      dkim_pass: response.risk?.dkimPass ?? false,
      has_dmarc: response.risk?.hasDmarc ?? false,
      dmarc_pass: response.risk?.dmarcPass ?? false,
      is_spoofed: response.risk?.isSpoofed ?? false,
      risk_summary: response.risk?.riskSummary ?? "",
    },
  };
}

export async function fetchMessageSignatureConnect(
  messageId: string
): Promise<SignatureDetection> {
  const response = await getCommunicationsConnectClient().getMessageSignature({
    messageId,
  });
  return {
    has_signature: response.hasSignature,
    signature_type: response.signatureType ?? null,
    signer_info: response.signerInfo ?? null,
    is_valid: response.isValid ?? null,
    cert_expiry_warning: response.certExpiryWarning ?? null,
  };
}

export async function generateAiReplyConnect(
  messageId: string,
  request: { tone?: string; language?: string; context?: string } = {}
): Promise<AiReplyResponse> {
  const response = await postCommunicationsConnectJson<{
    subject?: string;
    body?: string;
    tone?: string;
    language?: string;
    generated?: boolean;
    reason?: string;
  }>("GenerateAiReply", { messageId, ...request });
  return {
    subject: response.subject ?? undefined,
    body: response.body ?? undefined,
    tone: response.tone ?? undefined,
    language: response.language ?? undefined,
    generated: response.generated,
    reason: response.reason ?? undefined,
  };
}

export async function generateAiReplyVariantsConnect(
  messageId: string,
  request: { languages?: string[]; tones?: string[] } = {}
): Promise<AiReplyVariantsResponse> {
  const response = await postCommunicationsConnectJson<{
    variants: Array<{
      subject?: string;
      body?: string;
      tone?: string;
      language?: string;
      generated?: boolean;
      reason?: string;
    }>;
  }>("GenerateAiReplyVariants", { messageId, ...request });
  return {
    variants: response.variants.map((item) => ({
      subject: item.subject ?? undefined,
      body: item.body ?? undefined,
      tone: item.tone ?? undefined,
      language: item.language ?? undefined,
      generated: item.generated ?? undefined,
      reason: item.reason ?? undefined,
    })),
  };
}

export async function extractMessageTasksConnect(
  messageId: string
): Promise<ExtractTasksResponse> {
  const response = await getCommunicationsConnectClient().extractMessageTasks({
    messageId,
  });
  return {
    tasks: response.tasks.map((task) => ({
      title: task.title,
      due_date: task.dueDate ?? null,
      assignee: task.assignee ?? null,
      priority: task.priority ?? null,
      source: task.source,
    })),
  };
}

export async function extractMessageNotesConnect(
  messageId: string
): Promise<ExtractNotesResponse> {
  const response = await getCommunicationsConnectClient().extractMessageNotes({
    messageId,
  });
  return {
    notes: response.notes.map((note) => ({
      title: note.title,
      content: note.content,
      tags: note.tags,
      source: note.source,
    })),
  };
}
