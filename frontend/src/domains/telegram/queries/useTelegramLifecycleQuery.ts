import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramCommands,
  fetchTelegramMessageTombstones,
  fetchTelegramMessageVersions,
  fetchTelegramReactions,
  retryTelegramCommand,
} from '../api/telegramLifecycle'
import type {
  TelegramMessageTombstoneListResponse,
  TelegramMessageVersionListResponse,
  TelegramProviderWriteCommand,
  TelegramReactionListResponse,
} from '../types/telegram'

export function useTelegramMessageVersionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageVersionListResponse>({
    queryKey: computed(() => ['telegram', 'message-versions', toValue(messageId)]),
    queryFn: () => fetchTelegramMessageVersions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageTombstonesQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageTombstoneListResponse>({
    queryKey: computed(() => ['telegram', 'message-tombstones', toValue(messageId)]),
    queryFn: () => fetchTelegramMessageTombstones(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageReactionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReactionListResponse>({
    queryKey: computed(() => ['telegram', 'message-reactions', toValue(messageId)]),
    queryFn: () => fetchTelegramReactions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramCommandsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 25,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramProviderWriteCommand[]>({
    queryKey: computed(() => ['telegram', 'commands', toValue(accountId) ?? 'none', toValue(limit)]),
    queryFn: async () => {
      const response = await fetchTelegramCommands(toValue(accountId) as string, toValue(limit))
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramCommandRetryMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: retryTelegramCommand,
    onSuccess: (command) => {
      queryClient.invalidateQueries({ queryKey: ['telegram', 'commands', command.account_id] })
      queryClient.invalidateQueries({ queryKey: ['telegram', 'commands'] })
    },
  })
}
