import { getCommunicationsConnectClient } from '../../../../platform/connect/communicationsClient'
import type {
  CommunicationArchitectureBlocker,
  CommunicationPersona,
  CommunicationSearchResponse,
  CommunicationTemplate,
  LanguageDetection,
  MailboxHealth,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse,
  SenderStatsListResponse,
  SubscriptionListResponse,
  TranslationResponse,
  WorkflowStateCountsResponse
} from '../../types/communications'
import { mapRichTemplate, parseJsonObject, toNumber } from './mapping'
import { postCommunicationsConnectJson } from './shared'

export async function fetchMessageStateCountsConnect(
  accountId?: string,
  localState?: string
): Promise<WorkflowStateCountsResponse> {
  const response = await getCommunicationsConnectClient().listMessageWorkflowStateCounts({
    accountId,
    localState
  })

  return {
    counts: response.counts.map((item) => ({
      state: item.state,
      count: toNumber(item.count)
    }))
  }
}

export async function fetchSubscriptionsConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<SubscriptionListResponse> {
  const response = await getCommunicationsConnectClient().listSubscriptions({
    accountId,
    cursor,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      sender: item.sender,
      message_count: toNumber(item.messageCount),
      first_seen: item.firstSeen,
      last_seen: item.lastSeen,
      is_newsletter: item.isNewsletter,
      has_unsubscribe: item.hasUnsubscribe
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchMailboxHealthConnect(accountId?: string): Promise<MailboxHealth> {
  const response = await getCommunicationsConnectClient().getMailboxHealth({ accountId })
  return {
    total_messages: toNumber(response.item?.totalMessages ?? 0),
    unread: toNumber(response.item?.unread ?? 0),
    needs_action: toNumber(response.item?.needsAction ?? 0),
    waiting: toNumber(response.item?.waiting ?? 0),
    done: toNumber(response.item?.done ?? 0),
    archived: toNumber(response.item?.archived ?? 0),
    spam: toNumber(response.item?.spam ?? 0),
    important: toNumber(response.item?.important ?? 0),
    with_attachments: toNumber(response.item?.withAttachments ?? 0),
    average_importance: response.item?.averageImportance ?? 0,
    oldest_message_days: response.item?.oldestMessageDays ?? null
  }
}

export async function fetchTopSendersConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<SenderStatsListResponse> {
  const response = await getCommunicationsConnectClient().listTopSenders({
    accountId,
    cursor,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      sender: item.sender,
      message_count: toNumber(item.messageCount),
      avg_importance: item.avgImportance,
      last_message_days: item.lastMessageDays ?? null
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchCommunicationBlockersConnect(): Promise<CommunicationArchitectureBlocker[]> {
  const response = await getCommunicationsConnectClient().listCommunicationBlockers({})
  return response.items.map((item) => ({
    section: item.section,
    feature: item.feature,
    reason: item.reason,
    resolution: item.resolution
  }))
}

export async function fetchCommunicationPersonasConnect(): Promise<{ items: CommunicationPersona[] }> {
  const response = await postCommunicationsConnectJson<{
    items: Array<{
      personaId: string
      accountId: string
      name: string
      displayName: string
      signature: string
      defaultLanguage?: string
      defaultTone?: string
      isDefault: boolean
      metadataJson: string
      createdAt: string
      updatedAt: string
    }>
  }>('ListCommunicationPersonas', {})

  return {
    items: response.items.map((item) => ({
      persona_id: item.personaId,
      account_id: item.accountId,
      name: item.name,
      display_name: item.displayName,
      signature: item.signature,
      default_language: item.defaultLanguage ?? null,
      default_tone: item.defaultTone ?? null,
      is_default: item.isDefault,
      metadata: parseJsonObject(item.metadataJson),
      created_at: item.createdAt,
      updated_at: item.updatedAt
    }))
  }
}

export async function fetchRichTemplatesConnect(): Promise<{ templates: CommunicationTemplate[] }> {
  const response = await postCommunicationsConnectJson<{
    templates: Array<Parameters<typeof mapRichTemplate>[0]>
  }>('ListRichTemplates', {})
  return { templates: response.templates.map(mapRichTemplate) }
}

export async function saveRichTemplateConnect(
  request: RichTemplateUpsertRequest
): Promise<RichTemplateUpsertResponse> {
  const response = await postCommunicationsConnectJson<{
    saved: boolean
    template?: Parameters<typeof mapRichTemplate>[0]
  }>('UpsertRichTemplate', {
    templateId: request.template_id ?? undefined,
    name: request.name,
    subjectTemplate: request.subject_template,
    bodyTemplate: request.body_template,
    variables: request.variables,
    language: request.language ?? ''
  })

  return {
    saved: response.saved,
    template: mapRichTemplate(response.template)
  }
}

export async function deleteRichTemplateConnect(
  templateId: string
): Promise<RichTemplateDeleteResponse> {
  const response = await postCommunicationsConnectJson<{ templateId: string; deleted: boolean }>(
    'DeleteRichTemplate',
    { templateId }
  )
  return { template_id: response.templateId, deleted: response.deleted }
}

export async function renderRichTemplateConnect(
  request: RichTemplateRenderRequest
): Promise<RichTemplateRenderResponse> {
  const response = await postCommunicationsConnectJson<{
    templateId: string
    variables: Record<string, string>
    rendered?: {
      subject: string
      body: string
      missingVariables: string[]
      unresolvedVariables: string[]
      malformedPlaceholders: string[]
    }
  }>('RenderRichTemplate', {
    templateId: request.template_id,
    variables: request.variables
  })

  return {
    template_id: response.templateId,
    variables: response.variables,
    rendered: {
      subject: response.rendered?.subject ?? '',
      body: response.rendered?.body ?? '',
      missing_variables: response.rendered?.missingVariables ?? [],
      unresolved_variables: response.rendered?.unresolvedVariables ?? [],
      malformed_placeholders: response.rendered?.malformedPlaceholders ?? []
    }
  }
}

export async function previewRichTemplateMailMergeConnect(
  request: RichTemplateMailMergePreviewRequest
): Promise<RichTemplateMailMergePreviewResponse> {
  const response = await postCommunicationsConnectJson<{
    templateId: string
    rowCount: number | bigint
    readyCount: number | bigint
    blockedCount: number | bigint
    items: Array<{
      rowId: string
      ready: boolean
      rendered?: {
        subject: string
        body: string
        missingVariables: string[]
        unresolvedVariables: string[]
        malformedPlaceholders: string[]
      }
    }>
  }>('PreviewRichTemplateMailMerge', {
    templateId: request.template_id,
    rows: request.rows.map((row) => ({ rowId: row.row_id, variables: row.variables }))
  })

  return {
    template_id: response.templateId,
    row_count: toNumber(response.rowCount),
    ready_count: toNumber(response.readyCount),
    blocked_count: toNumber(response.blockedCount),
    items: response.items.map((item) => ({
      row_id: item.rowId,
      ready: item.ready,
      rendered: {
        subject: item.rendered?.subject ?? '',
        body: item.rendered?.body ?? '',
        missing_variables: item.rendered?.missingVariables ?? [],
        unresolved_variables: item.rendered?.unresolvedVariables ?? [],
        malformed_placeholders: item.rendered?.malformedPlaceholders ?? []
      }
    }))
  }
}

export async function searchMessagesConnect(
  query: string,
  limit?: number
): Promise<CommunicationSearchResponse> {
  const response = await getCommunicationsConnectClient().searchMessages({ query, limit: limit ?? 0 })
  return {
    results: response.results.map((item) => ({
      object_id: item.objectId,
      object_kind: item.objectKind,
      title: item.title
    }))
  }
}

export async function detectMessageLanguageConnect(messageId: string): Promise<LanguageDetection> {
  const response = await getCommunicationsConnectClient().detectMessageLanguage({ messageId })
  return {
    language: response.language,
    confidence: response.confidence,
    script: response.script ?? null
  }
}

export async function translateMessageConnect(
  messageId: string,
  targetLanguage: string
): Promise<TranslationResponse> {
  const response = await getCommunicationsConnectClient().translateMessage({
    messageId,
    targetLanguage
  })
  return {
    translated: response.translated,
    text: response.text ?? undefined,
    target: response.target ?? undefined,
    model: response.model ?? undefined,
    reason: response.reason ?? undefined
  }
}
