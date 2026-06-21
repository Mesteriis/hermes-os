import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import { fetchTelegramTopicSearch } from './telegramTopics'

describe('telegram topics API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('rejects projected topic search from integration clients', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)

    await expect(fetchTelegramTopicSearch('chat-42 ', '  architecture docs ', 25)).rejects.toThrow('moved')

    expect(fetchMock).not.toHaveBeenCalled()
  })
})
