import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { fetchProviderCallTranscript, fetchProviderCalls } from './callApi'

describe('communications call API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('uses provider-neutral calls routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ items: [] }))
      .mockResolvedValueOnce(ok({ transcript: null }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchProviderCalls(' zoom-live-1 ', 12, 'zoom')
    await fetchProviderCallTranscript('call-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/calls?limit=12&account_id=zoom-live-1&provider=zoom')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/calls/call-1/transcript')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })
})
