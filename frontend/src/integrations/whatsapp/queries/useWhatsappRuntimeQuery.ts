import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  deadLetterWhatsappProviderCommand,
  fetchWhatsappProviderCommands,
  fetchWhatsappSyncChats,
  fetchWhatsappSyncCalls,
  fetchWhatsappSyncContacts,
  fetchWhatsappSyncHistory,
  fetchWhatsappSyncMedia,
  fetchWhatsappSyncMembers,
  fetchWhatsappSyncPresence,
  fetchWhatsappSyncStatuses,
  fetchWhatsappRuntimeHealth,
  fetchWhatsappRuntimeStatus,
  publishWhatsappStatus,
  relinkWhatsappRuntime,
  retryWhatsappProviderCommand,
  rotateWhatsappRuntime,
  removeWhatsappRuntime,
  revokeWhatsappRuntime,
  startWhatsappRuntime,
  stopWhatsappRuntime,
} from '../api/whatsapp'
import { startHiddenWhatsappWebview } from '../api/whatsappCompanion'
import type {
  WhatsAppCallSyncItem,
  WhatsAppChatSyncItem,
  WhatsAppContactSyncItem,
  WhatsAppMediaSyncItem,
  WhatsAppMembersSyncItem,
  WhatsAppProviderCommand,
  WhatsAppPresenceSyncItem,
  WhatsappWebMessage,
  WhatsAppProviderCommandListResponse,
  WhatsAppWebCompanionManifest,
  WhatsAppRuntimeHealth,
  WhatsAppRuntimeRemoveResponse,
  WhatsAppRuntimeStatus,
} from '../types/whatsapp'
import { whatsappQueryKeys } from './whatsappQueryKeys'

export function useWhatsappRuntimeStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<WhatsAppRuntimeStatus | null>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.runtimeStatus,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchWhatsappRuntimeStatus(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappRuntimeHealthQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<WhatsAppRuntimeHealth | null>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.runtimeHealth,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchWhatsappRuntimeHealth(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

function invalidateWhatsappRuntime(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.accounts })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.sessions })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.capabilities })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.accountCapabilities })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.runtimeStatus })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.runtimeHealth })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.commands })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncChats })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncHistory })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncMembers })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncStatuses })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncPresence })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncCalls })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncContacts })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncMedia })
}

export function useStartWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => startWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useStopWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => stopWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRevokeWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => revokeWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRelinkWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => relinkWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRotateWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => rotateWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRemoveWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsAppRuntimeRemoveResponse, Error, { account_id: string }>({
    mutationFn: (request) => removeWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useWhatsappProviderCommandsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 25
) {
  return useQuery<WhatsAppProviderCommand[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.commands,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response: WhatsAppProviderCommandListResponse =
        await fetchWhatsappProviderCommands({
          account_id: value,
          limit,
        })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncChatsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsAppChatSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncChats,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncChats({
        account_id: value,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncHistoryQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncHistory,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const account = toValue(accountId)
      const providerChatIdValue = toValue(providerChatId)
      if (!account || !providerChatIdValue) return []
      const response = await fetchWhatsappSyncHistory({
        account_id: account,
        provider_chat_id: providerChatIdValue,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId) && toValue(providerChatId))),
  })
}

export function useWhatsappSyncMembersQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsAppMembersSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncMembers,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const account = toValue(accountId)
      const providerChatIdValue = toValue(providerChatId)
      if (!account || !providerChatIdValue) return []
      const response = await fetchWhatsappSyncMembers({
        account_id: account,
        provider_chat_id: providerChatIdValue,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId) && toValue(providerChatId))),
  })
}

export function useWhatsappSyncPresenceQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined> = null,
  limit = 12
) {
  return useQuery<WhatsAppPresenceSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncPresence,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'all',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncPresence({
        account_id: value,
        provider_chat_id: toValue(providerChatId) ?? undefined,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncStatusesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncStatuses,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncStatuses({
        account_id: value,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncCallsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined> = null,
  limit = 12
) {
  return useQuery<WhatsAppCallSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncCalls,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'all',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncCalls({
        account_id: value,
        provider_chat_id: toValue(providerChatId) ?? undefined,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncContactsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsAppContactSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncContacts,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncContacts({
        account_id: value,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncMediaQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined> = null,
  limit = 12
) {
  return useQuery<WhatsAppMediaSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncMedia,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'all',
      'image/',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncMedia({
        account_id: value,
        provider_chat_id: toValue(providerChatId) ?? undefined,
        content_type: 'image/',
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useRetryWhatsappProviderCommandMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsAppProviderCommand, Error, { command_id: string }>({
    mutationFn: ({ command_id }) => retryWhatsappProviderCommand(command_id),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useDeadLetterWhatsappProviderCommandMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsAppProviderCommand, Error, { command_id: string; reason: string }>({
    mutationFn: ({ command_id, reason }) =>
      deadLetterWhatsappProviderCommand({ command_id, reason }),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function usePublishWhatsappStatusMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    WhatsAppProviderCommand,
    Error,
    { account_id: string; idempotency_key: string; text: string; command_id?: string }
  >({
    mutationFn: (request) => publishWhatsappStatus(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useStartHiddenWhatsappWebviewMutation() {
  const queryClient = useQueryClient()

  return useMutation<WhatsAppWebCompanionManifest, Error, { account_id: string }>({
    mutationFn: ({ account_id }) => startHiddenWhatsappWebview(account_id),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}
