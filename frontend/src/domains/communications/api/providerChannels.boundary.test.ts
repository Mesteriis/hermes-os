import { readFileSync } from 'node:fs'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchCommunicationConversationDetail,
  fetchCommunicationMessages,
  fetchCommunicationRawEvidence,
  searchCommunicationMessages,
} from './providerChannels'

describe('communications provider-neutral channel API boundary', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('does not embed provider-control integration routes in domain clients', () => {
    const source = readFileSync(new URL('./providerChannels.ts', import.meta.url), 'utf8')

    expect(source).not.toMatch(/\/api\/v1\/integrations\/.*\/provider-(commands|search|media|sync)/)
  })

  it('uses provider-neutral Communications routes for business reads and evidence', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ item: { conversation_id: 'conv-1' } }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ query: 'alpha', items: [], total: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ raw_record: { raw_record_id: 'raw-1' } }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchCommunicationConversationDetail('conv-1')
    await fetchCommunicationMessages({ conversationId: 'conv-1', channelKind: 'telegram_user' })
    await searchCommunicationMessages({ q: 'alpha', account_id: 'acct-1', provider_chat_id: 'chat-1' })
    await fetchCommunicationRawEvidence('msg-1')

    expect(String(fetchMock.mock.calls[0][0])).toContain('/api/v1/communications/conversations/conv-1')
    expect(String(fetchMock.mock.calls[1][0])).toContain('/api/v1/communications/messages?')
    expect(String(fetchMock.mock.calls[1][0])).toContain('conversation_id=conv-1')
    expect(String(fetchMock.mock.calls[2][0])).toContain('/api/v1/communications/search/messages?')
    expect(String(fetchMock.mock.calls[3][0])).toContain('/api/v1/communications/messages/msg-1/raw-evidence')
  })
})
