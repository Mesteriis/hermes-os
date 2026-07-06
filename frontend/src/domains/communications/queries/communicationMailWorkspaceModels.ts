import {
  conversationPreview,
  messageTime,
  senderEmail,
  senderLabel,
} from '../stores/communications'
import type { CommunicationMessageSummary } from '../types/communications'
import type {
  MailListItemMarker,
  MailListItemModel,
} from '../components/mail/mailElements'

export function mailItem(
  message: CommunicationMessageSummary,
  selectedMessageId: string
): MailListItemModel {
  const attachmentTotal = message.attachment_count
  const counters = [
    { kind: 'messages' as const, value: 1 },
    ...(attachmentTotal > 0
      ? [{ kind: 'attachments' as const, value: attachmentTotal }]
      : []),
  ]
  const labels = [
    message.workflow_state,
    ...(message.ai_category ? [message.ai_category] : []),
  ]

  return {
    id: message.message_id,
    accountLabel: message.account_id,
    mailboxLabel: mailboxLabel(message),
    providerRecordId: message.provider_record_id,
    fromName: senderLabel(message.sender_display_name ?? message.sender),
    fromAddress: senderEmail(message.sender),
    recipients: message.recipients,
    subject: message.subject,
    snippet: conversationPreview(message),
    sourceKind: 'mail',
    timestampLabel: messageTime(message.occurred_at ?? message.projected_at),
    workflowState: message.workflow_state,
    localState: message.local_state,
    deliveryState: message.delivery_state,
    aiCategory: message.ai_category ?? undefined,
    importanceScore: message.importance_score ?? undefined,
    attachmentCount: attachmentTotal,
    counters,
    labels,
    evidenceKinds: ['raw_record', 'provider_record'],
    hasOpenAction: message.workflow_state === 'needs_action',
    markers: mailItemMarkers(message),
    selected: message.message_id === selectedMessageId,
  }
}

function mailboxLabel(message: CommunicationMessageSummary): string {
  if (message.local_state === 'trash') return 'Trash'
  const mailbox = messageMailbox(message)
  if (mailbox && mailboxIsSpam(mailbox)) return 'Spam'
  if (mailbox && mailboxIsTrash(mailbox)) return 'Trash'
  if (mailbox && mailboxIsArchive(mailbox)) return 'Archive'
  if (mailbox && mailboxIsDrafts(mailbox)) return 'Drafts'
  if (mailbox && mailboxIsSent(mailbox)) return 'Sent'
  if (message.delivery_state === 'sent') return 'Sent'
  if (message.workflow_state === 'archived') return 'Archive'
  return mailbox || 'Inbox'
}

function mailItemMarkers(
  message: CommunicationMessageSummary
): MailListItemMarker[] {
  const markers: MailListItemMarker[] = []
  const mailbox = messageMailbox(message)
  if (message.workflow_state === 'spam' || mailboxIsSpam(mailbox)) {
    markers.push('spam')
  }
  if (message.workflow_state === 'archived' || mailboxIsArchive(mailbox)) {
    markers.push('archived')
  }
  if ((message.importance_score ?? 0) >= 75) markers.push('important')
  return markers
}

function messageMailbox(
  message: CommunicationMessageSummary
): string | undefined {
  const mailbox = message.message_metadata.mailbox
  if (typeof mailbox !== 'string') return undefined
  const trimmed = mailbox.trim()
  return trimmed || undefined
}

function mailboxIsSpam(mailbox: string | undefined): boolean {
  if (!mailbox) return false
  const normalized = mailbox.toLowerCase()
  return (
    normalized.includes('spam') ||
    normalized.includes('junk') ||
    normalized.includes('bulk mail')
  )
}

function mailboxIsArchive(mailbox: string | undefined): boolean {
  if (!mailbox) return false
  return mailbox.toLowerCase().includes('archive')
}

function mailboxIsTrash(mailbox: string | undefined): boolean {
  if (!mailbox) return false
  const normalized = mailbox.toLowerCase()
  return (
    normalized.includes('trash') ||
    normalized.includes('deleted') ||
    normalized.includes('bin')
  )
}

function mailboxIsDrafts(mailbox: string | undefined): boolean {
  if (!mailbox) return false
  return mailbox.toLowerCase().includes('draft')
}

function mailboxIsSent(mailbox: string | undefined): boolean {
  if (!mailbox) return false
  const normalized = mailbox.toLowerCase()
  return (
    normalized === 'sent' ||
    normalized.includes('sent messages') ||
    normalized.includes('sent mail')
  )
}
