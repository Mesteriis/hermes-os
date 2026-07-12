import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../platform/api/ApiClient'
import {
  fetchMailProviderCommandDiagnostics,
  retryMailProviderCommand,
} from './providerCommandDiagnostics'

describe('mail provider command diagnostics API', () => {
  beforeEach(() => ApiClient.init('http://127.0.0.1:8080', 'test-secret'))
  afterEach(() => {
    ApiClient.resetForTests()
    vi.unstubAllGlobals()
  })

  it('scopes diagnostics to account and optional status without provider payloads', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      items: [{
        command_id: 'command-1',
        account_id: 'account-1',
        command_kind: 'mark_read',
        message_id: 'message-1',
        status: 'retrying',
        retry_count: 1,
        max_retries: 3,
        reconciliation_status: 'not_observed',
        next_attempt_at: null,
        last_attempt_at: null,
        dead_lettered_at: null,
        last_error: 'temporary provider failure',
        created_at: '2026-07-11T08:00:00Z',
        updated_at: '2026-07-11T08:01:00Z'
      }],
      counts: [{ status: 'retrying', count: 1 }]
    }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
    vi.stubGlobal('fetch', fetchMock)

    const result = await fetchMailProviderCommandDiagnostics('account-1', 'retrying')

    expect(result.items[0]?.command_kind).toBe('mark_read')
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      'http://127.0.0.1:8080/api/v1/communications/provider-commands/diagnostics?account_id=account-1&limit=50&status=retrying'
    )
  })

  it('requeues only the selected command through the protected recovery route', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      command_id: 'command/dead 1',
      status: 'retrying',
      retry_count: 0,
      max_retries: 3,
      reconciliation_status: 'not_observed',
      next_attempt_at: '2026-07-12T12:00:00Z',
    }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
    vi.stubGlobal('fetch', fetchMock)

    const result = await retryMailProviderCommand('command/dead 1')

    expect(result.status).toBe('retrying')
    expect(result.retry_count).toBe(0)
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      'http://127.0.0.1:8080/api/v1/communications/provider-commands/command%2Fdead%201/retry'
    )
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
      method: 'POST',
      body: '{}',
      headers: { 'X-Hermes-Secret': 'test-secret' },
    })
  })
})
