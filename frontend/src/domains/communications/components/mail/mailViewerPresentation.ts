import { htmlToComposePlainText } from '../richComposeHtml'
import type { CommunicationConversationMessageModel } from '../communicationDomainElements'

type Translate = (key: string, params?: Record<string, string | number>) => string
export type MailViewerBodyMode = 'translation' | 'clean' | 'original' | 'plain'

export interface RecipientDetailRow {
  id: string
  label: string
  value: string
}

export interface ContextSummaryItem {
  id: string
  label: string
  tone?: string
}

export function isMailViewerBodyMode(value: string | string[]): value is MailViewerBodyMode {
  return value === 'translation' || value === 'clean' || value === 'original' || value === 'plain'
}

export function mailViewerInitialBodyMode(
  message: CommunicationConversationMessageModel
): MailViewerBodyMode {
  return mailViewerHasTranslation(message) ? 'translation' : message.bodyHtml ? 'original' : 'clean'
}

export function mailViewerBodyPreviewClass(mode: MailViewerBodyMode): string {
  return [
    'communication-email-message__body-preview',
    `communication-email-message__body-preview--${mode}`,
  ].join(' ')
}

export function mailViewerSenderLabel(message: CommunicationConversationMessageModel): string {
  return message.fromLabel ?? message.author
}

export function mailViewerRecipientLabel(message: CommunicationConversationMessageModel): string {
  return message.toLabel ?? 'Owner'
}

export function mailViewerHasTranslation(message: CommunicationConversationMessageModel): boolean {
  return Boolean(message.translation?.text.trim())
}

export function mailViewerRecipientDetailRows(
  message: CommunicationConversationMessageModel,
  t: Translate
): RecipientDetailRow[] {
  const rows: RecipientDetailRow[] = []
  if (message.ccLabel) rows.push({ id: 'cc', label: t('CC'), value: message.ccLabel })
  if (message.bccLabel) rows.push({ id: 'bcc', label: t('BCC'), value: message.bccLabel })
  if (message.replyToLabel) rows.push({ id: 'reply-to', label: t('Reply to'), value: message.replyToLabel })
  return rows
}

export function mailViewerBodyModeItems(
  message: CommunicationConversationMessageModel,
  t: Translate
): Array<{ value: MailViewerBodyMode; label: string; icon: string }> {
  return [
    ...(mailViewerHasTranslation(message)
      ? [{ value: 'translation' as const, label: t('Translation'), icon: 'tabler:language' }]
      : []),
    { value: 'clean', label: t('Clean'), icon: 'tabler:sparkles' },
    { value: 'original', label: t('Original HTML'), icon: 'tabler:code' },
    { value: 'plain', label: t('Plain text'), icon: 'tabler:file-text' },
  ]
}

export function mailViewerTranslationMeta(
  message: CommunicationConversationMessageModel,
  t: Translate
): string {
  if (!message.translation) return ''
  const parts = [t('Translation'), message.translation.target]
  if (message.translation.model) parts.push(message.translation.model)
  return parts.join(' · ')
}

export function mailViewerBodyPreviewContent(
  message: CommunicationConversationMessageModel,
  mode: MailViewerBodyMode
): string {
  if (mode === 'translation' && message.translation) return message.translation.text
  if (mode === 'plain') {
    return message.bodyFormat === 'html' && message.bodyHtml
      ? htmlToComposePlainText(message.bodyHtml)
      : message.body
  }
  return message.bodyFormat === 'html' ? (message.bodyHtml ?? message.body) : message.body
}

export function mailViewerBodyPreviewFormat(
  message: Pick<CommunicationConversationMessageModel, 'bodyFormat'>,
  mode: MailViewerBodyMode
): 'html' | 'text' {
  return mode === 'plain' || mode === 'translation'
    ? 'text'
    : message.bodyFormat === 'html' ? 'html' : 'text'
}

export function mailViewerBodyPreviewIsSanitized(
  message: Pick<CommunicationConversationMessageModel, 'bodyFormat' | 'bodyHtmlSanitized'>,
  mode: MailViewerBodyMode
): boolean {
  return mailViewerBodyPreviewFormat(message, mode) === 'html'
    && message.bodyHtmlSanitized === true
}

export function mailViewerContextSummaryItems(
  message: CommunicationConversationMessageModel,
  t: Translate
): ContextSummaryItem[] {
  const evidenceItems = message.evidenceItems ?? []
  const evidenceTone = evidenceItems.some((item) => item.tone === 'danger')
    ? 'danger'
    : evidenceItems.some((item) => item.tone === 'warning') ? 'warning' : 'info'
  const items: ContextSummaryItem[] = []
  for (const label of message.labels ?? []) items.push({ id: `label-${label}`, label })
  for (const marker of message.markers ?? []) {
    items.push({ id: marker.id, label: typeof marker.value === 'number' ? String(marker.value) : t(marker.value), tone: marker.tone })
  }
  if (evidenceItems.length > 0) {
    items.push({ id: 'evidence-count', label: t('{count} evidence', { count: evidenceItems.length }), tone: evidenceTone })
  }
  return items
}
