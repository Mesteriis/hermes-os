import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { prepareBilingualReplyFlow } from './bilingualReplyFlow'

describe('bilingual reply flow API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts Russian reply text and tone to the protected review endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          message_id: 'mail_message:1',
          subject: 'Re: Contrato',
          tone: 'business',
          reply_language: 'ru',
          send_ready: false,
          original: {
            language: 'es',
            confidence: 0.7,
            text: 'Hola equipo'
          },
          translation: {
            target: 'ru',
            translated: false,
            text: null,
            model: null,
            reason: 'translation runtime unavailable'
          },
          reply: {
            language: 'ru',
            tone: 'business',
            text: 'Спасибо.'
          },
          back_translation: {
            target: 'es',
            translated: false,
            text: null,
            model: null,
            reason: 'translation runtime unavailable'
          }
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await prepareBilingualReplyFlow('mail_message:1', {
      reply_text_ru: 'Спасибо.',
      tone: 'business'
    })

    expect(response.reply.text).toBe('Спасибо.')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/api/v1/communications/messages/mail_message%3A1/bilingual-reply-flow'
    )
    expect(init.method).toBe('POST')
    expect(init.headers['X-Hermes-Secret']).toBe('test-secret')
    expect(JSON.parse(init.body as string)).toEqual({
      reply_text_ru: 'Спасибо.',
      tone: 'business'
    })
  })
})
