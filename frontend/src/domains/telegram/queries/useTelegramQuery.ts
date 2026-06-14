import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  fetchTelegramCapabilities,
  fetchTelegramChats,
  fetchTelegramMessages,
  fetchTelegramRuntimeStatus,
  fetchTelegramAccounts,
  fetchTelegramCalls,
  syncTelegramChats,
  syncTelegramHistory,
  sendTelegramMessage,
  ingestTelegramFixtureMessage,
  startTelegramRuntime
} from '../api/telegram'
import type {
  TelegramCapabilitiesResponse,
  TelegramRuntimeStatus,
  TelegramAccount,
  TelegramChat,
  TelegramMessage,
  TelegramChatSyncRequest,
  TelegramHistorySyncRequest
} from '../types/telegram'

// --- Fetch capabilities ---
export function useTelegramCapabilitiesQuery() {
  return useQuery<TelegramCapabilitiesResponse>({
    queryKey: ['telegram', 'capabilities'],
    queryFn: () => fetchTelegramCapabilities()
  })
}

// --- Fetch accounts ---
export function useTelegramAccountsQuery() {
  return useQuery<TelegramAccount[]>({
    queryKey: ['telegram', 'accounts'],
    queryFn: async () => {
      const res = await fetchTelegramAccounts()
      return res.items
    }
  })
}

// --- Fetch chats ---
export function useTelegramChatsQuery(accountId?: string, limit = 50) {
  return useQuery<TelegramChat[]>({
    queryKey: ['telegram', 'chats', accountId ?? 'all', limit],
    queryFn: async () => {
      const res = await fetchTelegramChats(accountId, limit)
      return res.items
    }
  })
}

// --- Fetch messages ---
export function useTelegramMessagesQuery(accountId?: string, providerChatId?: string, limit = 50) {
  return useQuery<TelegramMessage[]>({
    queryKey: ['telegram', 'messages', accountId ?? 'all', providerChatId ?? 'all', limit],
    queryFn: async () => {
      const res = await fetchTelegramMessages(accountId, providerChatId, limit)
      return res.items
    }
  })
}

// --- Fetch runtime status for a specific account ---
export function useTelegramRuntimeStatusQuery(accountId: string | null) {
  return useQuery<TelegramRuntimeStatus | null>({
    queryKey: ['telegram', 'runtime', accountId],
    queryFn: async () => {
      if (!accountId) return null
      return fetchTelegramRuntimeStatus(accountId)
    },
    enabled: !!accountId
  })
}

// --- Fetch calls ---
export function useTelegramCallsQuery(accountId?: string, limit = 50) {
  return useQuery<{ call_id: string; account_id: string; provider_chat_id: string; status: string; occurred_at: string | null }[]>({
    queryKey: ['telegram', 'calls', accountId ?? 'all', limit],
    queryFn: async () => {
      const res = await fetchTelegramCalls(accountId, limit)
      return res.items
    }
  })
}

// --- Sync chats mutation ---
export function useSyncTelegramChatsMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramChatSyncRequest) => syncTelegramChats(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['telegram', 'chats'] })
    }
  })
}

// --- Sync history mutation ---
export function useSyncTelegramHistoryMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramHistorySyncRequest) => syncTelegramHistory(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['telegram', 'messages'] })
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
      queryClient.invalidateQueries({ queryKey: ['telegram', 'messages'] })
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
      queryClient.invalidateQueries({ queryKey: ['telegram', 'messages'] })
    }
  })
}

// --- Start runtime mutation ---
export function useStartTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => startTelegramRuntime(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['telegram', 'runtime'] })
    }
  })
}
