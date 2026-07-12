import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import type { TelegramConversationRuntimeActionRunner } from '@/shared/communications/types/telegramRuntimeActions'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import { useDelayedMessageRead } from './useDelayedMessageRead'
import { telegramChatNeedsRead } from './telegramWorkspacePresentation'

export function useTelegramWorkspaceAutoRead(
  selectedChat: MaybeRefOrGetter<TelegramChat | null>,
  messagesAreVisible: MaybeRefOrGetter<boolean>,
  resolveRunner: () => TelegramConversationRuntimeActionRunner | undefined,
  markRead: (chat: TelegramChat, runner: TelegramConversationRuntimeActionRunner) => Promise<void>,
  onError: (error: unknown) => void
) {
  const unreadChatId = computed(() => {
    const chat = toValue(selectedChat)
    return chat && toValue(messagesAreVisible) && telegramChatNeedsRead(chat) && resolveRunner()
      ? chat.telegram_chat_id
      : null
  })

  useDelayedMessageRead(unreadChatId, async (chatId) => {
    const chat = toValue(selectedChat)
    const runner = resolveRunner()
    if (!chat || !toValue(messagesAreVisible) || chat.telegram_chat_id !== chatId || !runner || !telegramChatNeedsRead(chat)) return
    await markRead(chat, runner)
  }, onError)
}
