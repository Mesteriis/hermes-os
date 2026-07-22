import { describe, expect, it } from 'vitest'
import type { CommunicationConversationMessageModel } from '../communicationDomainElements'
import {
  mailViewerBodyPreviewContent,
  mailViewerBodyPreviewClass,
  mailViewerBodyPreviewFormat,
  mailViewerContextSummaryItems,
  mailViewerInitialBodyMode,
  isMailViewerBodyMode,
  mailViewerRecipientDetailRows,
  mailViewerTranslationMeta,
} from './mailViewerPresentation'

describe('mail viewer presentation', () => {
  it('projects recipient, translation and body modes without UI state', () => {
    const message = messageModel()
    const t = (key: string) => key

    expect(mailViewerRecipientDetailRows(message, t).map((row) => row.id)).toEqual(['cc', 'reply-to'])
    expect(mailViewerTranslationMeta(message, t)).toBe('Translation · es · hermes')
    expect(mailViewerBodyPreviewContent(message, 'translation')).toBe('Hola')
    expect(mailViewerBodyPreviewFormat(message, 'plain')).toBe('text')
    expect(mailViewerBodyPreviewContent(message, 'plain')).toContain('Hello')
    expect(mailViewerInitialBodyMode(message)).toBe('translation')
    expect(mailViewerBodyPreviewClass('plain')).toContain('--plain')
    expect(isMailViewerBodyMode('clean')).toBe(true)
    expect(isMailViewerBodyMode('invalid')).toBe(false)
  })

  it('summarizes labels, markers and evidence tone', () => {
    const items = mailViewerContextSummaryItems(messageModel(), (key) => key)
    expect(items.map((item) => item.id)).toEqual(['label-inbox', 'marker-1', 'evidence-count'])
    expect(items.at(-1)?.tone).toBe('danger')
  })
})

function messageModel(): CommunicationConversationMessageModel {
  return {
    id: 'message-1', author: 'sender@example.test', body: 'Hello', bodyFormat: 'html',
    bodyHtml: '<p>Hello</p>', bodyHtmlSanitized: true, timestamp: '', direction: 'inbound',
    ccLabel: 'copy@example.test', replyToLabel: 'reply@example.test',
    translation: { text: 'Hola', target: 'es', model: 'hermes' }, labels: ['inbox'],
    markers: [{ id: 'marker-1', label: 'Priority', value: 1 }],
    evidenceItems: [{ id: 'evidence-1', label: 'Risk', value: 'high', tone: 'danger' }],
  }
}
