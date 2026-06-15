import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchMessageAiState,
  updateMessageAiState
} from './aiState'

describe('communication AI state API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('gets and updates first-class message AI state', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg:1', ai_state: 'NEW' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg:1', ai_state: 'REVIEW_REQUIRED' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchMessageAiState('msg:1')
    await updateMessageAiState('msg:1', {
      ai_state: 'REVIEW_REQUIRED',
      review_reason: 'Needs owner review'
    })

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/communications/messages/msg%3A1/ai-state')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/communications/messages/msg%3A1/ai-state')
    expect(fetchMock.mock.calls[1][1].method).toBe('PUT')
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      ai_state: 'REVIEW_REQUIRED',
      review_reason: 'Needs owner review'
    })
  })
})
