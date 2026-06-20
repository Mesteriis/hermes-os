import { computed, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { telegramChatGroupFilters } from '../stores/telegram'
import type { TelegramChat, TelegramChatGroupFilter } from '../types/telegram'
import { useTelegramFoldersQuery } from './useTelegramQuery'

export function resolveTelegramGroupFilters(
  chats: TelegramChat[],
  serverFilters: TelegramChatGroupFilter[] | null | undefined
): TelegramChatGroupFilter[] {
  if (serverFilters?.length) {
    return serverFilters
  }
  return telegramChatGroupFilters(chats)
}

export function useTelegramFolderFilters(
  chats: MaybeRefOrGetter<TelegramChat[]>,
  accountId: MaybeRefOrGetter<string | null | undefined>,
  activeGroupFilter: MaybeRefOrGetter<string>,
  onInvalidSelection: (fallbackFilterId: string) => void
) {
  const foldersQuery = useTelegramFoldersQuery(computed(() => toValue(accountId) ?? undefined))
  const groupFilters = computed(() =>
    resolveTelegramGroupFilters(toValue(chats), foldersQuery.data.value ?? null)
  )

  watch(
    groupFilters,
    (groups) => {
      if (!groups.some((group) => group.id === toValue(activeGroupFilter))) {
        onInvalidSelection('local:all')
      }
    },
    { immediate: true }
  )

  return {
    foldersQuery,
    groupFilters,
  }
}
