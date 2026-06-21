import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram members sync realtime handling', () => {
  it('patches cached telegram runtime status for members sync events', () => {
    const runtimeKey = ['integrations', 'telegram', 'runtime', 'account-1']
    const runtimeStatus = {
      account_id: 'account-1',
      provider_kind: 'telegram_user',
      runtime_kind: 'tdlib_qr_authorized',
      status: 'idle',
      fixture_runtime: false,
      tdjson_runtime_available: true,
      telegram_app_credentials_configured: true,
      live_send_available: true,
      last_error: null,
      updated_at: '2026-06-17T09:00:00Z'
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(runtimeStatus) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram', 'messages'])) return []
        return [[runtimeKey, runtimeStatus]]
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-members-sync-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.sync.completed',
            metadata: { account_id: 'account-1' },
            payload: {
              scope: 'members',
              status: 'completed',
              synced_count: 2,
              provider_chat_id: 'chat-1'
            }
          }
        })
      },
      queryClient
    )

    const patchedRuntime = setQueryData.mock.results[0]?.value
    expect(patchedRuntime.last_sync_scope).toBe('members')
    expect(patchedRuntime.last_sync_status).toBe('completed')
    expect(patchedRuntime.last_synced_count).toBe(2)
    expect(patchedRuntime.last_sync_provider_chat_id).toBe('chat-1')
  })
})
