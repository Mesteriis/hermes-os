import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  addWhatsappBusinessReaction,
  archiveWhatsappBusinessConversation,
  deleteWhatsappBusinessMessage,
  editWhatsappBusinessMessage,
  fetchWhatsappBusinessReactions,
  fetchWhatsappWebBusinessConversationDetail,
  fetchWhatsappWebBusinessConversationMembers,
  fetchWhatsappWebBusinessConversations,
  fetchWhatsappWebBusinessMessages,
  fetchWhatsappWebBusinessPinnedMessages,
  forwardWhatsappBusinessMessage,
  markWhatsappBusinessConversationRead,
  markWhatsappBusinessConversationUnread,
  muteWhatsappBusinessConversation,
  pinWhatsappBusinessConversation,
  pinWhatsappBusinessMessage,
  replyToWhatsappBusinessMessage,
  removeWhatsappBusinessReaction,
  sendWhatsappBusinessMessage,
  searchWhatsappWebBusinessMedia,
  searchWhatsappWebBusinessMessages,
  unarchiveWhatsappBusinessConversation,
  unmuteWhatsappBusinessConversation,
  unpinWhatsappBusinessConversation,
} from '../api/whatsappBusinessApi'
import type {
  WhatsAppLifecycleResponse,
  WhatsappWebMediaSearchResponse,
  WhatsappWebMessage,
  WhatsappWebMessageSearchResponse,
} from '../../../shared/communications/types/whatsapp'
import type {
  TelegramChatMember,
  TelegramChatMemberListResponse,
} from '../../../shared/communications/types/telegramMembers'
import type {
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
} from '../../../shared/communications/types/telegram'
import type { CommunicationProviderConversation } from '../types/providerChannels'
import type {
  CommunicationProviderMessageListResponse,
} from '../types/providerChannels'
import type {
  CommunicationProviderMessageCommandResponse,
  ConversationPinToggleResponse,
  MessagePinToggleResponse,
} from '../types/communications'

export const whatsappBusinessQueryKeys = {
  conversations: ['communications', 'whatsapp', 'conversations'] as const,
  conversationDetail: ['communications', 'whatsapp', 'conversation-detail'] as const,
  chatMembers: ['communications', 'whatsapp', 'chat-members'] as const,
  messages: ['communications', 'whatsapp', 'messages'] as const,
  search: ['communications', 'whatsapp', 'search'] as const,
}

export function useWhatsappBusinessConversationsQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<CommunicationProviderConversation[]>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.conversations,
      toValue(accountId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappWebBusinessConversations(
        toValue(accountId) ?? undefined,
        toValue(limit)
      )
      return response.items
    },
  })
}

export function useWhatsappBusinessMessagesQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.messages,
      toValue(accountId) ?? 'all',
      toValue(providerChatId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappWebBusinessMessages(
        toValue(accountId) ?? undefined,
        toValue(providerChatId) ?? undefined,
        toValue(limit)
      )
      return response.items
    },
    enabled: computed(() => {
      const providerChatIdValue = toValue(providerChatId)
      if (providerChatIdValue === null) return false
      return providerChatIdValue === undefined || Boolean(toValue(accountId) && providerChatIdValue)
    }),
  })
}

export function useWhatsappConversationDetailQuery(
  conversationId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<CommunicationProviderConversation | null>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.conversationDetail,
      toValue(conversationId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(conversationId)
      if (!value) return null
      const response = await fetchWhatsappWebBusinessConversationDetail(value)
      return response.item
    },
    enabled: computed(() => Boolean(toValue(conversationId))),
  })
}

export function useWhatsappConversationMembersQuery(
  conversationId: MaybeRefOrGetter<string | null | undefined>,
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
      ...whatsappBusinessQueryKeys.chatMembers,
      toValue(conversationId) ?? 'none',
      toValue(limit),
      normalizeWhatsappQueryValue(toValue(query)),
      normalizeWhatsappQueryValue(toValue(role)),
    ]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      const value = toValue(conversationId)
      if (!value) return { items: [], next_cursor: null }
      return fetchWhatsappWebBusinessConversationMembers(
        value,
        toValue(limit),
        normalizeWhatsappQueryValue(toValue(query)) || undefined,
        normalizeWhatsappQueryValue(toValue(role)) || undefined,
        pageParam ?? undefined
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    enabled: computed(() => Boolean(toValue(conversationId))),
  })
}

export function useWhatsappMessageSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<WhatsappWebMessageSearchResponse>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.search,
      'messages',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.limit) ?? 50,
    ]),
    queryFn: () =>
      searchWhatsappWebBusinessMessages({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        limit: toValue(params.limit) ?? 50,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useWhatsappMediaSearchQuery(params: {
  q?: MaybeRefOrGetter<string | null | undefined>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  kind?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<WhatsappWebMediaSearchResponse>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.search,
      'media',
      toValue(params.q)?.trim() ?? '',
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.kind) ?? 'all',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      searchWhatsappWebBusinessMedia({
        q: toValue(params.q) ?? undefined,
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        kind: toValue(params.kind) ?? undefined,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.accountId) && toValue(params.providerChatId))),
  })
}

export function useWhatsappPinnedMessagesQuery(params: {
  conversationId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<CommunicationProviderMessageListResponse>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.conversations,
      toValue(params.conversationId) ?? 'none',
      'pinned-messages',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      fetchWhatsappWebBusinessPinnedMessages({
        conversation_id: toValue(params.conversationId) as string,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.conversationId))),
  })
}

export function useWhatsappMessageReactionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReactionListResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'reactions']),
    queryFn: () => fetchWhatsappBusinessReactions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

function normalizeWhatsappQueryValue(value: string | null | undefined): string {
  return value?.trim() ?? ''
}

function useInvalidateWhatsappBusinessState() {
  const queryClient = useQueryClient()
  return () => {
    queryClient.invalidateQueries({ queryKey: whatsappBusinessQueryKeys.messages })
    queryClient.invalidateQueries({ queryKey: whatsappBusinessQueryKeys.conversations })
    queryClient.invalidateQueries({ queryKey: whatsappBusinessQueryKeys.conversationDetail })
    queryClient.invalidateQueries({ queryKey: ['communications', 'messages'] })
  }
}

export function useSendWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    CommunicationProviderMessageCommandResponse,
    Error,
    { account_id: string; provider_chat_id: string; text: string }
  >({
    mutationFn: sendWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useReplyWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    CommunicationProviderMessageCommandResponse,
    Error,
    {
      message_id: string
      account_id?: string
      provider_chat_id?: string
      reply_to_provider_message_id?: string
      text: string
    }
  >({
    mutationFn: (request) =>
      replyToWhatsappBusinessMessage({ message_id: request.message_id, text: request.text }),
    onSuccess: invalidate,
  })
}

export function useForwardWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    CommunicationProviderMessageCommandResponse,
    Error,
    {
      message_id: string
      account_id?: string
      provider_chat_id: string
      from_provider_chat_id?: string
      from_provider_message_id?: string
    }
  >({
    mutationFn: (request) =>
      forwardWhatsappBusinessMessage({
        message_id: request.message_id,
        provider_chat_id: request.provider_chat_id,
      }),
    onSuccess: invalidate,
  })
}

export function useEditWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    WhatsAppLifecycleResponse,
    Error,
    Parameters<typeof editWhatsappBusinessMessage>[0]
  >({
    mutationFn: editWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useDeleteWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    WhatsAppLifecycleResponse,
    Error,
    Parameters<typeof deleteWhatsappBusinessMessage>[0]
  >({
    mutationFn: deleteWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function usePinWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<MessagePinToggleResponse, Error, { message_id: string }>({
    mutationFn: pinWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useAddWhatsappReactionMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    TelegramReactionResponse,
    Error,
    { message_id: string; request: TelegramReactionRequest }
  >({
    mutationFn: ({ message_id, request }) => addWhatsappBusinessReaction(message_id, request),
    onSuccess: invalidate,
  })
}

export function useRemoveWhatsappReactionMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    TelegramReactionResponse,
    Error,
    { message_id: string; request: TelegramReactionRequest }
  >({
    mutationFn: ({ message_id, request }) => removeWhatsappBusinessReaction(message_id, request),
    onSuccess: invalidate,
  })
}

export function usePinWhatsappConversationMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    ConversationPinToggleResponse,
    Error,
    { conversation_id: string }
  >({
    mutationFn: pinWhatsappBusinessConversation,
    onSuccess: invalidate,
  })
}

export function useUnpinWhatsappConversationMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    ConversationPinToggleResponse,
    Error,
    { conversation_id: string }
  >({
    mutationFn: unpinWhatsappBusinessConversation,
    onSuccess: invalidate,
  })
}

export function useArchiveWhatsappConversationMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<ConversationPinToggleResponse, Error, { conversation_id: string }>({
    mutationFn: archiveWhatsappBusinessConversation,
    onSuccess: invalidate,
  })
}

export function useUnarchiveWhatsappConversationMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<ConversationPinToggleResponse, Error, { conversation_id: string }>({
    mutationFn: unarchiveWhatsappBusinessConversation,
    onSuccess: invalidate,
  })
}

export function useMuteWhatsappConversationMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<ConversationPinToggleResponse, Error, { conversation_id: string }>({
    mutationFn: muteWhatsappBusinessConversation,
    onSuccess: invalidate,
  })
}

export function useUnmuteWhatsappConversationMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<ConversationPinToggleResponse, Error, { conversation_id: string }>({
    mutationFn: unmuteWhatsappBusinessConversation,
    onSuccess: invalidate,
  })
}

export function useMarkWhatsappConversationReadMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<ConversationPinToggleResponse, Error, { conversation_id: string }>({
    mutationFn: markWhatsappBusinessConversationRead,
    onSuccess: invalidate,
  })
}

export function useMarkWhatsappConversationUnreadMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<ConversationPinToggleResponse, Error, { conversation_id: string }>({
    mutationFn: markWhatsappBusinessConversationUnread,
    onSuccess: invalidate,
  })
}
