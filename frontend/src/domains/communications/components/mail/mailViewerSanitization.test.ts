import { describe, expect, it } from 'vitest'
import { mailViewerBodyPreviewIsSanitized } from './mailViewerPresentation'
import type { CommunicationConversationMessageModel } from '../communicationDomainElements'

describe('mail viewer sanitization boundary', () => {
  it('allows sanitized HTML only in HTML preview modes', () => {
    const sanitized = ({ bodyFormat: 'html', bodyHtmlSanitized: true }) satisfies Pick<CommunicationConversationMessageModel, 'bodyFormat' | 'bodyHtmlSanitized'>
    const unsanitized = ({ bodyFormat: 'html', bodyHtmlSanitized: false }) satisfies Pick<CommunicationConversationMessageModel, 'bodyFormat' | 'bodyHtmlSanitized'>

    expect(mailViewerBodyPreviewIsSanitized(sanitized, 'original')).toBe(true)
    expect(mailViewerBodyPreviewIsSanitized(unsanitized, 'original')).toBe(false)
    expect(mailViewerBodyPreviewIsSanitized(sanitized, 'plain')).toBe(false)
    expect(mailViewerBodyPreviewIsSanitized(sanitized, 'translation')).toBe(false)
  })
})
