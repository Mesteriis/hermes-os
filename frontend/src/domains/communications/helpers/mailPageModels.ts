import type {
  CommunicationMessageSummary,
  ComposeFormModel,
  EmailDraft,
  MailKnowledgeCandidate,
  MailMessageInsight,
  SendEmailRequest,
  ThreadMessage
} from '../types/communications'
import { datetimeLocalToIso } from '../forms/composeDraftAutosave'
import { splitComposeRecipients } from '../forms/composeValidation'

export type AiSummaryContract = {
  key_points: string[]
  action_items: string[]
  risks: string[]
  deadlines: string[]
  event_candidates: MailKnowledgeCandidate[]
  persona_candidates: MailKnowledgeCandidate[]
  organization_candidates: MailKnowledgeCandidate[]
  document_candidates: MailKnowledgeCandidate[]
  agreement_candidates: MailKnowledgeCandidate[]
}

export type MailExtractionReviewItem = {
  title: string
  meta: string[]
  body: string
}

export type MailExtractionReviewSection = {
  kind: 'task' | 'note'
  title: string
  items: MailExtractionReviewItem[]
}

export type MailKnowledgeReviewSection = {
  kind: 'event' | 'persona' | 'organization' | 'document' | 'agreement'
  title: string
  items: MailKnowledgeCandidate[]
}

export function emptyMailMessageInsight(messageId: string): MailMessageInsight {
  return {
    messageId,
    explain: null,
    smartCc: null,
    auth: null,
    signature: null,
    language: null,
    aiReply: null,
    tasks: [],
    notes: [],
    translation: null
  }
}

export function mailExtractionSectionsFromInsight(
  insight: MailMessageInsight | null
): MailExtractionReviewSection[] {
  if (!insight) return []
  const sections: MailExtractionReviewSection[] = []
  if (insight.tasks.length > 0) {
    sections.push({
      kind: 'task',
      title: 'Task candidates',
      items: insight.tasks.map((task) => ({
        title: task.title,
        meta: [
          task.due_date ? `Due ${task.due_date}` : '',
          task.assignee ? `Assignee ${task.assignee}` : '',
          task.priority ? `Priority ${task.priority}` : ''
        ].filter(nonEmptyString),
        body: task.source
      }))
    })
  }
  if (insight.notes.length > 0) {
    sections.push({
      kind: 'note',
      title: 'Note candidates',
      items: insight.notes.map((note) => ({
        title: note.title,
        meta: note.tags.filter(nonEmptyString),
        body: note.content.trim() || note.source
      }))
    })
  }
  return sections
}

export function mailKnowledgeSectionsFromSummaryContract(
  contract: AiSummaryContract | null
): MailKnowledgeReviewSection[] {
  if (!contract) return []
  return [
    { kind: 'event' as const, title: 'Event candidates', items: contract.event_candidates },
    { kind: 'persona' as const, title: 'Persona candidates', items: contract.persona_candidates },
    {
      kind: 'organization' as const,
      title: 'Organization candidates',
      items: contract.organization_candidates
    },
    { kind: 'document' as const, title: 'Document candidates', items: contract.document_candidates },
    { kind: 'agreement' as const, title: 'Agreement candidates', items: contract.agreement_candidates }
  ].filter((section) => section.items.length > 0)
}

export function mailMessageLabelsFromMetadata(metadata: Record<string, unknown>): string[] {
  const labels = metadata.labels
  if (!Array.isArray(labels)) return []
  return [...new Set(labels
    .filter((label): label is string => typeof label === 'string' && label.trim().length > 0)
    .map((label) => label.trim()))]
}

export function mailMessageSnoozeUntilFromMetadata(metadata: Record<string, unknown>): string | null {
  return typeof metadata.snooze_until === 'string' && metadata.snooze_until.trim().length > 0
    ? metadata.snooze_until.trim()
    : null
}

export function aiSummaryContractFromMetadata(metadata: Record<string, unknown>): AiSummaryContract | null {
  const value = metadata.ai_summary_contract
  if (!isRecord(value)) return null
  return {
    key_points: stringArrayValue(value.key_points),
    action_items: stringArrayValue(value.action_items),
    risks: stringArrayValue(value.risks),
    deadlines: stringArrayValue(value.deadlines),
    event_candidates: candidateArrayValue(value.event_candidates),
    persona_candidates: candidateArrayValue(value.persona_candidates),
    organization_candidates: candidateArrayValue(value.organization_candidates),
    document_candidates: candidateArrayValue(value.document_candidates),
    agreement_candidates: candidateArrayValue(value.agreement_candidates)
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function stringArrayValue(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((item): item is string => typeof item === 'string' && item.trim().length > 0)
}

function candidateArrayValue(value: unknown): MailKnowledgeCandidate[] {
  if (!Array.isArray(value)) return []
  return value.flatMap((item): MailKnowledgeCandidate[] => {
    if (typeof item === 'string' && item.trim().length > 0) {
      const candidate = item.trim()
      return [{ title: candidate, evidence: candidate }]
    }
    if (!isRecord(item) || typeof item.title !== 'string' || item.title.trim().length === 0) {
      return []
    }
    return [{
      title: item.title.trim(),
      evidence: typeof item.evidence === 'string' ? item.evidence.trim() : ''
    }]
  })
}

function nonEmptyString(value: string): value is string {
  return value.trim().length > 0
}

export function replyComposeForm(
  message: CommunicationMessageSummary,
  fallbackAccountId: string,
  draftId: string
): ComposeFormModel {
  return {
    mode: 'reply',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: message.sender,
    ccText: '',
    bccText: '',
    subject: message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`,
    body: '',
    bodyHtml: null,
    bodyFormat: 'plain',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: message.provider_record_id || null
  }
}

export function replyAllComposeForm(
  message: CommunicationMessageSummary,
  fallbackAccountId: string,
  draftId: string
): ComposeFormModel {
  return {
    ...replyComposeForm(message, fallbackAccountId, draftId),
    ccText: message.recipients.join(', ')
  }
}

export function forwardComposeForm(
  message: CommunicationMessageSummary,
  fallbackAccountId: string,
  draftId: string
): ComposeFormModel {
  const subject = message.subject.startsWith('Fwd:') ? message.subject : `Fwd: ${message.subject}`
  const body = [
    '',
    '',
    '--- Forwarded message ---',
    `From: ${message.sender}`,
    `Subject: ${message.subject}`,
    '',
    message.body_text_preview
  ].join('\n')
  return {
    mode: 'forward',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: '',
    ccText: '',
    bccText: '',
    subject,
    body,
    bodyHtml: null,
    bodyFormat: 'plain',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: null
  }
}

export function threadReplyComposeForm(
  message: ThreadMessage,
  fallbackAccountId: string,
  draftId: string,
  draftBodyHtml = ''
): ComposeFormModel {
  const quotedText = quotedPlainText(message)
  const normalizedDraftHtml = draftBodyHtml.trim()
  return {
    mode: 'reply',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: message.sender,
    ccText: '',
    bccText: '',
    subject: message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`,
    body: normalizedDraftHtml
      ? `${htmlToPlainText(normalizedDraftHtml)}${quotedText}`
      : quotedText,
    bodyHtml: normalizedDraftHtml
      ? `${normalizedDraftHtml}${quotedHtml(message)}`
      : quotedHtml(message),
    bodyFormat: 'html',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: message.provider_record_id || message.message_id
  }
}

export function newComposeForm(accountId: string, draftId: string): ComposeFormModel {
  return {
    mode: 'compose',
    draftId,
    accountId,
    toText: '',
    ccText: '',
    bccText: '',
    subject: '',
    body: '',
    bodyHtml: null,
    bodyFormat: 'plain',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: null
  }
}

export function composeFormToSendRequest(form: ComposeFormModel): SendEmailRequest {
  return {
    account_id: form.accountId,
    to: splitComposeRecipients(form.toText),
    cc: splitComposeRecipients(form.ccText),
    bcc: splitComposeRecipients(form.bccText),
    subject: form.subject,
    body_text: form.body,
    body_html: form.bodyFormat === 'html' ? form.bodyHtml : null,
    in_reply_to: form.inReplyTo,
    draft_id: form.draftId,
    scheduled_send_at: datetimeLocalToIso(form.scheduledSendAt),
    undo_send_seconds: form.undoSendSeconds,
    confirmed_provider_write: true
  }
}

function quotedPlainText(message: ThreadMessage): string {
  const header = `On ${message.projected_at}, ${message.sender} wrote:`
  const quoted = message.body_text
    .split(/\r?\n/)
    .map((line) => `> ${line}`)
    .join('\n')
  return `\n\n${header}\n${quoted}`
}

function quotedHtml(message: ThreadMessage): string {
  const body = escapeHtml(message.body_text).replace(/\r?\n/g, '<br>')
  return [
    '<p><br></p>',
    `<p>On ${escapeHtml(message.projected_at)}, ${escapeHtml(message.sender)} wrote:</p>`,
    `<blockquote>${body}</blockquote>`
  ].join('')
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function htmlToPlainText(value: string): string {
  return value
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/p>/gi, '\n')
    .replace(/<[^>]+>/g, '')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&')
    .trim()
}

export function draftToComposeForm(draft: EmailDraft): ComposeFormModel {
  return {
    mode: 'compose',
    draftId: draft.draft_id,
    accountId: draft.account_id,
    toText: draft.to_recipients.join(', '),
    ccText: draft.cc_recipients.join(', '),
    bccText: draft.bcc_recipients.join(', '),
    subject: draft.subject,
    body: draft.body_text,
    bodyHtml: draft.body_html,
    bodyFormat: draft.body_html ? 'html' : 'plain',
    scheduledSendAt: isoToDatetimeLocal(draft.scheduled_send_at),
    undoSendSeconds: null,
    inReplyTo: draft.in_reply_to
  }
}

function isoToDatetimeLocal(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) return ''
  const offsetMs = date.getTimezoneOffset() * 60_000
  return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16)
}
