import { describe, expect, it, vi } from 'vitest'
import {
  createTelegramInitialHistorySynchronizer,
  waitForTelegramHistoryProjection,
} from './telegramInitialHistorySync'

const chat = {
  telegram_chat_id: 'chat-1',
  account_id: 'account-1',
  provider_chat_id: 'provider-chat-1',
  title: 'Test chat',
  username: null,
  chat_kind: 'private' as const,
  last_message_at: null,
  metadata: {},
  created_at: '2026-07-12T00:00:00Z',
  updated_at: '2026-07-12T00:00:00Z',
  sync_state: 'synced',
}

describe('Telegram initial history synchronizer', () => {
  it('loads private dialog history in full once through the provider read runner', async () => {
    const runner = vi.fn().mockResolvedValue(undefined)
    const refetchMessages = vi.fn().mockResolvedValue(undefined)
    const setError = vi.fn()
    const setSyncing = vi.fn()
    const waitForHistoryProjection = vi.fn(async () => {
      await refetchMessages()
    })
    const sync = createTelegramInitialHistorySynchronizer({
      resolveRunner: () => runner,
      refetchMessages,
      messageCount: () => 0,
      waitForHistoryProjection,
      setError,
      setSyncing,
    })

    await sync(chat)
    await sync(chat)

    expect(runner).toHaveBeenCalledOnce()
    expect(runner).toHaveBeenCalledWith({
      action: 'sync_full',
      accountId: 'account-1',
      providerChatId: 'provider-chat-1',
      telegramChatId: 'chat-1',
    })
    expect(refetchMessages).toHaveBeenCalledOnce()
    expect(waitForHistoryProjection).toHaveBeenCalledWith(0)
    expect(setError).toHaveBeenCalledWith('')
    expect(setSyncing.mock.calls).toEqual([[true], [false]])
  })

  it('keeps group history lazy until its full-history policy is enabled', async () => {
    const runner = vi.fn().mockResolvedValue(undefined)
    const waitForHistoryProjection = vi.fn().mockResolvedValue(undefined)
    const sync = createTelegramInitialHistorySynchronizer({
      resolveRunner: () => runner,
      refetchMessages: vi.fn().mockResolvedValue(undefined),
      messageCount: () => 0,
      waitForHistoryProjection,
      setError: vi.fn(),
      setSyncing: vi.fn(),
    })

    await sync({ ...chat, chat_kind: 'group' })

    expect(runner).toHaveBeenCalledWith(expect.objectContaining({ action: 'sync_latest' }))
  })

  it('allows a retry after a transient provider read failure', async () => {
    const runner = vi.fn()
      .mockRejectedValueOnce(new Error('TDLib reconnecting'))
      .mockResolvedValueOnce(undefined)
    const waitForHistoryProjection = vi.fn().mockResolvedValue(undefined)
    const sync = createTelegramInitialHistorySynchronizer({
      resolveRunner: () => runner,
      refetchMessages: vi.fn().mockResolvedValue(undefined),
      messageCount: () => 0,
      waitForHistoryProjection,
      setError: vi.fn(),
      setSyncing: vi.fn(),
    })

    await sync(chat)
    await sync(chat)

    expect(runner).toHaveBeenCalledTimes(2)
  })

  it('waits for the durable projection before releasing the dialog', async () => {
    let count = 0
    const refetchMessages = vi.fn().mockImplementation(async () => {
      count += 1
    })
    const wait = vi.fn().mockResolvedValue(undefined)

    await waitForTelegramHistoryProjection({
      initialMessageCount: 0,
      messageCount: () => count,
      refetchMessages,
      wait,
    })

    expect(refetchMessages).toHaveBeenCalledOnce()
    expect(wait).not.toHaveBeenCalled()
  })

  it('keeps refetching while the projection is still pending', async () => {
    let count = 0
    const refetchMessages = vi.fn().mockResolvedValue(undefined)
    const wait = vi.fn().mockImplementation(async () => {
      count = 1
    })

    await waitForTelegramHistoryProjection({
      initialMessageCount: 0,
      messageCount: () => count,
      refetchMessages,
      wait,
    })

    expect(refetchMessages).toHaveBeenCalledTimes(2)
    expect(wait).toHaveBeenCalledWith(250)
  })
})
