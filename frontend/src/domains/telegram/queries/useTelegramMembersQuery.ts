import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { fetchTelegramChatMembers, syncTelegramChatMembers } from '../api/telegram'
import type { TelegramChatMember, TelegramChatMemberListResponse } from '../types/telegram'
import { telegramQueryKeys } from './useTelegramQuery'

export function useTelegramChatMembersQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50,
  query: MaybeRefOrGetter<string | null | undefined> = '',
  role: MaybeRefOrGetter<string | null | undefined> = ''
) {
  return useInfiniteQuery<
    TelegramChatMemberListResponse,
    Error,
    TelegramChatMember[],
    readonly unknown[],
    string | null
  >({
    queryKey: computed(() => [
      ...telegramQueryKeys.chatMembers,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
      normalizeQueryValue(toValue(query)),
      normalizeQueryValue(toValue(role)),
    ]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      const value = toValue(telegramChatId)
      if (!value) return { items: [], next_cursor: null }
      return fetchTelegramChatMembers(
        value,
        toValue(limit),
        normalizeQueryValue(toValue(query)) || undefined,
        normalizeQueryValue(toValue(role)) || undefined,
        pageParam ?? undefined
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useSyncTelegramChatMembersMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (telegramChatId: string) => syncTelegramChatMembers(telegramChatId),
    onSuccess: (_response, telegramChatId) => {
      queryClient.invalidateQueries({ queryKey: [...telegramQueryKeys.chatMembers, telegramChatId] })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

function normalizeQueryValue(value: string | null | undefined): string {
  return value?.trim() ?? ''
}
