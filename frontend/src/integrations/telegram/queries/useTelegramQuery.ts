import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramAccountCapabilities,
  fetchTelegramCapabilities,
  fetchTelegramFolders,
  fetchTelegramAccounts,
  fetchTelegramCalls,
  fetchTelegramCallTranscript,
} from '../api/telegram'
import type {
  TelegramCapabilitiesResponse,
  TelegramCall,
  TelegramCallTranscript,
  TelegramAccount,
  TelegramChatGroupFilter,
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

function computedTelegramFoldersQueryKey(
  accountId?: MaybeRefOrGetter<string | undefined>
) {
  return computed(() => [
    ...telegramQueryKeys.folders,
    toValue(accountId) ?? 'all',
  ])
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
