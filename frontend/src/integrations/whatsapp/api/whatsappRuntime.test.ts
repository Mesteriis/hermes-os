import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  deadLetterWhatsappProviderCommand,
  fetchWhatsappAccounts,
  fetchWhatsappAccountCapabilities,
  fetchWhatsappProviderCommands,
  fetchWhatsappSyncChats,
  fetchWhatsappSyncCalls,
  fetchWhatsappSyncContacts,
  fetchWhatsappSyncHistory,
  fetchWhatsappSyncMedia,
  fetchWhatsappSyncMembers,
  fetchWhatsappSyncPresence,
  fetchWhatsappSyncStatuses,
  fetchWhatsappRuntimeHealth,
  fetchWhatsappRuntimeStatus,
  publishWhatsappStatus,
  relinkWhatsappRuntime,
  retryWhatsappProviderCommand,
  rotateWhatsappRuntime,
  removeWhatsappRuntime,
  revokeWhatsappRuntime,
  startWhatsappRuntime,
  stopWhatsappRuntime,
} from './whatsapp'
import {
  WHATSAPP_RUNTIME_COMMANDS_PAGE_SIZE,
  WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE,
} from '../queries/useWhatsappRuntimeQuery'

describe('whatsapp runtime API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('loads whatsapp account list with and without removed accounts', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ items: [{ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', display_name: 'Account One', external_account_id: 'wa:1', runtime: 'live_blocked', lifecycle_state: 'created', created_at: '2026-06-25T10:00:00Z', updated_at: '2026-06-25T10:00:00Z' }] }))
      .mockResolvedValueOnce(ok({ items: [{ account_id: 'wa-removed', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', display_name: 'Removed Account', external_account_id: 'wa:removed', runtime: 'live_blocked', lifecycle_state: 'removed', created_at: '2026-06-25T10:00:00Z', updated_at: '2026-06-25T10:00:00Z' }] }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappAccounts()
    await fetchWhatsappAccounts(true)

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/accounts')
    expect(fetchMock.mock.calls[0][0]).not.toContain('include_removed=true')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/whatsapp/accounts?include_removed=true')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })

  it('calls account capabilities and runtime lifecycle routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ version: '2.0', runtime_mode: 'synthetic', provider_shapes: [], account_scope: null, capabilities: [], planned_features: [], unsupported_features: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'linked', synthetic_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: true, session_secret_ref: 'secret:wa-1', runtime_blockers: [], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'available', healthy: true, checks: {}, checked_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'available', synthetic_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: true, session_secret_ref: 'secret:wa-1', runtime_blockers: [], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'linked', synthetic_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: true, session_secret_ref: 'secret:wa-1', runtime_blockers: [], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'revoked', synthetic_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: false, pair_code_available: false, media_download_available: false, media_upload_available: false, session_restore_available: false, session_secret_ref: null, runtime_blockers: ['whatsapp_session_revoked'], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'link_required', synthetic_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: false, session_secret_ref: null, runtime_blockers: ['whatsapp_session_link_required'], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'synthetic', status: 'link_required', synthetic_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: false, session_secret_ref: null, runtime_blockers: ['whatsapp_session_link_required'], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', removed: true, unbound_secret_refs: ['secret:wa-1'], removed_at: '2026-06-25T10:00:00Z' }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappAccountCapabilities('wa-1')
    await fetchWhatsappRuntimeStatus('wa-1')
    await fetchWhatsappRuntimeHealth('wa-1')
    await startWhatsappRuntime({ account_id: 'wa-1' })
    await stopWhatsappRuntime({ account_id: 'wa-1' })
    await revokeWhatsappRuntime({ account_id: 'wa-1' })
    await relinkWhatsappRuntime({ account_id: 'wa-1' })
    await rotateWhatsappRuntime({ account_id: 'wa-1' })
    await removeWhatsappRuntime({ account_id: 'wa-1' })

    expect(fetchMock).toHaveBeenCalledTimes(9)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/accounts/wa-1/capabilities')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/whatsapp/runtime/status?account_id=wa-1')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/whatsapp/runtime/health?account_id=wa-1')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/whatsapp/runtime/start')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/integrations/whatsapp/runtime/stop')
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/integrations/whatsapp/runtime/revoke')
    expect(fetchMock.mock.calls[6][0]).toContain('/api/v1/integrations/whatsapp/runtime/relink')
    expect(fetchMock.mock.calls[7][0]).toContain('/api/v1/integrations/whatsapp/runtime/rotate')
    expect(fetchMock.mock.calls[8][0]).toContain('/api/v1/integrations/whatsapp/runtime/remove')
  })

  it('loads provider commands and posts retry/dead-letter actions', async () => {
    const defaultCommandsChunkSize = WHATSAPP_RUNTIME_COMMANDS_PAGE_SIZE
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({
        items: [
          {
            command_id: 'wa-cmd-1',
            account_id: 'wa-1',
            command_kind: 'send_text',
            idempotency_key: 'send:1',
            provider_chat_id: 'chat-1',
            provider_message_id: null,
            capability_state: 'available',
            action_class: 'provider_write',
            confirmation_decision: 'not_required',
            status: 'failed',
            retry_count: 1,
            max_retries: 3,
            last_error: 'temporary failure',
            result_payload: {},
            audit_metadata: {},
            provider_state: {},
            reconciliation_status: 'not_observed',
            next_attempt_at: null,
            last_attempt_at: '2026-06-26T09:00:00Z',
            provider_observed_at: null,
            reconciled_at: null,
            dead_lettered_at: null,
            completed_at: null,
            created_at: '2026-06-26T08:55:00Z',
            updated_at: '2026-06-26T09:00:00Z',
          },
        ],
      }))
      .mockResolvedValueOnce(ok({
        command_id: 'wa-cmd-1',
        account_id: 'wa-1',
        command_kind: 'send_text',
        idempotency_key: 'send:1',
        provider_chat_id: 'chat-1',
        provider_message_id: null,
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'not_required',
        status: 'retrying',
        retry_count: 0,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        provider_state: {},
        reconciliation_status: 'not_observed',
        next_attempt_at: null,
        last_attempt_at: null,
        provider_observed_at: null,
        reconciled_at: null,
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-26T08:55:00Z',
        updated_at: '2026-06-26T09:02:00Z',
      }))
      .mockResolvedValueOnce(ok({
        command_id: 'wa-cmd-2',
        account_id: 'wa-1',
        command_kind: 'send_media',
        idempotency_key: 'media:2',
        provider_chat_id: 'chat-1',
        provider_message_id: null,
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: 'dead_letter',
        retry_count: 2,
        max_retries: 3,
        last_error: 'manual_dead_letter_from_runtime_panel',
        result_payload: {},
        audit_metadata: {},
        provider_state: {},
        reconciliation_status: 'not_observed',
        next_attempt_at: null,
        last_attempt_at: '2026-06-26T09:05:00Z',
        provider_observed_at: null,
        reconciled_at: null,
        dead_lettered_at: '2026-06-26T09:06:00Z',
        completed_at: null,
        created_at: '2026-06-26T08:58:00Z',
        updated_at: '2026-06-26T09:06:00Z',
      }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappProviderCommands({
      account_id: 'wa-1',
      provider_chat_id: 'chat-1',
      command_kinds: ['send_text', 'send_media'],
      limit: defaultCommandsChunkSize,
    })
    await retryWhatsappProviderCommand('wa-cmd-1')
    await deadLetterWhatsappProviderCommand({
      command_id: 'wa-cmd-2',
      reason: 'manual_dead_letter_from_runtime_panel',
    })

    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/commands?')
    expect(fetchMock.mock.calls[0][0]).toContain('account_id=wa-1')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_chat_id=chat-1')
    expect(fetchMock.mock.calls[0][0]).toContain('command_kinds=send_text%2Csend_media')
    expect(fetchMock.mock.calls[0][0]).toContain(`limit=${defaultCommandsChunkSize}`)
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/whatsapp/commands/wa-cmd-1/retry')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/whatsapp/commands/wa-cmd-2/dead-letter')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[2][1].body as string)).toEqual({
      reason: 'manual_dead_letter_from_runtime_panel',
    })
  })

  it('posts projected sync snapshot routes for chats, history, members, statuses, presence, calls, contacts and media', async () => {
    const defaultSyncChunkSize = WHATSAPP_RUNTIME_SYNC_CHUNK_SIZE

    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_chat_id: 'chat-1', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_chat_id: 'chat-1', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_chat_id: 'status-feed', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_chat_id: 'chat-1', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_chat_id: 'chat-1', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_chat_id: 'chat-1', content_type: 'image/', runtime_kind: 'synthetic', status: 'synced', synced_count: 1, has_more: false, items: [] }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappSyncChats({ account_id: 'wa-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncHistory({ account_id: 'wa-1', provider_chat_id: 'chat-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncMembers({ account_id: 'wa-1', provider_chat_id: 'chat-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncStatuses({ account_id: 'wa-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncPresence({ account_id: 'wa-1', provider_chat_id: 'chat-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncCalls({ account_id: 'wa-1', provider_chat_id: 'chat-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncContacts({ account_id: 'wa-1', limit: defaultSyncChunkSize })
    await fetchWhatsappSyncMedia({ account_id: 'wa-1', provider_chat_id: 'chat-1', content_type: 'image/', limit: defaultSyncChunkSize })

    expect(fetchMock).toHaveBeenCalledTimes(8)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/chats')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/history')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/conversations/chat-1/members')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/statuses')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/presence')
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/calls')
    expect(fetchMock.mock.calls[6][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/contacts')
    expect(fetchMock.mock.calls[7][0]).toContain('/api/v1/integrations/whatsapp/provider-sync/media')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(fetchMock.mock.calls[3][1].method).toBe('POST')
    expect(fetchMock.mock.calls[4][1].method).toBe('POST')
    expect(fetchMock.mock.calls[5][1].method).toBe('POST')
    expect(fetchMock.mock.calls[6][1].method).toBe('POST')
    expect(fetchMock.mock.calls[7][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({
      account_id: 'wa-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      account_id: 'wa-1',
      provider_chat_id: 'chat-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[2][1].body as string)).toEqual({
      account_id: 'wa-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[3][1].body as string)).toEqual({
      account_id: 'wa-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[4][1].body as string)).toEqual({
      account_id: 'wa-1',
      provider_chat_id: 'chat-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[5][1].body as string)).toEqual({
      account_id: 'wa-1',
      provider_chat_id: 'chat-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[6][1].body as string)).toEqual({
      account_id: 'wa-1',
      limit: defaultSyncChunkSize,
    })
    expect(JSON.parse(fetchMock.mock.calls[7][1].body as string)).toEqual({
      account_id: 'wa-1',
      provider_chat_id: 'chat-1',
      content_type: 'image/',
      limit: defaultSyncChunkSize,
    })
  })

  it('posts publish status command through provider command route', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi.fn().mockResolvedValueOnce(ok({
      command_id: 'wa-status-1',
      account_id: 'wa-1',
      command_kind: 'publish_status',
      idempotency_key: 'status:1',
      provider_chat_id: 'status-feed',
      provider_message_id: null,
      capability_state: 'blocked',
      action_class: 'provider_write',
      confirmation_decision: 'not_required',
      status: 'blocked',
      retry_count: 0,
      max_retries: 3,
      last_error: 'live_runtime_blocked',
      result_payload: {},
      audit_metadata: {},
      provider_state: {},
      reconciliation_status: 'not_observed',
      next_attempt_at: null,
      last_attempt_at: null,
      provider_observed_at: null,
      reconciled_at: null,
      dead_lettered_at: null,
      completed_at: null,
      created_at: '2026-06-26T09:10:00Z',
      updated_at: '2026-06-26T09:10:00Z',
    }))
    vi.stubGlobal('fetch', fetchMock)

    await publishWhatsappStatus({
      account_id: 'wa-1',
      idempotency_key: 'status:1',
      text: 'Ship it',
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/provider-commands/statuses/publish')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({
      account_id: 'wa-1',
      idempotency_key: 'status:1',
      text: 'Ship it',
    })
  })
})
