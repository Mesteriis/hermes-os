import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import { fetchTelegramRawMessageEvidence } from './telegramRawEvidence'

describe('telegram raw evidence API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('rejects projected raw evidence from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(fetchTelegramRawMessageEvidence('msg/raw 1')).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })
})
