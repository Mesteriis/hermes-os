import { describe, expect, it } from 'vitest'
import {
  bilingualReplyFlowFormDefaults,
  bilingualReplyFlowFormSchema,
  bilingualReplyFlowFormToRequest,
  bilingualReplyToneOptions
} from './bilingualReplyFlowForm'

describe('bilingual reply flow form', () => {
  it('normalizes Russian reply text and the selected tone into an API request', () => {
    const values = bilingualReplyFlowFormSchema.parse({
      replyTextRu: '  Спасибо, мы проверим контракт сегодня.  ',
      tone: 'business'
    })

    expect(bilingualReplyFlowFormToRequest(values)).toEqual({
      reply_text_ru: 'Спасибо, мы проверим контракт сегодня.',
      tone: 'business'
    })
  })

  it('supports the required Hermes bilingual reply tones', () => {
    expect(bilingualReplyToneOptions).toEqual([
      'formal',
      'business',
      'friendly',
      'short',
      'detailed'
    ])
  })

  it('rejects empty replies, unsupported tones and oversized reply text', () => {
    const result = bilingualReplyFlowFormSchema.safeParse({
      replyTextRu: ' ',
      tone: 'casual'
    })

    expect(result.success).toBe(false)

    expect(() =>
      bilingualReplyFlowFormSchema.parse({
        replyTextRu: 'x'.repeat(64001),
        tone: 'formal'
      })
    ).toThrow()
  })

  it('uses a business tone and empty draft by default', () => {
    expect(bilingualReplyFlowFormDefaults()).toEqual({
      replyTextRu: '',
      tone: 'business'
    })
  })
})
