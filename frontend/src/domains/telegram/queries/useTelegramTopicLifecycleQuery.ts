import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { createTelegramTopic, toggleTelegramTopicClosed } from '../api/telegramTopics'
import type {
  TelegramTopicCloseRequest,
  TelegramTopicCreateRequest,
} from '../types/telegramTopics'
import { telegramQueryKeys } from './useTelegramQuery'

function newCommandId(): string {
  const random = typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
    ? crypto.randomUUID().slice(0, 12)
    : Math.random().toString(36).slice(2, 14)
  return `tcmd_${Date.now()}_${random}`
}

export function useCreateTelegramTopicMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (request: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      title: string
    }) => {
      const payload: TelegramTopicCreateRequest = {
        command_id: newCommandId(),
        account_id: request.accountId,
        provider_chat_id: request.providerChatId,
        title: request.title.trim(),
      }
      return createTelegramTopic(request.telegramChatId, payload)
    },
    onSuccess: (_response, request) => {
      queryClient.invalidateQueries({ queryKey: [...telegramQueryKeys.topics, request.telegramChatId] })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.topicSearch })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
      queryClient.invalidateQueries({ queryKey: ['telegram', 'commands', request.accountId] })
      queryClient.invalidateQueries({ queryKey: ['telegram', 'commands'] })
    },
  })
}

export function useToggleTelegramTopicClosedMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (request: {
      topicId: string
      telegramChatId: string
      accountId: string
      providerChatId: string
      isClosed: boolean
    }) => {
      const payload: TelegramTopicCloseRequest = {
        command_id: newCommandId(),
        account_id: request.accountId,
        provider_chat_id: request.providerChatId,
        is_closed: request.isClosed,
      }
      return toggleTelegramTopicClosed(request.topicId, payload)
    },
    onSuccess: (_response, request) => {
      queryClient.invalidateQueries({ queryKey: [...telegramQueryKeys.topics, request.telegramChatId] })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.topicSearch })
      queryClient.invalidateQueries({ queryKey: ['telegram', 'commands', request.accountId] })
      queryClient.invalidateQueries({ queryKey: ['telegram', 'commands'] })
    },
  })
}
