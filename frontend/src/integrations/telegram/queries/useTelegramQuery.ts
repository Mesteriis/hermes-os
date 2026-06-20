import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramAccountCapabilities,
  fetchTelegramCapabilities,
  fetchTelegramChatDetail,
  fetchTelegramChats,
  fetchTelegramFolders,
  fetchTelegramMessages,
  fetchTelegramAccounts,
  fetchTelegramCalls,
  fetchTelegramCallTranscript,
  fetchTelegramTopics,
  fetchTelegramTopicMessages,
  fetchTelegramTopicSearch,
} from '../api/telegram'
import type {
  TelegramCapabilitiesResponse,
  TelegramCall,
  TelegramCallTranscript,
  TelegramAccount,
  TelegramChat,
  TelegramChatGroupFilter,
  TelegramMessage,
  TelegramTopicListResponse,
  TelegramMessageListResponse
} from '../types/telegram'
import { telegramQueryKeys } from './telegramQueryKeys'

export { telegramQueryKeys } from './telegramQueryKeys'
export * from './useTelegramMutations'

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
