import { describe, expect, it } from 'vitest'
import {
  buildTelegramAccountSetupRequest,
  buildTelegramDryRunRequest,
  buildTelegramProviderSearchRequest,
  buildTelegramSyncChatsRequest,
  createTelegramRuntimeSetupForm,
  parseTelegramDryRunVariables,
  TELEGRAM_RUNTIME_CHAT_SYNC_CHUNK_SIZE,
  TELEGRAM_RUNTIME_PROVIDER_SEARCH_CHUNK_SIZE,
} from './telegramRuntimePanelActions'

describe('telegram runtime panel actions', () => {
  it('normalizes setup form and builds dry-run requests', () => {
    const form = { ...createTelegramRuntimeSetupForm(), accountId: ' account-1 ', displayName: ' Account ', externalAccountId: ' user-1 ', apiId: '123' }
    const setup = buildTelegramAccountSetupRequest(form)
    expect('request' in setup && setup.request).toMatchObject({ account_id: 'account-1', api_id: 123 })
    expect(parseTelegramDryRunVariables('{"name":"owner"}')).toEqual({ name: 'owner' })
    expect(buildTelegramDryRunRequest('policy-1', 'chat-1', { name: 'owner' }, 'command-1')).toMatchObject({ policy_id: 'policy-1', variables: { name: 'owner' } })
    expect(buildTelegramSyncChatsRequest('account-1')).toEqual({ account_id: 'account-1', limit: TELEGRAM_RUNTIME_CHAT_SYNC_CHUNK_SIZE })
    expect(buildTelegramProviderSearchRequest('account-1', ' alice ')).toEqual({ account_id: 'account-1', q: ' alice ', limit: TELEGRAM_RUNTIME_PROVIDER_SEARCH_CHUNK_SIZE })
  })

  it('rejects invalid setup and dry-run variable inputs', () => {
    const form = { ...createTelegramRuntimeSetupForm(), accountId: 'account', displayName: 'Name', externalAccountId: 'external', apiId: 'not-a-number' }
    expect(buildTelegramAccountSetupRequest(form)).toEqual({ error: 'api_id_invalid' })
    expect(() => parseTelegramDryRunVariables('{"count":1}')).toThrow('must be a string')
  })
})
