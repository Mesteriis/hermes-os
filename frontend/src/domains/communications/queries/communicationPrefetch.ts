import { useQueryClient, type QueryClient } from '@tanstack/vue-query'
import { fetchCommunicationMessage, fetchCommunicationMessages, fetchThreadMessages } from '../api/communications'
import type { LocalMessageState, CommunicationMessageDetailResponse, CommunicationMessagesResponse, ThreadMessagesResponse, WorkflowState } from '../types/communications'
import type { AttachmentSearchResult } from '../types/attachments'
import type { CommunicationSavedSearch } from '../types/savedSearches'

const MESSAGE_PREFETCH_STALE_MS = 30_000
const THREAD_MESSAGES_PREFETCH_STALE_MS = 30_000

export function communicationMessageQueryKey(messageId: string) {
  return ['communications-message', messageId] as const
}

export function communicationListQueryKey(
  accountId?: string,
  workflowState?: WorkflowState | '',
  channelKind?: string,
  query?: string,
  localState?: LocalMessageState
) {
  return [
    'communications-list',
    accountId,
    workflowState,
    channelKind,
    query,
    localState
  ] as const
}

export function threadMessagesQueryKey(accountId: string, subject: string) {
  return ['communications-thread-messages', accountId, subject] as const
}

export async function prefetchCommunicationMessage(
  queryClient: QueryClient,
  messageId: string
): Promise<void> {
  const normalizedMessageId = messageId.trim()
  if (!normalizedMessageId) return

  await queryClient.prefetchQuery<CommunicationMessageDetailResponse>({
    queryKey: communicationMessageQueryKey(normalizedMessageId),
    queryFn: () => fetchCommunicationMessage(normalizedMessageId),
    staleTime: MESSAGE_PREFETCH_STALE_MS
  })
}

export function useCommunicationMessagePrefetch() {
  const queryClient = useQueryClient()
  return (messageId: string) => prefetchCommunicationMessage(queryClient, messageId)
}

export async function prefetchCommunicationMessageForAttachmentResult(
  queryClient: QueryClient,
  result: AttachmentSearchResult
): Promise<void> {
  await prefetchCommunicationMessage(queryClient, result.message_id)
}

export function useAttachmentSearchResultPrefetch() {
  const queryClient = useQueryClient()
  return (result: AttachmentSearchResult) => prefetchCommunicationMessageForAttachmentResult(queryClient, result)
}

export async function prefetchThreadMessages(
  queryClient: QueryClient,
  accountId: string,
  subject: string
): Promise<void> {
  const normalizedAccountId = accountId.trim()
  const normalizedSubject = subject.trim()
  if (!normalizedAccountId || !normalizedSubject) return

  await queryClient.prefetchQuery<ThreadMessagesResponse>({
    queryKey: threadMessagesQueryKey(normalizedAccountId, normalizedSubject),
    queryFn: () => fetchThreadMessages(normalizedAccountId, normalizedSubject, 100),
    staleTime: THREAD_MESSAGES_PREFETCH_STALE_MS
  })
}

export function useThreadMessagesPrefetch() {
  const queryClient = useQueryClient()
  return (accountId: string, subject: string) => prefetchThreadMessages(queryClient, accountId, subject)
}

export async function prefetchCommunicationListForSavedSearch(
  queryClient: QueryClient,
  savedSearch: CommunicationSavedSearch,
  fallbackAccountId?: string | null
): Promise<void> {
  const accountId = savedSearch.account_id?.trim() || fallbackAccountId?.trim() || undefined
  const workflowState = savedSearch.workflow_state ?? undefined
  const channelKind = savedSearch.channel_kind?.trim() || undefined
  const query = savedSearch.query.trim() || undefined
  const localState = savedSearch.local_state

  await queryClient.prefetchQuery<CommunicationMessagesResponse>({
    queryKey: communicationListQueryKey(accountId, workflowState, channelKind, query, localState),
    queryFn: () => fetchCommunicationMessages(accountId, workflowState, channelKind, query, localState, 100, null),
    staleTime: MESSAGE_PREFETCH_STALE_MS
  })
}

export function useSavedSearchCommunicationListPrefetch(accountId: () => string | null | undefined) {
  const queryClient = useQueryClient()
  return (savedSearch: CommunicationSavedSearch) => prefetchCommunicationListForSavedSearch(queryClient, savedSearch, accountId())
}
