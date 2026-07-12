import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  addTelegramBusinessReaction,
  closeTelegramBusinessTopic,
  createTelegramBusinessTopic,
  deleteTelegramBusinessMessage,
  editTelegramBusinessMessage,
  fetchTelegramBusinessChatDetail,
  fetchTelegramBusinessChatFolders,
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
  updateTelegramBusinessChatHistoryPolicy,
  updateTelegramBusinessChatReadReceiptPolicy,
  updateTelegramBusinessChatUnreadCounterPolicy,
} from '../api/telegramBusinessApi'
import type {
  TelegramChat,
  TelegramChatDetailResponse,
  TelegramChatGroupFilter,
  TelegramChatListResponse,
  TelegramChatMember,
  TelegramChatMemberListResponse,
  TelegramChatSearchResponse,
  TelegramForwardChainResponse,
  TelegramLifecycleResponse,
  TelegramMediaSearchResponse,
  TelegramMessage,
  TelegramMessageListResponse,
  TelegramMessagePageResponse,
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
import type {
  CommunicationProviderMessageCommandResponse,
  MessagePinToggleResponse,
} from '../types/communications'
import type { TelegramTopicCloseRequest, TelegramTopicCreateRequest, TelegramTopicLifecycleResponse } from '../../../shared/communications/types/telegramTopics'

export const telegramBusinessQueryKeys = {
  chats: ['communications', 'telegram', 'chats'] as const,
  chatDetail: ['communications', 'telegram', 'chat-detail'] as const,
  chatMembers: ['communications', 'telegram', 'chat-members'] as const,
  messages: ['communications', 'telegram', 'messages'] as const,
  topics: ['communications', 'telegram', 'topics'] as const,
  topicMessages: ['communications', 'telegram', 'topic-messages'] as const,
  search: ['communications', 'telegram', 'search'] as const,
}

// SSE is the primary realtime path. Polling keeps the list current while a
// local runtime reconnects or the browser briefly loses its event stream.
const TELEGRAM_CONVERSATION_REFETCH_INTERVAL_MS = 15_000
const TELEGRAM_MESSAGE_REFETCH_INTERVAL_MS = 8_000

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
    refetchInterval: TELEGRAM_CONVERSATION_REFETCH_INTERVAL_MS,
  })
}

export function useTelegramChatFoldersQuery(
  accountId?: MaybeRefOrGetter<string | undefined>
) {
  return useQuery<TelegramChatGroupFilter[]>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chats,
      'folders',
      toValue(accountId) ?? 'all',
    ]),
    queryFn: async () => {
      const response = await fetchTelegramBusinessChatFolders(toValue(accountId))
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
    refetchInterval: TELEGRAM_MESSAGE_REFETCH_INTERVAL_MS,
  })
}

export function useTelegramMessagesInfiniteQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useInfiniteQuery<TelegramMessagePageResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.messages,
      'infinite',
      toValue(accountId) ?? 'all',
      toValue(providerChatId) ?? 'all',
      toValue(limit),
    ]),
    initialPageParam: null as string | null,
    queryFn: ({ pageParam }) => fetchTelegramBusinessMessages(
      toValue(accountId) ?? undefined,
      toValue(providerChatId) ?? undefined,
      toValue(limit),
      typeof pageParam === 'string' ? pageParam : undefined
    ),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    enabled: computed(() => Boolean(toValue(accountId) && toValue(providerChatId))),
    refetchInterval: TELEGRAM_MESSAGE_REFETCH_INTERVAL_MS,
  })
}

export function useUpdateTelegramChatHistoryPolicyMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, enabled }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      enabled: boolean
    }) => updateTelegramBusinessChatHistoryPolicy(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      full_history_sync_enabled: enabled,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chatDetail })
    },
  })
}

export function useUpdateTelegramChatReadReceiptPolicyMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, enabled }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      enabled: boolean
    }) => updateTelegramBusinessChatReadReceiptPolicy(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      read_receipt_reports_enabled: enabled,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chatDetail })
    },
  })
}

export function useUpdateTelegramChatUnreadCounterPolicyMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, hideUnreadCounter }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      hideUnreadCounter: boolean
    }) => updateTelegramBusinessChatUnreadCounterPolicy(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      hide_unread_counter: hideUnreadCounter,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chatDetail })
    },
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

function useInvalidateTelegramTopics() {
  const queryClient = useQueryClient()
  return () => {
    queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.topics })
    queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.topicMessages })
    queryClient.invalidateQueries({ queryKey: telegramBusinessQueryKeys.chats })
  }
}

export function useCreateTelegramTopicMutation() {
  const invalidate = useInvalidateTelegramTopics()
  return useMutation<TelegramTopicLifecycleResponse, Error, { conversationId: string; request: TelegramTopicCreateRequest }>({
    mutationFn: ({ conversationId, request }) => createTelegramBusinessTopic(conversationId, request),
    onSuccess: invalidate,
  })
}

export function useCloseTelegramTopicMutation() {
  const invalidate = useInvalidateTelegramTopics()
  return useMutation<TelegramTopicLifecycleResponse, Error, { topicId: string; request: TelegramTopicCloseRequest }>({
    mutationFn: ({ topicId, request }) => closeTelegramBusinessTopic(topicId, request),
    onSuccess: invalidate,
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
  return useMutation<CommunicationProviderMessageCommandResponse, Error, { account_id: string; provider_chat_id: string; text: string }>({
    mutationFn: (request: { account_id: string; provider_chat_id: string; text: string }) =>
      sendTelegramBusinessMessage(request),
    onSuccess: invalidate,
  })
}

export function useReplyTelegramMessageMutation() {
  const invalidate = useInvalidateTelegramBusinessState()
  return useMutation<CommunicationProviderMessageCommandResponse, Error, {
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
  return useMutation<CommunicationProviderMessageCommandResponse, Error, {
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
  return useMutation<MessagePinToggleResponse, Error, { message_id: string }>({
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
