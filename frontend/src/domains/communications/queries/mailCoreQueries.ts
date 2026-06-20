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
import type { MailAiStateRecord } from '../types/aiState'
import { mailListQueryKey, mailMessageQueryKey, threadMessagesQueryKey } from './mailPrefetch'
import {
  mailDetailQueryOptions,
  mailRealtimeQueryOptions,
  mailReferenceQueryOptions
} from './mailQueryPolicies'
import type { NullableQueryParam, QueryParam } from './queryTypes'

export function useMailListQuery(
  accountId?: QueryParam<string>,
  workflowState?: QueryParam<WorkflowState>,
  channelKind?: QueryParam<string>,
  query?: QueryParam<string>,
  localState?: QueryParam<LocalMessageState>
) {
  return useInfiniteQuery<CommunicationMessagesResponse, Error, CommunicationMessageSummary[], readonly unknown[], string | null>({
    queryKey: computed(() => mailListQueryKey(
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
        250,
        pageParam
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => {
      return data.pages.flatMap((page) => page.items)
    },
    ...mailRealtimeQueryOptions
  })
}

export function useMessageQuery(messageId: NullableQueryParam<string>) {
  return useQuery<CommunicationMessageDetailResponse | null>({
    queryKey: computed(() => {
      const id = toValue(messageId)
      return id ? mailMessageQueryKey(id) : ['communications-message', null] as const
    }),
    queryFn: async () => {
      const id = toValue(messageId)
      if (!id) return null
      return fetchCommunicationMessage(id)
    },
    enabled: computed(() => !!toValue(messageId)),
    ...mailDetailQueryOptions
  })
}

export function useMessageAiStateQuery(messageId: NullableQueryParam<string>) {
  return useQuery<MailAiStateRecord | null>({
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
    ...mailRealtimeQueryOptions
  })
}

export function useStateCountsQuery(accountId?: QueryParam<string>, localState?: QueryParam<LocalMessageState>) {
  return useQuery<WorkflowStateCountItem[]>({
    queryKey: computed(() => ['communications-state-counts', toValue(accountId), toValue(localState)]),
    queryFn: async () => {
      const res = await fetchMessageStateCounts(toValue(accountId), toValue(localState))
      return res.counts
    },
    ...mailRealtimeQueryOptions
  })
}

export function useSyncStatusesQuery() {
  return useQuery<MailSyncStatus[]>({
    queryKey: ['communications', 'mail', 'sync-statuses'],
    queryFn: async () => {
      const res = await fetchMailSyncStatus()
      return res.items
    },
    ...mailRealtimeQueryOptions
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
    ...mailReferenceQueryOptions
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
    ...mailRealtimeQueryOptions
  })
}

export function useConversationsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<ThreadListResponse, Error, CommunicationThread[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-threads', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchThreads(toValue(accountId), 50, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
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
    ...mailDetailQueryOptions
  })
}

export function usePersonasQuery() {
  return useQuery<CommunicationPersona[]>({
    queryKey: ['communications-personas'],
    queryFn: async () => {
      const res = await fetchPersonas()
      return res.items
    },
    ...mailReferenceQueryOptions
  })
}
