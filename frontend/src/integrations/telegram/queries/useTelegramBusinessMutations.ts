import { useMutation, useQueryClient } from '@tanstack/vue-query'
import type {
  TelegramLifecycleResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
} from '../../../shared/communications/types/telegram'
import {
  addTelegramBusinessReaction,
  deleteTelegramBusinessMessage,
  editTelegramBusinessMessage,
  forwardTelegramBusinessMessage,
  markTelegramBusinessMessageRead,
  pinTelegramBusinessMessage,
  removeTelegramBusinessReaction,
  replyToTelegramBusinessMessage,
  restoreTelegramBusinessMessageVisibility,
  sendTelegramBusinessMessage,
} from '../api/telegramBusiness'
import type {
  TelegramBusinessMessageCommandResponse,
  TelegramBusinessMessagePinResponse,
} from '../types/business'
import { telegramBusinessQueryKeys } from './telegramBusinessQueryKeys'

function useInvalidateTelegramBusinessState() {
  const queryClient = useQueryClient()
  return () => {
    queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.messages })
    queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chats })
    queryClient.invalidateQueries({ queryKey: ['communications', 'messages'] })
  }
}

export function useSendTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramBusinessMessageCommandResponse, Error, { account_id: string; provider_chat_id: string; text: string }>({
    mutationFn: (request) => sendTelegramBusinessMessage(request),
    onSuccess: invalidate,
  })
}

export function useReplyTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramBusinessMessageCommandResponse, Error, {
    message_id: string
    account_id?: string
    provider_chat_id?: string
    reply_to_provider_message_id?: string
    text: string
  }>({
    mutationFn: (request) => replyToTelegramBusinessMessage({ message_id: request.message_id, text: request.text }),
    onSuccess: invalidate,
  })
}

export function useForwardTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramBusinessMessageCommandResponse, Error, {
    message_id: string
    account_id?: string
    provider_chat_id: string
    from_provider_chat_id?: string
    from_provider_message_id?: string
  }>({
    mutationFn: (request) => forwardTelegramBusinessMessage({
      message_id: request.message_id,
      provider_chat_id: request.provider_chat_id,
    }),
    onSuccess: invalidate,
  })
}

export function useEditTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramLifecycleResponse, Error, Parameters<typeof editTelegramBusinessMessage>[0]>({
    mutationFn: editTelegramBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useDeleteTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramLifecycleResponse, Error, Parameters<typeof deleteTelegramBusinessMessage>[0]>({
    mutationFn: deleteTelegramBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useRestoreTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramLifecycleResponse, Error, Parameters<typeof restoreTelegramBusinessMessageVisibility>[0]>({
    mutationFn: restoreTelegramBusinessMessageVisibility,
    onSuccess: invalidate,
  })
}

export function usePinTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramBusinessMessagePinResponse, Error, { message_id: string }>({
    mutationFn: pinTelegramBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useMarkReadTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation({ mutationFn: markTelegramBusinessMessageRead, onSuccess: invalidate })
}

export function useAddTelegramReactionMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramReactionResponse, Error, { messageId: string; request: TelegramReactionRequest }>({
    mutationFn: ({ messageId, request }) => addTelegramBusinessReaction(messageId, request),
    onSuccess: invalidate,
  })
}

export function useRemoveTelegramReactionMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramReactionResponse, Error, { messageId: string; request: TelegramReactionRequest }>({
    mutationFn: ({ messageId, request }) => removeTelegramBusinessReaction(messageId, request),
    onSuccess: invalidate,
  })
}
