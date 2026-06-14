import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'
import {
  fetchMailMessages,
  fetchMailMessage,
  fetchMessageStateCounts,
  fetchMailSyncStatus,
  fetchDrafts,
  fetchMailboxHealth,
  fetchThreads,
  sendEmail,
  createDraft,
  deleteDraft
} from '../api/communications'
import type {
  CommunicationMessageSummary,
  MailMessageDetailResponse,
  MailboxHealth,
  MailSyncStatus,
  EmailDraft,
  WorkflowState,
  LocalMessageState,
  WorkflowStateCountItem,
  SendEmailRequest,
  SendEmailResponse
} from '../types/communications'

// --- Query: Mail list ---

export function useMailListQuery(
  accountId?: string,
  workflowState?: WorkflowState,
  channelKind?: string,
  query?: string,
  localState?: LocalMessageState
) {
  return useQuery<CommunicationMessageSummary[]>({
    queryKey: ['communications-mail-list', accountId, workflowState, channelKind, query, localState],
    queryFn: async () => {
      const res = await fetchMailMessages(accountId, workflowState, channelKind, query, localState)
      return res.items
    }
  })
}

// --- Query: Single message detail ---

export function useMessageQuery(messageId: string | null) {
  return useQuery<MailMessageDetailResponse | null>({
    queryKey: ['communications-message', messageId],
    queryFn: async () => {
      if (!messageId) return null
      return fetchMailMessage(messageId)
    },
    enabled: computed(() => !!messageId)
  })
}

// --- Query: State counts ---

export function useStateCountsQuery(accountId?: string, localState?: LocalMessageState) {
  return useQuery<WorkflowStateCountItem[]>({
    queryKey: ['communications-state-counts', accountId, localState],
    queryFn: async () => {
      const res = await fetchMessageStateCounts(accountId, localState)
      return res.counts
    }
  })
}

// --- Query: Sync statuses ---

export function useSyncStatusesQuery() {
  return useQuery<MailSyncStatus[]>({
    queryKey: ['communications-sync-statuses'],
    queryFn: async () => {
      const res = await fetchMailSyncStatus()
      return res.items
    }
  })
}

// --- Query: Drafts ---

export function useDraftsQuery(accountId?: string) {
  return useQuery<EmailDraft[]>({
    queryKey: ['communications-drafts', accountId],
    queryFn: async () => {
      const res = await fetchDrafts(accountId)
      return res.items
    }
  })
}

// --- Query: Mailbox health ---

export function useMailboxHealthQuery(accountId?: string) {
  return useQuery<MailboxHealth | null>({
    queryKey: ['communications-mailbox-health', accountId],
    queryFn: async () => {
      return fetchMailboxHealth(accountId)
    }
  })
}

// --- Query: Threads ---

export function useConversationsQuery(accountId?: string) {
  return useQuery({
    queryKey: ['communications-threads', accountId],
    queryFn: async () => {
      const res = await fetchThreads(accountId)
      return res.items
    }
  })
}

// --- Mutation: Send mail ---

export function useSendMailMutation() {
  const queryClient = useQueryClient()
  return useMutation<SendEmailResponse, Error, SendEmailRequest>({
    mutationFn: async (request: SendEmailRequest) => {
      return sendEmail(request)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}

// --- Mutation: Save draft ---

export function useSaveDraftMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (draft: Record<string, unknown>) => {
      return createDraft(draft)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}

// --- Mutation: Delete draft ---

export function useDeleteDraftMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (draftId: string) => {
      return deleteDraft(draftId)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}
