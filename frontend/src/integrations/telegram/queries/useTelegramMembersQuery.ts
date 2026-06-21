import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { syncTelegramChatMembers } from '../api/telegram'
import { telegramQueryKeys } from './useTelegramQuery'

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
