import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchCommunicationMessage,
  fetchCommunicationMessages,
  fetchMailSyncSettings,
  fetchMailboxHealth,
  fetchMailSyncStatus,
  fetchMessageStateCounts,
  fetchPersonas,
  fetchThreadMessages,
  fetchThreads,
  updateMailSyncSettings
} from '../api/communications'
import { fetchMessageAiState } from '../api/aiState'
import type {
  CommunicationMessageSummary,
  CommunicationPersona,
  LocalMessageState,
  MailboxHealth,
  CommunicationMessageDetailResponse,
  CommunicationMessagesResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatus,
  CommunicationThread,
  ThreadMessagesResponse,
  ThreadListResponse,
  WorkflowState,
  WorkflowStateCountItem
} from '../types/communications'
import type { CommunicationAiStateRecord } from '../types/aiState'
import { communicationListQueryKey, communicationMessageQueryKey, threadMessagesQueryKey } from './communicationPrefetch'
import {
  communicationDetailQueryOptions,
  communicationRealtimeQueryOptions,
  communicationReferenceQueryOptions
} from './communicationQueryPolicies'
import type { NullableQueryParam, QueryParam } from './queryTypes'

export function useMailListQuery(
  accountId?: QueryParam<string>,
  workflowState?: QueryParam<WorkflowState>,
  channelKind?: QueryParam<string>,
  query?: QueryParam<string>,
  localState?: QueryParam<LocalMessageState>
) {
  const pageSize = 100

  return useInfiniteQuery<CommunicationMessagesResponse, Error, CommunicationMessageSummary[], readonly unknown[], string | null>({
    queryKey: computed(() => communicationListQueryKey(
      toValue(accountId),
      toValue(workflowState),
      toValue(channelKind),
      toValue(query),
      toValue(localState)
    )),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      return fetchCommunicationMessages(
        toValue(accountId),
        toValue(workflowState),
        toValue(channelKind),
        toValue(query),
        toValue(localState),
        pageSize,
        pageParam
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => {
      return data.pages.flatMap((page) => page.items)
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useMessageQuery(messageId: NullableQueryParam<string>) {
  return useQuery<CommunicationMessageDetailResponse | null>({
    queryKey: computed(() => {
      const id = toValue(messageId)
      return id ? communicationMessageQueryKey(id) : ['communications-message', null] as const
    }),
    queryFn: async () => {
      const id = toValue(messageId)
      if (!id) return null
      return fetchCommunicationMessage(id)
    },
    enabled: computed(() => !!toValue(messageId)),
    ...communicationDetailQueryOptions
  })
}

export function useMessageAiStateQuery(messageId: NullableQueryParam<string>) {
  return useQuery<CommunicationAiStateRecord | null>({
    queryKey: computed(() => {
      const id = toValue(messageId)
      return id ? ['communications-ai-state', id] as const : ['communications-ai-state', null] as const
    }),
    queryFn: async () => {
      const id = toValue(messageId)
      if (!id) return null
      return fetchMessageAiState(id)
    },
    enabled: computed(() => !!toValue(messageId)),
    ...communicationRealtimeQueryOptions
  })
}

export function useStateCountsQuery(accountId?: QueryParam<string>, localState?: QueryParam<LocalMessageState>) {
  return useQuery<WorkflowStateCountItem[]>({
    queryKey: computed(() => ['communications-state-counts', toValue(accountId), toValue(localState)]),
    queryFn: async () => {
      const res = await fetchMessageStateCounts(toValue(accountId), toValue(localState))
      return res.counts
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useSyncStatusesQuery() {
  return useQuery<MailSyncStatus[]>({
    queryKey: ['communications', 'mail', 'sync-statuses'],
    queryFn: async () => {
      const res = await fetchMailSyncStatus()
      return res.items
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useMailSyncSettingsQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailSyncSettings | null>({
    queryKey: computed(() => {
      const id = toValue(accountId)
      return id
        ? ['communications', 'mail', 'sync-settings', id] as const
        : ['communications', 'mail', 'sync-settings', null] as const
    }),
    queryFn: async () => {
      const id = toValue(accountId)
      if (!id) return null
      return fetchMailSyncSettings(id)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
    ...communicationReferenceQueryOptions
  })
}

export function useUpdateMailSyncSettingsMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailSyncSettings,
    Error,
    { accountId: string; settings: MailSyncSettingsUpdate }
  >({
    mutationFn: async ({ accountId, settings }) => updateMailSyncSettings(accountId, settings),
    onSuccess: (_settings, variables) => {
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-settings', variables.accountId] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
    }
  })
}

export function useMailboxHealthQuery(accountId?: QueryParam<string>) {
  return useQuery<MailboxHealth | null>({
    queryKey: computed(() => ['communications', 'mail', 'mailbox-health', toValue(accountId)]),
    queryFn: async () => {
      return fetchMailboxHealth(toValue(accountId))
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useConversationsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<ThreadListResponse, Error, CommunicationThread[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-threads', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchThreads(toValue(accountId), 50, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useThreadMessagesQuery(accountId: NullableQueryParam<string>, subject: NullableQueryParam<string>) {
  return useQuery<ThreadMessagesResponse>({
    queryKey: computed(() => {
      const currentAccountId = toValue(accountId)?.trim() ?? ''
      const currentSubject = toValue(subject)?.trim() ?? ''
      return currentAccountId && currentSubject
        ? threadMessagesQueryKey(currentAccountId, currentSubject)
        : ['communications-thread-messages', currentAccountId, currentSubject] as const
    }),
    queryFn: async () => {
      const currentAccountId = toValue(accountId)?.trim() ?? ''
      const currentSubject = toValue(subject)?.trim() ?? ''
      if (!currentAccountId || !currentSubject) return { items: [] }
      return fetchThreadMessages(currentAccountId, currentSubject, 100)
    },
    enabled: computed(() => Boolean(toValue(accountId)?.trim() && toValue(subject)?.trim())),
    ...communicationDetailQueryOptions
  })
}

export function useCommunicationPersonasQuery() {
  return useQuery<CommunicationPersona[]>({
    queryKey: ['communications-personas'],
    queryFn: async () => {
      const res = await fetchPersonas()
      return res.items
    },
    ...communicationReferenceQueryOptions
  })
}
