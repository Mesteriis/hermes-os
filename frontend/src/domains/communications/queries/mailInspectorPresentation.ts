import { aiSummaryContractFromMetadata } from '../helpers/communicationPageModels'
import type {
  CommunicationMessageDetailItem,
  CommunicationMessageSummary,
} from '../types/communications'
import type { CommunicationAiStateRecord } from '../types/aiState'
import type {
  MailInspectorEntityItem,
  MailInspectorModel,
} from '../components/mail/mailInspector'
import { mailInspectorFacts, mailInspectorTopics } from './mailInspectorFacts'

export function buildMailInspector(
  message: CommunicationMessageDetailItem | CommunicationMessageSummary | null,
  attachmentFallbackCount: number,
  aiState: CommunicationAiStateRecord | null
): MailInspectorModel {
  if (!message) return emptyMailInspector()

  const attachmentTotal = attachmentCount(message, attachmentFallbackCount)
  const importanceScore = message.importance_score ?? 0

  return {
    intelligence: {
      score: importanceScore,
      maxScore: 100,
      label: 'Projected importance',
      summary: mailInspectorSummary(message, aiState),
      checks: [
        {
          id: 'raw-record',
          label: 'Raw record',
          description: message.raw_record_id,
          icon: 'tabler:file-database',
          tone: 'neutral',
        },
        {
          id: 'provider-record',
          label: 'Provider record',
          description: message.provider_record_id,
          icon: 'tabler:mail-code',
          tone: 'info',
        },
        {
          id: 'attachments',
          label: 'Attachments',
          description: String(attachmentTotal),
          icon: 'tabler:paperclip',
          tone: attachmentTotal > 0 ? 'success' : 'neutral',
        },
        ...(aiState ? [mailAiStateCheck(aiState)] : []),
      ],
    },
    entityGroups: mailInspectorEntityGroups(message),
    topics: mailInspectorTopics(message),
    semanticFacts: mailInspectorFacts(message, attachmentTotal),
    suggestedActions: [],
    relatedContext: [],
  }
}

export function mailInspectorSummary(
  message: Pick<CommunicationMessageDetailItem | CommunicationMessageSummary, 'ai_summary' | 'message_metadata'>,
  aiState: CommunicationAiStateRecord | null = null
): string {
  const backendSummary = message.ai_summary?.trim()
  if (backendSummary) return backendSummary

  const contract = aiSummaryContractFromMetadata(message.message_metadata)
  const structuredSummary = [...contract?.key_points ?? [], ...contract?.action_items ?? []]
    .join(' ')
    .trim()
  if (structuredSummary) return structuredSummary

  if (aiState?.ai_state === 'PROCESSING') {
    return 'AI processing is in progress. Hermes will keep the local message state while the model finishes.'
  }
  if (aiState?.ai_state === 'REVIEW_REQUIRED') {
    if (aiState.review_reason === 'model_response_invalid') {
      return 'AI returned an unstructured response. Hermes kept only deterministic local triage; review the original message before promotion.'
    }
    return 'AI processing needs owner review before Hermes can promote any result.'
  }
  if (aiState?.ai_state === 'FAILED') {
    return aiState.next_attempt_at
      ? `AI retry is scheduled for ${aiState.next_attempt_at}.`
      : 'AI processing needs attention. Open Hermes actions to retry after fixing the model or route.'
  }
  if (aiState?.ai_state === 'NEW') {
    return 'AI analysis is queued and will run when the local worker is available.'
  }

  return 'No backend summary is available for this message.'
}

function attachmentCount(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary,
  fallbackCount: number
): number {
  if ('attachment_count' in source) return source.attachment_count
  return fallbackCount
}

function mailAiStateCheck(aiState: CommunicationAiStateRecord) {
  const details: string[] = [`state: ${aiState.ai_state}`]
  if (aiState.next_attempt_at) details.push(`retry: ${aiState.next_attempt_at}`)
  if (aiState.processing_lease_expires_at) {
    details.push(`lease: ${aiState.processing_lease_expires_at}`)
  }
  if (aiState.retry_count > 0) details.push(`attempts: ${aiState.retry_count}`)

  return {
    id: 'ai-lifecycle',
    label: 'AI lifecycle',
    description: details.join(' · '),
    icon: aiState.ai_state === 'PROCESSED' ? 'tabler:circle-check' : 'tabler:brain',
    tone: mailAiStateTone(aiState.ai_state),
  } satisfies MailInspectorModel['intelligence']['checks'][number]
}

function mailAiStateTone(
  aiState: CommunicationAiStateRecord['ai_state']
): 'success' | 'warning' | 'info' | 'neutral' {
  switch (aiState) {
    case 'PROCESSED':
      return 'success'
    case 'PROCESSING':
    case 'NEW':
      return 'info'
    case 'REVIEW_REQUIRED':
    case 'FAILED':
      return 'warning'
    case 'ARCHIVED':
      return 'neutral'
  }
}

function emptyMailInspector(): MailInspectorModel {
  return {
    intelligence: {
      score: 0,
      maxScore: 100,
      label: 'Projected importance',
      summary: 'Select a message to inspect Communications evidence.',
      checks: [
        {
          id: 'empty',
          label: 'No message selected',
          description: 'No backend message projection is currently selected.',
          icon: 'tabler:circle-dashed',
          tone: 'neutral',
        },
      ],
    },
    entityGroups: [],
    topics: [],
    semanticFacts: [],
    suggestedActions: [],
    relatedContext: [],
  }
}

function mailInspectorEntityGroups(
  message: CommunicationMessageDetailItem | CommunicationMessageSummary
): MailInspectorModel['entityGroups'] {
  const items: MailInspectorEntityItem[] = [
    {
      id: 'raw-record',
      entity: 'document',
      title: 'Raw provider record',
      description: message.raw_record_id,
      evidenceLabel: 'Source record retained before promotion',
      tone: 'neutral',
    },
    {
      id: 'provider-record',
      entity: 'knowledge',
      title: 'Provider record',
      description: message.provider_record_id,
      evidenceLabel: message.channel_kind,
      tone: 'info',
    },
  ]

  if (message.observation_id) {
    items.push({
      id: 'observation',
      entity: 'knowledge',
      title: 'Observation',
      description: message.observation_id,
      evidenceLabel: 'Canonical observation reference',
      tone: 'info',
    })
  }

  if (message.ai_summary) {
    items.push({
      id: 'ai-summary',
      entity: 'knowledge',
      title: 'AI summary candidate',
      description: message.ai_summary,
      evidenceLabel:
        message.ai_summary_generated_at ??
        'summary generated without timestamp',
      tone: 'accent',
    })
  }

  return [
    {
      id: 'source-evidence',
      title: 'Source evidence',
      items,
    },
  ]
}
