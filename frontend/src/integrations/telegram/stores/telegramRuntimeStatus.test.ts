import { describe, expect, it } from 'vitest'

import { telegramRuntimeCommandTarget } from './telegramRuntimeStatus'

describe('telegramRuntimeCommandTarget', () => {
  it('formats mark-read command targets as read progress', () => {
    expect(
      telegramRuntimeCommandTarget({
        account_id: 'account-1',
        provider_kind: 'telegram_user',
        runtime_kind: 'fixture',
        status: 'running',
        fixture_runtime: true,
        tdjson_path: null,
        tdjson_runtime_available: false,
        tdjson_probe_error: null,
        telegram_api_id_configured: false,
        telegram_api_hash_configured: false,
        telegram_app_credentials_configured: false,
        live_send_available: false,
        runtime_blockers: [],
        last_error: null,
        last_command_status: 'completed',
        last_command_kind: 'mark_read',
        last_command_message_id: 'chat-1:777',
        updated_at: '2026-06-17T12:00:00Z',
      })
    ).toBe('Read through chat-1:777')
  })

  it('falls back to generic runtime command targets for other command kinds', () => {
    expect(
      telegramRuntimeCommandTarget({
        account_id: 'account-1',
        provider_kind: 'telegram_user',
        runtime_kind: 'fixture',
        status: 'running',
        fixture_runtime: true,
        tdjson_path: null,
        tdjson_runtime_available: false,
        tdjson_probe_error: null,
        telegram_api_id_configured: false,
        telegram_api_hash_configured: false,
        telegram_app_credentials_configured: false,
        live_send_available: false,
        runtime_blockers: [],
        last_error: null,
        last_command_status: 'completed',
        last_command_kind: 'pin',
        last_command_provider_chat_id: 'chat-1',
        updated_at: '2026-06-17T12:00:00Z',
      })
    ).toBe('chat-1')
  })
})
