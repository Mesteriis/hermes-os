import { attachmentIcon, messageTime, senderLabel } from '../stores/communications'
import type {
  CommunicationAttachment,
  CommunicationMessageDetailItem,
  CommunicationMessageSummary,
  TranslationResponse,
} from '../types/communications'
import type {
  CommunicationConversationAttachmentModel,
  CommunicationConversationMessageModel,
  CommunicationConversationModel,
} from '../components/communicationDomainElements'
import { mailActionGroups } from './communicationMailWorkspaceActions'

export function mailSyncStatusIsActive(status: string): boolean {
  return (
    status === 'queued' ||
    status === 'running' ||
    status === 'recoverable_full_resync_needed'
  )
}

export function attachmentCount(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary,
  fallbackCount: number
): number {
  return 'attachment_count' in source ? source.attachment_count : fallbackCount
}

export function conversationMessage(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary,
  attachments: readonly CommunicationAttachment[],
  translation: TranslationResponse | null,
  providerFlagMutationAvailable: boolean
): CommunicationConversationMessageModel {
  return {
    id: source.message_id,
    author: senderLabel(source.sender_display_name ?? source.sender),
    body: messageBody(source),
    bodyFormat: 'body_html' in source && source.body_html ? 'html' : 'plain',
    bodyHtml: 'body_html' in source ? source.body_html ?? undefined : undefined,
    bodyHtmlSanitized:
      'body_html' in source ? Boolean(source.body_html) : undefined,
    timestamp: messageTime(source.occurred_at ?? source.projected_at),
    direction: messageDirection(source.delivery_state),
    subject: source.subject,
    fromLabel: source.sender,
    toLabel: source.recipients.join(', '),
    meta: source.provider_record_id,
    attachments: attachments.map(conversationAttachment),
    translation:
      translation?.translated && translation.text?.trim()
        ? {
            text: translation.text.trim(),
            target: translation.target ?? 'ru',
            model: translation.model,
          }
        : undefined,
    evidenceItems: [
      { id: 'raw-record', label: 'raw record', value: source.raw_record_id, mono: true },
      { id: 'provider-record', label: 'provider record', value: source.provider_record_id, mono: true },
    ],
    markers: [
      { id: 'workflow', label: 'workflow', value: source.workflow_state },
      { id: 'delivery', label: 'delivery', value: source.delivery_state },
    ],
    actionGroups: mailActionGroups(source, { providerFlagMutationAvailable }),
  }
}

export function messageTranslation(
  insight: { messageId: string; translation: TranslationResponse | null } | null,
  messageId: string
): TranslationResponse | null {
  return insight?.messageId === messageId ? insight.translation : null
}

export function emptyConversation(): CommunicationConversationModel {
  return {
    id: 'empty',
    channelKind: 'mail',
    title: 'No message selected',
    subtitle: 'Import or select a communication to inspect source evidence.',
    workflowState: 'new',
    facts: [],
    messages: [],
    draftPreview: '',
  }
}

function messageBody(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary
): string {
  if ('body_text' in source) return source.body_text
  return source.body_text_preview || source.ai_summary || source.subject
}

function messageDirection(
  deliveryState: string
): CommunicationConversationMessageModel['direction'] {
  return deliveryState === 'sent' || deliveryState === 'queued' || deliveryState === 'scheduled'
    ? 'outbound'
    : 'inbound'
}

function conversationAttachment(
  attachment: CommunicationAttachment
): CommunicationConversationAttachmentModel {
  return {
    id: attachment.attachment_id,
    name: attachment.filename ?? attachment.attachment_id,
    meta: `${attachment.content_type} · ${attachment.scan_status}`,
    icon: attachmentIcon(attachment.content_type),
    tone: attachment.scan_status === 'clean' ? 'success' : 'warning',
    scanStatus: attachment.scan_status,
  }
}
