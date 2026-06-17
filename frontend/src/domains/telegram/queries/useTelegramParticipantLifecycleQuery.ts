import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { joinTelegramChat, leaveTelegramChat } from '../api/telegram'
import { telegramQueryKeys } from './useTelegramQuery'

type TelegramParticipantLifecycleInput = {
  telegramChatId?: string | null
  accountId: string
  providerChatId: string
}

function invalidateParticipantLifecycleCaches(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatMembers })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
  queryClient.invalidateQueries({ queryKey: ['telegram', 'commands'] })
}

export function useJoinTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ accountId, providerChatId }: TelegramParticipantLifecycleInput) =>
      joinTelegramChat({ account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => invalidateParticipantLifecycleCaches(queryClient),
  })
}

export function useLeaveTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: TelegramParticipantLifecycleInput) => {
      if (!telegramChatId?.trim()) {
        throw new Error('Telegram chat id is required for leave command')
      }
      return leaveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId })
    },
    onSuccess: () => invalidateParticipantLifecycleCaches(queryClient),
  })
}
