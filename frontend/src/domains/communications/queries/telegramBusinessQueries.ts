import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  addTelegramBusinessReaction,
  deleteTelegramBusinessMessage,
  editTelegramBusinessMessage,
  fetchTelegramBusinessChatDetail,
  fetchTelegramBusinessChatMembers,
  fetchTelegramBusinessChats,
  fetchTelegramBusinessForwardChain,
  fetchTelegramBusinessMessageTombstones,
  fetchTelegramBusinessMessageVersions,
  fetchTelegramBusinessMessages,
  fetchTelegramBusinessPinnedMessages,
  fetchTelegramBusinessRawEvidence,
  fetchTelegramBusinessReactions,
  fetchTelegramBusinessReplyChain,
  fetchTelegramBusinessTopicMessages,
  fetchTelegramBusinessTopics,
  forwardTelegramBusinessMessage,
  markTelegramBusinessMessageRead,
  pinTelegramBusinessMessage,
  previewTelegramBusinessAttachment,
  removeTelegramBusinessReaction,
  replyToTelegramBusinessMessage,
  restoreTelegramBusinessMessageVisibility,
  searchTelegramBusinessChats,
  searchTelegramBusinessMedia,
  searchTelegramBusinessMessages,
  searchTelegramBusinessTopics,
  sendTelegramBusinessMessage,
} from '../api/telegramBusinessApi'
import type {
  TelegramChat,
  TelegramChatDetailResponse,
  TelegramChatListResponse,
  TelegramChatMember,
  TelegramChatMemberListResponse,
  TelegramChatSearchResponse,
  TelegramForwardChainResponse,
  TelegramLifecycleResponse,
  TelegramManualSendResponse,
  TelegramMediaSearchResponse,
  TelegramMessage,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
  TelegramMessageTombstoneListResponse,
  TelegramMessageVersionListResponse,
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
  TelegramReplyChainResponse,
  TelegramTopicListResponse,
} from '../../../shared/communications/types/telegram'
import type { TelegramRawMessageResponse } from '../../../shared/communications/types/telegramRawEvidence'
import type { AttachmentPreviewResponse } from '../types/attachments'

export const telegramBusinessQueryKeys = {
  chats: ['communications', 'telegram', 'chats'] as const,
  chatDetail: ['communications', 'telegram', 'chat-detail'] as const,
  chatMembers: ['communications', 'telegram', 'chat-members'] as const,
  messages: ['communications', 'telegram', 'messages'] as const,
  topics: ['communications', 'telegram', 'topics'] as const,
  topicMessages: ['communications', 'telegram', 'topic-messages'] as const,
  search: ['communications', 'telegram', 'search'] as const,
}

export function useTelegramChatsQuery(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramChat[]>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chats,
      toValue(accountId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response: TelegramChatListResponse = await fetchTelegramBusinessChats(
        toValue(accountId),
        toValue(limit)
      )
      return response.items
    },
  })
}

export function useTelegramChatDetailQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramChat | null>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chatDetail,
      toValue(telegramChatId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(telegramChatId)
      if (!value) return null
      const response: TelegramChatDetailResponse = await fetchTelegramBusinessChatDetail(value)
      return response.item
    },
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

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
      ...telegramBusinessQueryKeys.chatMembers,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
      normalizeTelegramBusinessQueryValue(toValue(query)),
      normalizeTelegramBusinessQueryValue(toValue(role)),
    ]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      const value = toValue(telegramChatId)
      if (!value) return { items: [], next_cursor: null }
      return fetchTelegramBusinessChatMembers(
        value,
        toValue(limit),
        normalizeTelegramBusinessQueryValue(toValue(query)) || undefined,
        normalizeTelegramBusinessQueryValue(toValue(role)) || undefined,
        pageParam ?? undefined
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useTelegramMessagesQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramMessage[]>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.messages,
      toValue(accountId) ?? 'all',
      toValue(providerChatId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchTelegramBusinessMessages(
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

export function useTelegramDialogSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramChatSearchResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'dialogs',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.limit) ?? 20,
    ]),
    queryFn: () =>
      searchTelegramBusinessChats({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        limit: toValue(params.limit) ?? 20,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useTelegramMessageSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMessageSearchResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'messages',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.limit) ?? 50,
    ]),
    queryFn: () =>
      searchTelegramBusinessMessages({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        limit: toValue(params.limit) ?? 50,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useTelegramMediaSearchQuery(params: {
  q?: MaybeRefOrGetter<string | null | undefined>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  kind?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMediaSearchResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'media',
      toValue(params.q)?.trim() ?? '',
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.kind) ?? 'all',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      searchTelegramBusinessMedia({
        q: toValue(params.q) ?? undefined,
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        kind: toValue(params.kind) ?? undefined,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.accountId) && toValue(params.providerChatId))),
  })
}

export function useTelegramPinnedMessagesQuery(params: {
  telegramChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMessageListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chats,
      toValue(params.telegramChatId) ?? 'none',
      'pinned-messages',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      fetchTelegramBusinessPinnedMessages({
        telegram_chat_id: toValue(params.telegramChatId) as string,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.telegramChatId))),
  })
}

export function useTelegramTopicsQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<TelegramTopicListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.topics,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const chatId = toValue(telegramChatId)
      if (!chatId) return { telegram_chat_id: '', items: [] }
      return fetchTelegramBusinessTopics(chatId, toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useTelegramTopicMessagesQuery(
  topicId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramMessageListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.topicMessages,
      toValue(topicId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const tid = toValue(topicId)
      if (!tid) return { items: [] }
      return fetchTelegramBusinessTopicMessages(tid, toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(topicId))),
  })
}

export function useTelegramTopicSearchQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  q: MaybeRefOrGetter<string>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramTopicListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'topics',
      toValue(telegramChatId) ?? 'none',
      toValue(q).trim(),
      toValue(limit),
    ]),
    queryFn: async () => {
      const chatId = toValue(telegramChatId)
      if (!chatId) return { telegram_chat_id: '', items: [] }
      return searchTelegramBusinessTopics(chatId, toValue(q), toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(telegramChatId)) && toValue(q).trim().length >= 2),
  })
}

export function useTelegramMessageVersionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageVersionListResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'versions']),
    queryFn: () => fetchTelegramBusinessMessageVersions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageTombstonesQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageTombstoneListResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'tombstones']),
    queryFn: () => fetchTelegramBusinessMessageTombstones(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageReactionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReactionListResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'reactions']),
    queryFn: () => fetchTelegramBusinessReactions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramReplyChainQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReplyChainResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'reply-chain']),
    queryFn: () => fetchTelegramBusinessReplyChain(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramForwardChainQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramForwardChainResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'forward-chain']),
    queryFn: () => fetchTelegramBusinessForwardChain(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramRawMessageEvidenceQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramRawMessageResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'raw-evidence']),
    queryFn: () => fetchTelegramBusinessRawEvidence(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramAttachmentPreviewQuery(
  attachmentId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<AttachmentPreviewResponse>({
    queryKey: computed(() => [
      'communications',
      'messages',
      toValue(attachmentId) ?? 'none',
      'attachment-preview',
    ]),
    queryFn: () => previewTelegramBusinessAttachment(toValue(attachmentId) as string),
    enabled: computed(() => Boolean(toValue(attachmentId)) && Boolean(toValue(enabled))),
  })
}

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
  return useMutation<TelegramManualSendResponse, Error, { account_id: string; provider_chat_id: string; text: string }>({
    mutationFn: (request: { account_id: string; provider_chat_id: string; text: string }) =>
      sendTelegramBusinessMessage(request),
    onSuccess: invalidate,
  })
}

export function useReplyTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<TelegramManualSendResponse, Error, {
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
  return useMutation<TelegramManualSendResponse, Error, {
      message_id: string
      account_id?: string
      provider_chat_id: string
      from_provider_chat_id?: string
      from_provider_message_id?: string
    }>({
    mutationFn: (request) =>
      forwardTelegramBusinessMessage({
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
  return useMutation<TelegramLifecycleResponse, Error, Parameters<typeof pinTelegramBusinessMessage>[0]>({
    mutationFn: pinTelegramBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useMarkReadTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation({
    mutationFn: markTelegramBusinessMessageRead,
    onSuccess: invalidate,
  })
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

function normalizeTelegramBusinessQueryValue(value: string | null | undefined): string {
  return value?.trim() ?? ''
}
