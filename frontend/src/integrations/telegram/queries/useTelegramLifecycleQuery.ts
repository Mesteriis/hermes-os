import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramCommands,
  retryTelegramCommand,
} from '../api/telegramLifecycle'
import type {
  TelegramProviderWriteCommand,
} from '../types/telegram'

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
