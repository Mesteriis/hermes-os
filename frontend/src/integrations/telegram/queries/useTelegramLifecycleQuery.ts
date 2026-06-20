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
    queryKey: computed(() => ['integrations', 'telegram', 'message-versions', toValue(messageId)]),
    queryFn: () => fetchTelegramMessageVersions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageTombstonesQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageTombstoneListResponse>({
    queryKey: computed(() => ['integrations', 'telegram', 'message-tombstones', toValue(messageId)]),
    queryFn: () => fetchTelegramMessageTombstones(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageReactionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReactionListResponse>({
    queryKey: computed(() => ['integrations', 'telegram', 'message-reactions', toValue(messageId)]),
    queryFn: () => fetchTelegramReactions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramCommandsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 25,
  enabled: MaybeRefOrGetter<boolean> = true,
  filters?: {
    providerChatId?: MaybeRefOrGetter<string | null | undefined>
    providerMessageId?: MaybeRefOrGetter<string | null | undefined>
    commandKinds?: MaybeRefOrGetter<string[] | null | undefined>
  }
) {
  return useQuery<TelegramProviderWriteCommand[]>({
    queryKey: computed(() => {
      const commandKinds = [...(toValue(filters?.commandKinds) ?? [])]
        .filter((value) => value.trim().length > 0)
        .sort()
      return [
        'integrations',
        'telegram',
        'runtime',
        'commands',
        toValue(accountId) ?? 'none',
        toValue(limit),
        toValue(filters?.providerChatId) ?? 'all',
        toValue(filters?.providerMessageId) ?? 'all',
        commandKinds.length > 0 ? commandKinds.join('|') : 'all',
      ]
    }),
    queryFn: async () => {
      const response = await fetchTelegramCommands(toValue(accountId) as string, toValue(limit), {
        providerChatId: toValue(filters?.providerChatId),
        providerMessageId: toValue(filters?.providerMessageId),
        commandKinds: toValue(filters?.commandKinds) ?? undefined,
      })
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
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands', command.account_id] })
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands'] })
    },
  })
}
