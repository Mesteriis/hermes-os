import type {
  CommunicationMessageDetailItem,
  CommunicationMessageSummary,
  ComposeFormModel,
  CommunicationDraft,
  CommunicationKnowledgeCandidate,
  CommunicationMessageInsight,
  SendCommunicationRequest,
  ThreadMessage
} from '../types/communications'
import { datetimeLocalToIso } from '../forms/composeDraftAutosave'
import { splitComposeRecipients } from '../forms/composeValidation'

type ReplyMessageSource = CommunicationMessageSummary | CommunicationMessageDetailItem
type QuoteMessageSource = ReplyMessageSource | ThreadMessage
type ReplyComposeOptions = {
  draftBodyText?: string
  draftBodyHtml?: string | null
}

export type AiSummaryContract = {
  key_points: string[]
  action_items: string[]
  risks: string[]
  deadlines: string[]
  event_candidates: CommunicationKnowledgeCandidate[]
  persona_candidates: CommunicationKnowledgeCandidate[]
  organization_candidates: CommunicationKnowledgeCandidate[]
  document_candidates: CommunicationKnowledgeCandidate[]
  agreement_candidates: CommunicationKnowledgeCandidate[]
}

export type CommunicationExtractionReviewItem = {
  title: string
  meta: string[]
  body: string
}

export type CommunicationExtractionReviewSection = {
  kind: 'task' | 'note'
  title: string
  items: CommunicationExtractionReviewItem[]
}

export type CommunicationKnowledgeReviewSection = {
  kind: 'event' | 'persona' | 'organization' | 'document' | 'agreement'
  title: string
  items: CommunicationKnowledgeCandidate[]
}

export function emptyCommunicationMessageInsight(messageId: string): CommunicationMessageInsight {
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

export function communicationExtractionSectionsFromInsight(
  insight: CommunicationMessageInsight | null
): CommunicationExtractionReviewSection[] {
  if (!insight) return []
  const sections: CommunicationExtractionReviewSection[] = []
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

export function communicationKnowledgeSectionsFromSummaryContract(
  contract: AiSummaryContract | null
): CommunicationKnowledgeReviewSection[] {
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

export function communicationMessageLabelsFromMetadata(metadata: Record<string, unknown>): string[] {
  const labels = metadata.labels
  if (!Array.isArray(labels)) return []
  return [...new Set(labels
    .filter((label): label is string => typeof label === 'string' && label.trim().length > 0)
    .map((label) => label.trim()))]
}

export function communicationMessageSnoozeUntilFromMetadata(metadata: Record<string, unknown>): string | null {
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

function candidateArrayValue(value: unknown): CommunicationKnowledgeCandidate[] {
  if (!Array.isArray(value)) return []
  return value.flatMap((item): CommunicationKnowledgeCandidate[] => {
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
  message: ReplyMessageSource,
  fallbackAccountId: string,
  draftId: string,
  options: ReplyComposeOptions = {}
): ComposeFormModel {
  const replyBody = replyBodyWithQuote(message, options)
  return {
    mode: 'reply',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: message.sender,
    ccText: '',
    bccText: '',
    subject: message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`,
    body: replyBody.body,
    bodyHtml: replyBody.bodyHtml,
    bodyFormat: 'html',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: message.provider_record_id || null
  }
}

export function replyAllComposeForm(
  message: ReplyMessageSource,
  fallbackAccountId: string,
  draftId: string,
  options: ReplyComposeOptions = {}
): ComposeFormModel {
  return {
    ...replyComposeForm(message, fallbackAccountId, draftId, options),
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
  const replyBody = replyBodyWithQuote(message, { draftBodyHtml })
  return {
    mode: 'reply',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: message.sender,
    ccText: '',
    bccText: '',
    subject: message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`,
    body: replyBody.body,
    bodyHtml: replyBody.bodyHtml,
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

export function composeFormToSendRequest(form: ComposeFormModel): SendCommunicationRequest {
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

function replyBodyWithQuote(
  message: QuoteMessageSource,
  options: ReplyComposeOptions
): Pick<ComposeFormModel, 'body' | 'bodyHtml'> {
  const quotedText = quotedPlainText(message)
  const quotedHtmlValue = quotedHtml(message)
  const draftBodyHtml = options.draftBodyHtml?.trim() ?? ''
  const draftBodyText = options.draftBodyText?.trim() ?? ''
  if (draftBodyHtml) {
    return {
      body: `${htmlToPlainText(draftBodyHtml)}${quotedText}`,
      bodyHtml: `${draftBodyHtml}${quotedHtmlValue}`
    }
  }
  if (draftBodyText) {
    return {
      body: `${draftBodyText}${quotedText}`,
      bodyHtml: `${plainTextToHtml(draftBodyText)}${quotedHtmlValue}`
    }
  }
  return {
    body: quotedText,
    bodyHtml: quotedHtmlValue
  }
}

function quotedPlainText(message: QuoteMessageSource): string {
  const header = `On ${message.projected_at}, ${message.sender} wrote:`
  const quoted = quoteBodyText(message)
    .split(/\r?\n/)
    .map((line) => `> ${line}`)
    .join('\n')
  return `\n\n${header}\n${quoted}`
}

function quotedHtml(message: QuoteMessageSource): string {
  const body = escapeHtml(quoteBodyText(message)).replace(/\r?\n/g, '<br>')
  return [
    '<p><br></p>',
    `<p>On ${escapeHtml(message.projected_at)}, ${escapeHtml(message.sender)} wrote:</p>`,
    `<blockquote>${body}</blockquote>`
  ].join('')
}

function quoteBodyText(message: QuoteMessageSource): string {
  if ('body_text' in message && message.body_text.trim().length > 0) {
    return message.body_text
  }
  if ('body_html' in message && message.body_html) {
    return htmlToPlainText(message.body_html)
  }
  if ('body_text_preview' in message) {
    return message.body_text_preview
  }
  return ''
}

function plainTextToHtml(value: string): string {
  const paragraphs = value
    .split(/\n{2,}/)
    .map((paragraph) => paragraph.trim())
    .filter((paragraph) => paragraph.length > 0)
  if (paragraphs.length === 0) return '<p></p>'
  return paragraphs
    .map((paragraph) => `<p>${escapeHtml(paragraph).replace(/\r?\n/g, '<br>')}</p>`)
    .join('')
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

export function draftToComposeForm(draft: CommunicationDraft): ComposeFormModel {
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
