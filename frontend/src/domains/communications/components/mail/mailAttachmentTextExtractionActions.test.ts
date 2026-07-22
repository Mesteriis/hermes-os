import { describe, expect, it } from 'vitest'
import { buildMailAttachmentTranslationRequest } from './mailAttachmentTextExtractionActions'

describe('mail attachment text extraction actions', () => {
  it('builds the translation mutation request', () => {
    expect(buildMailAttachmentTranslationRequest('attachment-1', 'ru')).toEqual({
      attachmentId: 'attachment-1',
      request: { target_language: 'ru' },
    })
  })
})
