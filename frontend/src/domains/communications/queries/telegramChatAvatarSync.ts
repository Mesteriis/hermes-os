import { ref } from 'vue'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import { isRecord } from '@/shared/communications/queries/realtimePatchShared'
import {
  fetchTelegramBusinessChatAvatar,
  syncTelegramBusinessChatAvatar,
} from '../api/telegramBusinessApi'

const AVATAR_SYNC_CONCURRENCY = 3

export function createTelegramChatAvatarSynchronizer() {
  const sources = ref<Record<string, string>>({})
  const requestedChatIds = new Set<string>()

  async function sync(chats: readonly TelegramChat[]): Promise<void> {
    const pending = chats.filter((chat) => hasProviderAvatar(chat) && !requestedChatIds.has(chat.telegram_chat_id))
    for (let index = 0; index < pending.length; index += AVATAR_SYNC_CONCURRENCY) {
      await Promise.all(pending.slice(index, index + AVATAR_SYNC_CONCURRENCY).map(loadAvatar))
    }
  }

  async function loadAvatar(chat: TelegramChat): Promise<void> {
    requestedChatIds.add(chat.telegram_chat_id)
    try {
      await syncTelegramBusinessChatAvatar(chat.telegram_chat_id)
      const blob = await fetchTelegramBusinessChatAvatar(chat.telegram_chat_id)
      if (!blob.type.startsWith('image/') || typeof URL.createObjectURL !== 'function') return
      const previous = sources.value[chat.telegram_chat_id]
      if (previous) URL.revokeObjectURL(previous)
      sources.value = {
        ...sources.value,
        [chat.telegram_chat_id]: URL.createObjectURL(blob),
      }
    } catch {
      // A missing/temporarily unavailable avatar must not break the dialog list.
      requestedChatIds.delete(chat.telegram_chat_id)
    }
  }

  function sourceFor(telegramChatId: string): string | undefined {
    return sources.value[telegramChatId]
  }

  function dispose(): void {
    for (const source of Object.values(sources.value)) {
      URL.revokeObjectURL(source)
    }
    sources.value = {}
  }

  return { dispose, sourceFor, sync, sources }
}

function hasProviderAvatar(chat: TelegramChat): boolean {
  const avatar = chat.metadata.avatar
  if (!isRecord(avatar)) return false
  const fileId = avatar.tdlib_file_id
  return typeof fileId === 'number' && Number.isSafeInteger(fileId) && fileId > 0
}
