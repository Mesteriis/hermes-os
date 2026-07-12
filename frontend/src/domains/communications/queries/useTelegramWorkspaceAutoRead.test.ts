import { effectScope, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import { useTelegramWorkspaceAutoRead } from './useTelegramWorkspaceAutoRead'

const runner = vi.fn()

function chat(id: string, unreadCount: number) {
  return {
    telegram_chat_id: id,
    account_id: 'telegram-account',
    provider_chat_id: `provider-${id}`,
    chat_kind: 'private',
    title: id,
    username: null,
    sync_state: 'synced',
    last_message_at: null,
    metadata: { unread_count: unreadCount },
    created_at: '2026-07-12T10:00:00Z',
    updated_at: '2026-07-12T10:00:00Z',
  } as TelegramChat
}

describe('useTelegramWorkspaceAutoRead', () => {
  afterEach(() => vi.useRealTimers())

  it('marks the opened unread Telegram dialog as read after two seconds', async () => {
    vi.useFakeTimers()
    const selectedChat = ref(chat('chat-1', 2))
    const messagesAreVisible = ref(false)
    const markRead = vi.fn().mockResolvedValue(undefined)
    const scope = effectScope()

    scope.run(() => useTelegramWorkspaceAutoRead(selectedChat, messagesAreVisible, () => runner, markRead, vi.fn()))
    await vi.advanceTimersByTimeAsync(2_000)
    expect(markRead).not.toHaveBeenCalled()

    messagesAreVisible.value = true
    await vi.advanceTimersByTimeAsync(2_000)

    expect(markRead).toHaveBeenCalledWith(selectedChat.value, runner)
    scope.stop()
  })

  it('cancels the pending provider command when the user switches dialogs', async () => {
    vi.useFakeTimers()
    const selectedChat = ref(chat('chat-1', 1))
    const messagesAreVisible = ref(true)
    const markRead = vi.fn().mockResolvedValue(undefined)
    const scope = effectScope()

    scope.run(() => useTelegramWorkspaceAutoRead(selectedChat, messagesAreVisible, () => runner, markRead, vi.fn()))
    await vi.advanceTimersByTimeAsync(1_000)
    selectedChat.value = chat('chat-2', 0)
    await nextTick()
    await vi.advanceTimersByTimeAsync(2_000)

    expect(markRead).not.toHaveBeenCalled()
    scope.stop()
  })
})
