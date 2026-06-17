import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramAccountCapabilities,
  fetchTelegramCapabilities,
  fetchTelegramChatDetail,
  fetchTelegramChatMembers,
  fetchTelegramChats,
  fetchTelegramFolders,
  fetchTelegramMessages,
  fetchTelegramAccounts,
  fetchTelegramCalls,
  fetchTelegramCallTranscript,
  logoutTelegramAccount,
  removeTelegramAccount,
  setupTelegramAccount,
  syncTelegramChatMembers,
  syncTelegramChats,
  syncTelegramHistory,
  sendTelegramMessage,
  ingestTelegramFixtureMessage,
  downloadTelegramMedia,
  pinTelegramChat,
  unpinTelegramChat,
  archiveTelegramChat,
  unarchiveTelegramChat,
  muteTelegramChat,
  unmuteTelegramChat,
  markTelegramChatRead,
  markTelegramChatUnread,
  fetchTelegramTopics,
  fetchTelegramTopicMessages,
  fetchTelegramTopicSearch,
  forwardTelegramMessage,
  replyToTelegramMessage
} from '../api/telegram'
import {
  addTelegramReaction,
  deleteTelegramMessage,
  editTelegramMessage,
  pinTelegramMessage,
  removeTelegramReaction,
  restoreTelegramMessageVisibility
} from '../api/telegramLifecycle'
import type {
  TelegramCapabilitiesResponse,
  TelegramCall,
  TelegramCallTranscript,
  TelegramAccount,
  TelegramChat,
  TelegramChatGroupFilter,
  TelegramChatMember,
  TelegramMessage,
  TelegramChatSyncRequest,
  TelegramHistorySyncRequest,
  TelegramMediaDownloadRequest,
  TelegramReactionRequest,
  TelegramTopicListResponse,
  TelegramMessageListResponse
} from '../types/telegram'

export const telegramQueryKeys = {
  capabilities: ['telegram', 'capabilities'] as const,
  accountCapabilities: ['telegram', 'account-capabilities'] as const,
  accounts: ['telegram', 'accounts'] as const,
  chats: ['telegram', 'chats'] as const,
  folders: ['telegram', 'folders'] as const,
  chatDetail: ['telegram', 'chat-detail'] as const,
  chatMembers: ['telegram', 'chat-members'] as const,
  messages: ['telegram', 'messages'] as const,
  runtime: ['telegram', 'runtime'] as const,
  calls: ['telegram', 'calls'] as const,
  callTranscript: ['telegram', 'call-transcript'] as const,
  topics: ['telegram', 'topics'] as const,
  topicMessages: ['telegram', 'topic-messages'] as const,
  topicSearch: ['telegram', 'topic-search'] as const,
}

// --- Fetch capabilities ---
export function useTelegramCapabilitiesQuery() {
  return useQuery<TelegramCapabilitiesResponse>({
    queryKey: telegramQueryKeys.capabilities,
    queryFn: () => fetchTelegramCapabilities()
  })
}

export function useTelegramAccountCapabilitiesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramCapabilitiesResponse | null>({
    queryKey: computed(() => [
      ...telegramQueryKeys.accountCapabilities,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchTelegramAccountCapabilities(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

// --- Fetch accounts ---
export function useTelegramAccountsQuery() {
  return useQuery<TelegramAccount[]>({
    queryKey: telegramQueryKeys.accounts,
    queryFn: async () => {
      const res = await fetchTelegramAccounts()
      return res.items
    }
  })
}

export function useSetupTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: setupTelegramAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    }
  })
}

export function useLogoutTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (accountId: string) => logoutTelegramAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
    }
  })
}

export function useRemoveTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (accountId: string) => removeTelegramAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
    }
  })
}

// --- Fetch chats ---
export function useTelegramChatsQuery(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramChat[]>({
    queryKey: computedTelegramChatsQueryKey(accountId, limit),
    queryFn: async () => {
      const res = await fetchTelegramChats(toValue(accountId), toValue(limit))
      return res.items
    }
  })
}

export function useTelegramFoldersQuery(
  accountId?: MaybeRefOrGetter<string | undefined>
) {
  return useQuery<TelegramChatGroupFilter[]>({
    queryKey: computedTelegramFoldersQueryKey(accountId),
    queryFn: async () => {
      const res = await fetchTelegramFolders(toValue(accountId))
      return res.items
    }
  })
}

export function useTelegramChatDetailQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramChat | null>({
    queryKey: computed(() => [
      ...telegramQueryKeys.chatDetail,
      toValue(telegramChatId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(telegramChatId)
      if (!value) return null
      const res = await fetchTelegramChatDetail(value)
      return res.item
    },
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useTelegramChatMembersQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramChatMember[]>({
    queryKey: computed(() => [
      ...telegramQueryKeys.chatMembers,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(telegramChatId)
      if (!value) return []
      const res = await fetchTelegramChatMembers(value, toValue(limit))
      return res.items
    },
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

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

// --- Fetch messages ---
export function useTelegramMessagesQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramMessage[]>({
    queryKey: computedTelegramMessagesQueryKey(accountId, providerChatId, limit),
    queryFn: async () => {
      const res = await fetchTelegramMessages(
        toValue(accountId) ?? undefined,
        toValue(providerChatId) ?? undefined,
        toValue(limit)
      )
      return res.items
    },
    enabled: computedTelegramMessagesEnabled(accountId, providerChatId)
  })
}

// --- Fetch calls ---
export function useTelegramCallsQuery(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramCall[]>({
    queryKey: computedTelegramCallsQueryKey(accountId, limit),
    queryFn: async () => {
      const res = await fetchTelegramCalls(toValue(accountId), toValue(limit))
      return res.items
    }
  })
}

export function useTelegramCallTranscriptQuery(
  callId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramCallTranscript | null>({
    queryKey: computed(() => [
      ...telegramQueryKeys.callTranscript,
      toValue(callId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(callId)
      if (!value) return null
      const res = await fetchTelegramCallTranscript(value)
      return res.transcript
    },
    enabled: computed(() => Boolean(toValue(callId))),
  })
}

// --- Sync chats mutation ---
export function useSyncTelegramChatsMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramChatSyncRequest) => syncTelegramChats(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    }
  })
}

// --- Sync history mutation ---
export function useSyncTelegramHistoryMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramHistorySyncRequest) => syncTelegramHistory(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

// --- Send message mutation ---
export function useSendTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string; provider_chat_id: string; text: string }) =>
      sendTelegramMessage(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    }
  })
}

// --- Ingest fixture message mutation ---
export function useIngestTelegramFixtureMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: Parameters<typeof ingestTelegramFixtureMessage>[0]) =>
      ingestTelegramFixtureMessage(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useEditTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: editTelegramMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useReplyTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: replyToTelegramMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useForwardTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: forwardTelegramMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    }
  })
}

export function useDeleteTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: deleteTelegramMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useRestoreTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: restoreTelegramMessageVisibility,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function usePinTelegramMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: pinTelegramMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useAddTelegramReactionMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ messageId, request }: { messageId: string; request: TelegramReactionRequest }) =>
      addTelegramReaction(messageId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
    }
  })
}

export function useRemoveTelegramReactionMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ messageId, request }: { messageId: string; request: TelegramReactionRequest }) =>
      removeTelegramReaction(messageId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
    }
  })
}

export function useDownloadTelegramMediaMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramMediaDownloadRequest) => downloadTelegramMedia(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })
    }
  })
}

export function usePinTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => pinTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useUnpinTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => unpinTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useArchiveTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => archiveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useUnarchiveTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => unarchiveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useMuteTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => muteTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useUnmuteTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => unmuteTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    }
  })
}

export function useMarkReadTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => markTelegramChatRead(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    }
  })
}

export function useMarkUnreadTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => markTelegramChatUnread(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    }
  })
}

function computedTelegramChatsQueryKey(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return computed(() => [
    ...telegramQueryKeys.chats,
    toValue(accountId) ?? 'all',
    toValue(limit)
  ])
}

function computedTelegramFoldersQueryKey(
  accountId?: MaybeRefOrGetter<string | undefined>
) {
  return computed(() => [
    ...telegramQueryKeys.folders,
    toValue(accountId) ?? 'all',
  ])
}

function computedTelegramMessagesQueryKey(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return computed(() => [
    ...telegramQueryKeys.messages,
    toValue(accountId) ?? 'all',
    toValue(providerChatId) ?? 'all',
    toValue(limit)
  ])
}

function computedTelegramMessagesEnabled(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
) {
  return computed(() => {
    const providerChatIdValue = toValue(providerChatId)
    if (providerChatIdValue === null) {
      return false
    }
    return providerChatIdValue === undefined || Boolean(toValue(accountId) && providerChatIdValue)
  })
}

function computedTelegramCallsQueryKey(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return computed(() => [
    ...telegramQueryKeys.calls,
    toValue(accountId) ?? 'all',
    toValue(limit)
  ])
}

// --- Forum topics ---

export function useTelegramTopicsQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<TelegramTopicListResponse>({
    queryKey: computed(() => [
      ...telegramQueryKeys.topics,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const chatId = toValue(telegramChatId)
      if (!chatId) return { telegram_chat_id: '', items: [] }
      return fetchTelegramTopics(chatId, toValue(limit))
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
      ...telegramQueryKeys.topicMessages,
      toValue(topicId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const tid = toValue(topicId)
      if (!tid) return { items: [] }
      return fetchTelegramTopicMessages(tid, toValue(limit))
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
      ...telegramQueryKeys.topicSearch,
      toValue(telegramChatId) ?? 'none',
      toValue(q),
      toValue(limit),
    ]),
    queryFn: async () => {
      const chatId = toValue(telegramChatId)
      const query = toValue(q).trim()
      if (!chatId || !query) return { telegram_chat_id: chatId ?? '', items: [] }
      return fetchTelegramTopicSearch(chatId, query, toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(telegramChatId)) && Boolean(toValue(q).trim())),
  })
}
