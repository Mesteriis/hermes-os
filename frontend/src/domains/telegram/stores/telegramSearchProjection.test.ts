import { describe, expect, it } from 'vitest'
import { telegramWorkspaceSearchSourceLabel } from './telegramSearchProjection'
import type { TelegramRuntimeStatus } from '../types/telegram'

function runtime(overrides: Partial<TelegramRuntimeStatus>): TelegramRuntimeStatus {
  return {
    account_id: 'acct-1',
    provider_kind: 'telegram_user',
    runtime_kind: 'tdlib',
    status: 'running',
    fixture_runtime: false,
    tdjson_path: '/tmp/tdjson',
    tdjson_runtime_available: true,
    tdjson_probe_error: null,
    telegram_api_id_configured: true,
    telegram_api_hash_configured: true,
    telegram_app_credentials_configured: true,
    live_send_available: true,
    runtime_blockers: [],
    last_error: null,
    updated_at: '2026-06-17T10:00:00Z',
    ...overrides,
  }
}

describe('telegram workspace search projection', () => {
  it('labels live TDLib search as provider-backed projection refresh', () => {
    expect(telegramWorkspaceSearchSourceLabel(runtime({}))).toBe(
      'Provider search with projection refresh'
    )
  })

  it('labels fixture and unavailable runtime search without implying provider access', () => {
    expect(telegramWorkspaceSearchSourceLabel(runtime({ fixture_runtime: true }))).toBe(
      'Fixture projection search'
    )
    expect(
      telegramWorkspaceSearchSourceLabel(
        runtime({
          status: 'blocked',
          tdjson_runtime_available: false,
        })
      )
    ).toBe('Projection search; provider runtime unavailable')
  })

  it('labels missing account scope as local projection search', () => {
    expect(telegramWorkspaceSearchSourceLabel(null)).toBe('Local projection search')
  })
})
