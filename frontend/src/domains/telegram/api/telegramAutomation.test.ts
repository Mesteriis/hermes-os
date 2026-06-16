import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchTelegramAutomationPolicies,
  fetchTelegramAutomationTemplates,
  runTelegramSendDryRun,
} from './telegramAutomation'

describe('telegram automation API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('loads policies, templates and send dry-run routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [{ policy_id: 'pol-1', template_id: 'tpl-1', name: 'Follow up', enabled: true, account_id: 'acc-1', allowed_chat_ids: ['chat-1'], trigger_kind: 'ai_follow_up', max_sends_per_hour: 3, quiet_hours: {}, expires_at: null, conditions: {}, created_at: '2026-06-16T12:00:00Z', updated_at: '2026-06-16T12:00:00Z' }] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [{ template_id: 'tpl-1', name: 'Follow up template', body_template: 'Hello {{name}}', required_variables: ['name'], created_at: '2026-06-16T12:00:00Z', updated_at: '2026-06-16T12:00:00Z' }] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ outbound_message_id: 'out-1', policy_id: 'pol-1', template_id: 'tpl-1', account_id: 'acc-1', provider_chat_id: 'chat-1', rendered_text: 'Hello Maria', rendered_preview_hash: 'sha256:abc', status: 'allowed', event_id: 'evt-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramAutomationPolicies()
    await fetchTelegramAutomationTemplates()
    await runTelegramSendDryRun({
      command_id: 'cmd-1',
      policy_id: 'pol-1',
      provider_chat_id: 'chat-1',
      variables: { name: 'Maria' },
      source_context: { source: 'telegram_workbench' },
    })

    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/policies')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/policies/templates')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/policies/telegram-send/dry-run')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
  })
})
