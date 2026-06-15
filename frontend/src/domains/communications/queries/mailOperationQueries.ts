import { useInfiniteQuery, useMutation, useQueryClient, type InfiniteData } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  bulkMessageAction,
  createDraft,
  deleteDraft,
  fetchDrafts,
  fetchOutboxItems,
  redirectMessage,
  sendEmail,
  undoOutboxItem
} from '../api/communications'
import { updateMessageAiState } from '../api/aiState'
import { prepareBilingualReplyFlow } from '../api/bilingualReplyFlow'
import type { BilingualReplyFlowRequest, BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'
import type {
  BulkMessageActionRequest,
  BulkMessageActionResponse,
  DraftListResponse,
  EmailDraft,
  EmailOutboxItem,
  EmailOutboxStatus,
  MailMessageDetailResponse,
  MailMessagesResponse,
  OutboxListResponse,
  RedirectMessageRequest,
  SendEmailRequest,
  SendEmailResponse
} from '../types/communications'
import type { MailAiStateRecord, MailAiStateTransitionRequest } from '../types/aiState'
import {
  applyBulkMessageActionToMailDetail,
  applyBulkMessageActionToMailList,
  markOutboxItemCanceled,
  removeDraftFromDraftList,
  upsertDraftInDraftList
} from './optimisticMailUpdates'
import { mailMessageQueryKey } from './mailPrefetch'
import { mailRealtimeQueryOptions } from './mailQueryPolicies'
import type { QueryParam } from './queryTypes'
import type { ComposeDraftPayload } from '../forms/composeDraftAutosave'

type BulkMessageActionMutationContext = {
  previousMailLists: Array<[readonly unknown[], InfiniteData<MailMessagesResponse> | undefined]>
  previousMessages: Array<[
    readonly ['communications-message', string],
    MailMessageDetailResponse | null | undefined
  ]>
}

type DraftMutationContext = {
  previousDraftLists: Array<[readonly unknown[], InfiniteData<DraftListResponse> | undefined]>
}

type OutboxMutationContext = {
  previousOutboxLists: Array<[readonly unknown[], InfiniteData<OutboxListResponse> | undefined]>
}

export function useDraftsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<DraftListResponse, Error, EmailDraft[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-drafts', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchDrafts(toValue(accountId), undefined, 50, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useOutboxQuery(accountId?: QueryParam<string>, status?: QueryParam<EmailOutboxStatus>) {
  return useInfiniteQuery<OutboxListResponse, Error, EmailOutboxItem[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-outbox', toValue(accountId), toValue(status)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchOutboxItems(toValue(accountId), toValue(status), 100, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useSendMailMutation() {
  const queryClient = useQueryClient()
  return useMutation<SendEmailResponse, Error, SendEmailRequest, DraftMutationContext>({
    mutationFn: async (request: SendEmailRequest) => {
      return sendEmail(request)
    },
    onMutate: async (request) => {
      if (!request.draft_id) return { previousDraftLists: [] }

      await queryClient.cancelQueries({ queryKey: ['communications-drafts'] })
      const previousDraftLists = queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })

      for (const [queryKey, data] of previousDraftLists) {
        queryClient.setQueryData(queryKey, removeDraftFromDraftPages(data, request.draft_id))
      }

      return { previousDraftLists }
    },
    onError: (_error, _request, context) => {
      restoreDraftLists(queryClient, context)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
      queryClient.invalidateQueries({ queryKey: ['communications-outbox'] })
    }
  })
}

export function useSaveDraftMutation() {
  const queryClient = useQueryClient()
  return useMutation<EmailDraft, Error, ComposeDraftPayload, DraftMutationContext>({
    mutationFn: async (draft: ComposeDraftPayload) => {
      return createDraft(draft)
    },
    onMutate: async (draft) => {
      await queryClient.cancelQueries({ queryKey: ['communications-drafts'] })
      const previousDraftLists = queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })
      const optimisticDraft = optimisticDraftFromPayload(draft, previousDraftLists)

      for (const [queryKey, data] of previousDraftLists) {
        if (!draftQueryMatchesAccount(queryKey, optimisticDraft.account_id)) continue
        queryClient.setQueryData(queryKey, upsertDraftInDraftPages(data, optimisticDraft))
      }

      return { previousDraftLists }
    },
    onError: (_error, _draft, context) => {
      restoreDraftLists(queryClient, context)
    },
    onSuccess: (draft) => {
      for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })) {
        if (!draftQueryMatchesAccount(queryKey, draft.account_id)) continue
        queryClient.setQueryData(queryKey, upsertDraftInDraftPages(data, draft))
      }
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}

export function useDeleteDraftMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (draftId: string) => {
      return deleteDraft(draftId)
    },
    onMutate: async (draftId) => {
      await queryClient.cancelQueries({ queryKey: ['communications-drafts'] })
      const previousDraftLists = queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })

      for (const [queryKey, data] of previousDraftLists) {
        queryClient.setQueryData(queryKey, removeDraftFromDraftPages(data, draftId))
      }

      return { previousDraftLists }
    },
    onError: (_error, _draftId, context) => {
      restoreDraftLists(queryClient, context)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}

export function useUndoOutboxMutation() {
  const queryClient = useQueryClient()
  return useMutation<EmailOutboxItem, Error, string, OutboxMutationContext>({
    mutationFn: async (outboxId: string) => {
      return undoOutboxItem(outboxId)
    },
    onMutate: async (outboxId) => {
      await queryClient.cancelQueries({ queryKey: ['communications-outbox'] })
      const previousOutboxLists = queryClient.getQueriesData<InfiniteData<OutboxListResponse>>({
        queryKey: ['communications-outbox']
      })

      for (const [queryKey, data] of previousOutboxLists) {
        queryClient.setQueryData(queryKey, markOutboxItemCanceledInOutboxPage(data, queryKey, outboxId))
      }

      return { previousOutboxLists }
    },
    onError: (_error, _outboxId, context) => {
      restoreOutboxLists(queryClient, context)
    },
    onSuccess: (item) => {
      for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<OutboxListResponse>>({
        queryKey: ['communications-outbox']
      })) {
        queryClient.setQueryData(queryKey, upsertOutboxItemInOutboxPage(data, queryKey, item))
      }
      queryClient.invalidateQueries({ queryKey: ['communications-outbox'] })
    }
  })
}

export function usePrepareBilingualReplyFlowMutation() {
  return useMutation<
    BilingualReplyFlowResponse,
    Error,
    { messageId: string; request: BilingualReplyFlowRequest }
  >({
    mutationFn: async ({ messageId, request }) => {
      return prepareBilingualReplyFlow(messageId, request)
    }
  })
}

export function useRedirectMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation<SendEmailResponse, Error, { messageId: string; request: RedirectMessageRequest }>({
    mutationFn: async ({ messageId, request }) => redirectMessage(messageId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-outbox'] })
    }
  })
}

function restoreDraftLists(
  queryClient: ReturnType<typeof useQueryClient>,
  context: DraftMutationContext | undefined
): void {
  if (!context) return
  for (const [queryKey, data] of context.previousDraftLists) {
    queryClient.setQueryData(queryKey, data)
  }
}

function restoreOutboxLists(
  queryClient: ReturnType<typeof useQueryClient>,
  context: OutboxMutationContext | undefined
): void {
  if (!context) return
  for (const [queryKey, data] of context.previousOutboxLists) {
    queryClient.setQueryData(queryKey, data)
  }
}

function upsertOutboxItemInOutboxPage(
  data: InfiniteData<OutboxListResponse> | undefined,
  queryKey: readonly unknown[],
  item: EmailOutboxItem
): InfiniteData<OutboxListResponse> | undefined {
  if (!data) return data
  if (!outboxQueryMatches(queryKey, item)) {
    return removeOutboxItemFromOutboxPage(data, item.outbox_id)
  }

  const existsInAnyPage = data.pages.some((page) =>
    page.items.some((existing) => existing.outbox_id === item.outbox_id)
  )
  let changed = false
  const pages = data.pages.map((page, pageIndex) => {
    const existingIndex = page.items.findIndex((existing) => existing.outbox_id === item.outbox_id)

    if (existingIndex >= 0) {
      const items = page.items.slice()
      items[existingIndex] = item
      changed = true
      return { ...page, items }
    }

    if (!existsInAnyPage && pageIndex === 0) {
      changed = true
      return { ...page, items: [item, ...page.items] }
    }
    return page
  })

  return changed ? { ...data, pages } : data
}

function markOutboxItemCanceledInOutboxPage(
  data: InfiniteData<OutboxListResponse> | undefined,
  queryKey: readonly unknown[],
  outboxId: string
): InfiniteData<OutboxListResponse> | undefined {
  if (!data) return data
  const queryStatus = queryKey[2]
  if (typeof queryStatus === 'string' && queryStatus !== 'canceled') {
    return removeOutboxItemFromOutboxPage(data, outboxId)
  }

  let changed = false
  const pages = data.pages.map((page) => {
    const items = markOutboxItemCanceled(page.items, outboxId)
    if (items === page.items) return page
    changed = true
    return { ...page, items }
  })

  return changed ? { ...data, pages } : data
}

function removeOutboxItemFromOutboxPage(
  data: InfiniteData<OutboxListResponse>,
  outboxId: string
): InfiniteData<OutboxListResponse> {
  let changed = false
  const pages = data.pages.map((page) => {
    const items = page.items.filter((item) => item.outbox_id !== outboxId)
    if (items.length === page.items.length) return page
    changed = true
    return { ...page, items }
  })

  return changed ? { ...data, pages } : data
}

function optimisticDraftFromPayload(
  draft: ComposeDraftPayload,
  draftLists: Array<[readonly unknown[], InfiniteData<DraftListResponse> | undefined]>
): EmailDraft {
  const existing = findCachedDraft(draftLists, draft.draft_id)
  const now = new Date().toISOString()

  return {
    draft_id: draft.draft_id,
    account_id: draft.account_id,
    persona_id: existing?.persona_id ?? null,
    to_recipients: draft.to_recipients,
    cc_recipients: draft.cc_recipients,
    bcc_recipients: draft.bcc_recipients,
    subject: draft.subject,
    body_text: draft.body_text,
    body_html: draft.body_html,
    in_reply_to: draft.in_reply_to,
    references: existing?.references ?? [],
    status: draft.status,
    scheduled_send_at: draft.scheduled_send_at,
    send_attempts: existing?.send_attempts ?? 0,
    last_error: existing?.last_error ?? null,
    metadata: draft.metadata,
    created_at: existing?.created_at ?? now,
    updated_at: now
  }
}

function findCachedDraft(
  draftLists: Array<[readonly unknown[], InfiniteData<DraftListResponse> | undefined]>,
  draftId: string
): EmailDraft | undefined {
  for (const [, data] of draftLists) {
    for (const page of data?.pages ?? []) {
      const draft = page.items.find((item) => item.draft_id === draftId)
      if (draft) return draft
    }
  }
  return undefined
}

function upsertDraftInDraftPages(
  data: InfiniteData<DraftListResponse> | undefined,
  draft: EmailDraft
): InfiniteData<DraftListResponse> | undefined {
  if (!data) return data
  const firstPage = data.pages[0]
  if (!firstPage) return data

  const pagesWithoutDraft = data.pages.map((page) => ({
    ...page,
    items: page.items.filter((item) => item.draft_id !== draft.draft_id)
  }))
  const [firstPageWithoutDraft, ...remainingPages] = pagesWithoutDraft
  if (!firstPageWithoutDraft) return data

  const updatedFirstPage = {
    ...firstPageWithoutDraft,
    items: upsertDraftInDraftList(firstPageWithoutDraft.items, draft) ?? [draft]
  }
  return {
    ...data,
    pages: [updatedFirstPage, ...remainingPages]
  }
}

function removeDraftFromDraftPages(
  data: InfiniteData<DraftListResponse> | undefined,
  draftId: string
): InfiniteData<DraftListResponse> | undefined {
  if (!data) return data
  let changed = false
  const pages = data.pages.map((page) => {
    const items = removeDraftFromDraftList(page.items, draftId)
    if (items !== page.items) changed = true
    return { ...page, items: items ?? [] }
  })
  return changed ? { ...data, pages } : data
}

function draftQueryMatchesAccount(queryKey: readonly unknown[], accountId: string): boolean {
  const queryAccountId = queryKey[1]
  return typeof queryAccountId !== 'string' || queryAccountId === accountId
}

function outboxQueryMatches(queryKey: readonly unknown[], item: EmailOutboxItem): boolean {
  const queryAccountId = queryKey[1]
  const queryStatus = queryKey[2]

  if (typeof queryAccountId === 'string' && queryAccountId !== item.account_id) return false
  if (typeof queryStatus === 'string' && queryStatus !== item.status) return false
  return true
}

export function useUpdateMessageAiStateMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailAiStateRecord,
    Error,
    { messageId: string; request: MailAiStateTransitionRequest }
  >({
    mutationFn: async ({ messageId, request }) => updateMessageAiState(messageId, request),
    onSuccess: (record) => {
      queryClient.setQueryData(['communications-ai-state', record.message_id], record)
      queryClient.invalidateQueries({ queryKey: ['communications-ai-state', record.message_id] })
      queryClient.invalidateQueries({ queryKey: ['communications-message', record.message_id] })
      queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
    }
  })
}

export function useBulkMessageActionMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    BulkMessageActionResponse,
    Error,
    BulkMessageActionRequest,
    BulkMessageActionMutationContext
  >({
    mutationFn: async (request: BulkMessageActionRequest) => {
      return bulkMessageAction(request)
    },
    onMutate: async (request) => {
      const messageQueryKeys = request.message_ids.map((messageId) => {
        return mailMessageQueryKey(messageId)
      })

      await Promise.all([
        queryClient.cancelQueries({ queryKey: ['communications-mail-list'] }),
        ...messageQueryKeys.map((queryKey) => queryClient.cancelQueries({ queryKey }))
      ])

      const previousMailLists =
        queryClient.getQueriesData<InfiniteData<MailMessagesResponse>>({
          queryKey: ['communications-mail-list']
        })
      const previousMessages = messageQueryKeys.map((queryKey) => {
        return [
          queryKey,
          queryClient.getQueryData<MailMessageDetailResponse | null>(queryKey)
        ] as [
          readonly ['communications-message', string],
          MailMessageDetailResponse | null | undefined
        ]
      })

      for (const [queryKey, data] of previousMailLists) {
        queryClient.setQueryData(
          queryKey,
          applyBulkMessageActionToMailList(data, request, queryKey)
        )
      }

      for (const [queryKey, data] of previousMessages) {
        queryClient.setQueryData(queryKey, applyBulkMessageActionToMailDetail(data, request))
      }

      return { previousMailLists, previousMessages }
    },
    onError: (_error, _request, context) => {
      if (!context) return

      for (const [queryKey, data] of context.previousMailLists) {
        queryClient.setQueryData(queryKey, data)
      }
      for (const [queryKey, data] of context.previousMessages) {
        queryClient.setQueryData(queryKey, data)
      }
    },
    onSettled: (_result, _error, request) => {
      queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
      queryClient.invalidateQueries({ queryKey: ['communications-threads'] })
      for (const messageId of request.message_ids) {
        queryClient.invalidateQueries({ queryKey: ['communications-message', messageId] })
      }
    }
  })
}
