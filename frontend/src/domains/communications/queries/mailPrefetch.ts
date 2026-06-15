import { useQueryClient, type QueryClient } from '@tanstack/vue-query'
import { fetchMailMessage, fetchMailMessages, fetchThreadMessages } from '../api/communications'
import type { LocalMessageState, MailMessageDetailResponse, MailMessagesResponse, ThreadMessagesResponse, WorkflowState } from '../types/communications'
import type { AttachmentSearchResult } from '../types/attachments'
import type { MailSavedSearch } from '../types/savedSearches'

const MESSAGE_PREFETCH_STALE_MS = 30_000
const THREAD_MESSAGES_PREFETCH_STALE_MS = 30_000

export function mailMessageQueryKey(messageId: string) {
  return ['communications-message', messageId] as const
}

export function mailListQueryKey(
  accountId?: string,
  workflowState?: WorkflowState | '',
  channelKind?: string,
  query?: string,
  localState?: LocalMessageState
) {
  return [
    'communications-mail-list',
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

export async function prefetchMailMessage(
  queryClient: QueryClient,
  messageId: string
): Promise<void> {
  const normalizedMessageId = messageId.trim()
  if (!normalizedMessageId) return

  await queryClient.prefetchQuery<MailMessageDetailResponse>({
    queryKey: mailMessageQueryKey(normalizedMessageId),
    queryFn: () => fetchMailMessage(normalizedMessageId),
    staleTime: MESSAGE_PREFETCH_STALE_MS
  })
}

export function useMailMessagePrefetch() {
  const queryClient = useQueryClient()
  return (messageId: string) => prefetchMailMessage(queryClient, messageId)
}

export async function prefetchMailMessageForAttachmentResult(
  queryClient: QueryClient,
  result: AttachmentSearchResult
): Promise<void> {
  await prefetchMailMessage(queryClient, result.message_id)
}

export function useAttachmentSearchResultPrefetch() {
  const queryClient = useQueryClient()
  return (result: AttachmentSearchResult) => prefetchMailMessageForAttachmentResult(queryClient, result)
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

export async function prefetchMailListForSavedSearch(
  queryClient: QueryClient,
  savedSearch: MailSavedSearch,
  fallbackAccountId?: string | null
): Promise<void> {
  const accountId = savedSearch.account_id?.trim() || fallbackAccountId?.trim() || undefined
  const workflowState = savedSearch.workflow_state ?? undefined
  const channelKind = savedSearch.channel_kind?.trim() || undefined
  const query = savedSearch.query.trim() || undefined
  const localState = savedSearch.local_state

  await queryClient.prefetchQuery<MailMessagesResponse>({
    queryKey: mailListQueryKey(accountId, workflowState, channelKind, query, localState),
    queryFn: () => fetchMailMessages(accountId, workflowState, channelKind, query, localState, 250, null),
    staleTime: MESSAGE_PREFETCH_STALE_MS
  })
}

export function useSavedSearchMailListPrefetch(accountId: () => string | null | undefined) {
  const queryClient = useQueryClient()
  return (savedSearch: MailSavedSearch) => prefetchMailListForSavedSearch(queryClient, savedSearch, accountId())
}
