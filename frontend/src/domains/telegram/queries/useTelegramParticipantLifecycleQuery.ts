import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { joinTelegramChat, leaveTelegramChat } from '../api/telegram'
import { patchTelegramCommandList } from './realtimeTelegramCommandPatches'
import { telegramQueryKeys } from './useTelegramQuery'
import type {
  TelegramChatLifecycleCommandResponse,
  TelegramProviderWriteCommand,
} from '../types/telegram'

type TelegramParticipantLifecycleInput = {
  telegramChatId?: string | null
  accountId: string
  providerChatId: string
}

type TelegramParticipantLifecycleQueryClient = Pick<
  ReturnType<typeof useQueryClient>,
  'getQueriesData' | 'setQueryData'
>

export function primeTelegramParticipantLifecycleCommandCache(
  queryClient: TelegramParticipantLifecycleQueryClient,
  accountId: string,
  command: TelegramChatLifecycleCommandResponse
) {
  if (!queryClient.getQueriesData || !queryClient.setQueryData) return

  const payload = {
    account_id: accountId,
    provider_chat_id: command.provider_chat_id,
    telegram_chat_id: command.telegram_chat_id,
    action: command.action,
    status: command.status,
    command_id: command.command_id,
    capability_state: 'available',
    action_class: 'provider_write',
    confirmation_decision: 'confirmed',
    target_ref: {
      provider_chat_id: command.provider_chat_id,
      telegram_chat_id: command.telegram_chat_id,
    },
    payload: {
      provider_chat_id: command.provider_chat_id,
      telegram_chat_id: command.telegram_chat_id,
      action: command.action,
    },
  }

  for (const [queryKey] of queryClient.getQueriesData<TelegramProviderWriteCommand[]>({
    queryKey: ['telegram', 'commands'],
  })) {
    queryClient.setQueryData<TelegramProviderWriteCommand[] | undefined>(
      queryKey,
      (cachedCommands) =>
        patchTelegramCommandList(
          queryKey,
          cachedCommands,
          'telegram.command.status_changed',
          payload
        )
    )
  }
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
    onSuccess: (command, variables) => {
      primeTelegramParticipantLifecycleCommandCache(queryClient, variables.accountId, command)
      invalidateParticipantLifecycleCaches(queryClient)
    },
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
    onSuccess: (command, variables) => {
      primeTelegramParticipantLifecycleCommandCache(queryClient, variables.accountId, command)
      invalidateParticipantLifecycleCaches(queryClient)
    },
  })
}
