import { describe, expect, it } from 'vitest'
import {
  canRunTelegramDryRun,
  canTriggerTelegramProviderSearch,
  parseTelegramDryRunVariables,
} from './telegramRuntimePanelActions'

describe('Telegram runtime panel validation', () => {
  it('requires ids for dry-run and search', () => {
    expect(canRunTelegramDryRun('policy-1', 'chat-1')).toBe(true)
    expect(canRunTelegramDryRun(' ', 'chat-1')).toBe(false)
    expect(canTriggerTelegramProviderSearch('account-1', 'alice')).toBe(true)
    expect(canTriggerTelegramProviderSearch(null, 'alice')).toBe(false)
    expect(canTriggerTelegramProviderSearch('account-1', ' ')).toBe(false)
  })

  it('parses string-only dry-run variables without assertions', () => {
    expect(parseTelegramDryRunVariables('{"name":"Alice"}')).toEqual({ name: 'Alice' })
    expect(() => parseTelegramDryRunVariables('["Alice"]')).toThrow('JSON object')
    expect(() => parseTelegramDryRunVariables('{"count":1}')).toThrow('must be a string')
  })
})
