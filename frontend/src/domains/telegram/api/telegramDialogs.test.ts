import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  addTelegramChatToFolder,
  archiveTelegramChat,
  fetchTelegramCalls,
  fetchTelegramCallTranscript,
  fetchTelegramAccountCapabilities,
  fetchTelegramChatDetail,
  fetchTelegramFolders,
  fetchTelegramChatMembers,
  joinTelegramChat,
  leaveTelegramChat,
  logoutTelegramAccount,
  markTelegramChatRead,
  markTelegramChatUnread,
  muteTelegramChat,
  pinTelegramChat,
  reassignTelegramChatFolders,
  removeTelegramAccount,
  removeTelegramChatFromFolder,
  restartTelegramRuntime,
  setupTelegramAccount,
  stopTelegramRuntime,
  syncTelegramChatMembers,
  unarchiveTelegramChat,
  unmuteTelegramChat,
  unpinTelegramChat,
} from './telegram'

describe('telegram dialog action API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  it('posts runtime restart requests for a selected telegram account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ account_id: 'acc-1', runtime_kind: 'fixture', status: 'running', blockers: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await restartTelegramRuntime({ account_id: 'acc-1' })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/runtime/restart')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({ account_id: 'acc-1' })
  })

  it('posts runtime stop requests for a selected telegram account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ account_id: 'acc-1', runtime_kind: 'fixture', status: 'stopped', blockers: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await stopTelegramRuntime({ account_id: 'acc-1' })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/runtime/stop')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({ account_id: 'acc-1' })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('loads projected chat detail and members routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { telegram_chat_id: 'tgchat-1' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [], next_cursor: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', synced_count: 0, items: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramChatDetail('tgchat-1')
    await fetchTelegramChatMembers('tgchat-1', 25, 'owner', 'admin', '50')
    await syncTelegramChatMembers('tgchat-1')

    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/chats/tgchat-1/members?limit=25&query=owner&role=admin&cursor=50')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/telegram/chats/tgchat-1/members/sync')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
  })

  it('loads projection-backed telegram folders for the selected account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [{ id: 'local:all', label: 'All', source: 'local', count: 2, icon: 'tabler:message' }] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramFolders('acc-1')

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/folders?account_id=acc-1')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
  })

  it('loads account-scoped capability routes for a selected telegram account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ version: '2.0', runtime_mode: 'fixture', capabilities: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramAccountCapabilities('acc-1')

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/accounts/acc-1/capabilities')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
  })

  it('loads projected call metadata and transcript routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [{ call_id: 'call-1', account_id: 'acc-1', provider_chat_id: 'chat-1', status: 'ended', occurred_at: '2026-06-06T12:20:00Z' }] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ transcript: { transcript_id: 'tx-1', call_id: 'call-1', account_id: 'acc-1', provider_chat_id: 'chat-1', transcript_status: 'succeeded', stt_provider: 'fixture-stt', source_audio_ref: 'audio.wav', language_code: 'en', transcript_text: 'Follow up on the Telegram call.', segments: [], provenance: {}, created_at: '2026-06-06T12:21:00Z', updated_at: '2026-06-06T12:21:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramCalls('acc-1', 10)
    await fetchTelegramCallTranscript('call-1')

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/calls?limit=10&account_id=acc-1')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/calls/call-1/transcript')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })

  it('posts account setup and lifecycle routes for telegram accounts', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ account_id: 'acc-1', runtime: 'live_blocked', provider_kind: 'telegram_user', transcription_enabled: false, credential_bindings: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ account: { account_id: 'acc-1' }, stopped_runtime_actor: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ account: { account_id: 'acc-1' }, stopped_runtime_actor: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await setupTelegramAccount({
      account_id: 'acc-1',
      provider_kind: 'telegram_user',
      display_name: 'Account One',
      external_account_id: 'telegram:1',
      tdlib_data_path: '/tmp/telegram-1',
      qr_authorized: true,
      transcription_enabled: false,
    })
    await logoutTelegramAccount('acc-1')
    await removeTelegramAccount('acc-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/accounts')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/accounts/acc-1/logout')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/telegram/accounts/acc-1')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(fetchMock.mock.calls[2][1].method).toBe('DELETE')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toMatchObject({
      account_id: 'acc-1',
      provider_kind: 'telegram_user',
      qr_authorized: true,
    })
  })

  it('posts pin and unpin requests for projected Telegram chats', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'pin', status: 'pinned', metadata: {} }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'unpin', status: 'unpinned', metadata: {} }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await pinTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await unpinTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1/pin')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/chats/tgchat-1/unpin')
  })

  it('posts archive/mute dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockImplementation(() =>
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'ok', status: 'ok', metadata: {} }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await archiveTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await unarchiveTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await muteTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await unmuteTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1/archive')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/chats/tgchat-1/unarchive')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/telegram/chats/tgchat-1/mute')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/telegram/chats/tgchat-1/unmute')
    for (const [, init] of fetchMock.mock.calls) {
      expect(init.method).toBe('POST')
      expect(JSON.parse(init.body as string)).toEqual({
        account_id: 'acc-1',
        provider_chat_id: 'provider-chat-1',
      })
    }
  })

  it('posts add-to-folder dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        new Response(JSON.stringify({
          telegram_chat_id: 'tgchat-1',
          provider_chat_id: 'provider-chat-1',
          action: 'folder_add',
          status: 'queued',
          command_id: 'cmd-folder-add',
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await addTelegramChatToFolder('tgchat-1', 7, {
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1/folders/7')
    expect(fetchMock.mock.calls[0][1]?.method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1]?.body as string)).toEqual({
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })
  })

  it('posts remove-from-folder dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        new Response(JSON.stringify({
          telegram_chat_id: 'tgchat-1',
          provider_chat_id: 'provider-chat-1',
          action: 'folder_remove',
          status: 'queued',
          command_id: 'cmd-folder-remove',
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await removeTelegramChatFromFolder('tgchat-1', 7, {
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1/folders/7/remove')
    expect(fetchMock.mock.calls[0][1]?.method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1]?.body as string)).toEqual({
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })
  })

  it('posts folder reassignment requests for projected Telegram chats', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        new Response(JSON.stringify({
          telegram_chat_id: 'tgchat-1',
          provider_chat_id: 'provider-chat-1',
          action: 'folder_reassign',
          status: 'queued',
          command_ids: ['cmd-folder-add', 'cmd-folder-remove'],
          added_provider_folder_ids: [11],
          removed_provider_folder_ids: [7],
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await reassignTelegramChatFolders('tgchat-1', {
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
      target_provider_folder_ids: [11],
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1/folders/reassign')
    expect(fetchMock.mock.calls[0][1]?.method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1]?.body as string)).toEqual({
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
      target_provider_folder_ids: [11],
    })
  })

  it('posts participant join and leave lifecycle requests through command routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          telegram_chat_id: null,
          provider_chat_id: 'provider-chat-1',
          action: 'join',
          status: 'queued',
          command_id: 'cmd-join',
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          telegram_chat_id: 'tgchat-1',
          provider_chat_id: 'provider-chat-1',
          action: 'leave',
          status: 'queued',
          command_id: 'cmd-leave',
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await joinTelegramChat({ account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await leaveTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/join')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/chats/tgchat-1/leave')
    for (const [, init] of fetchMock.mock.calls) {
      expect(init.method).toBe('POST')
      expect(JSON.parse(init.body as string)).toEqual({
        account_id: 'acc-1',
        provider_chat_id: 'provider-chat-1',
      })
    }
  })

  it('posts local read and unread dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockImplementation(() =>
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'ok', status: 'ok', metadata: { unread_count: 0 } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await markTelegramChatRead('tgchat-1', {
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
      last_read_inbox_provider_message_id: 'provider-chat-1:777',
    })
    await markTelegramChatUnread('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/chats/tgchat-1/read')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/chats/tgchat-1/unread')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
      last_read_inbox_provider_message_id: 'provider-chat-1:777',
    })
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })
  })
})
