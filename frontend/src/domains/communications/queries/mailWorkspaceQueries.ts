import { useInfiniteQuery, useMutation, useQuery, useQueryClient, type InfiniteData } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  copyMessageToFolder,
  createMailCertificate,
  createMailFolder,
  createSavedSearch,
  deleteMailFolder,
  deleteRichTemplate,
  deleteSavedSearch,
  fetchFolderMessages,
  fetchMailBlockers,
  fetchExpiringMailCertificates,
  fetchMailCertificates,
  fetchMailFolders,
  fetchRichTemplates,
  fetchSavedSearches,
  fetchSubscriptions,
  fetchTopSenders,
  inspectAttachmentArchive,
  moveMessageToFolder,
  previewAttachment,
  previewRichTemplateMailMerge,
  renderRichTemplate,
  saveRichTemplate,
  searchAttachments,
  translateAttachment,
  updateMailFolder,
  updateSavedSearch
} from '../api/communications'
import type {
  CommunicationTemplate,
  MailArchitectureBlocker,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse,
  SenderStats,
  SenderStatsListResponse,
  SubscriptionListResponse,
  SubscriptionSource
} from '../types/communications'
import type {
  MailCertificate,
  MailCertificateCreateRequest
} from '../types/certificates'
import type {
  AttachmentArchiveInspectionResponse,
  AttachmentPreviewResponse,
  AttachmentSearchRequest,
  AttachmentSearchResponse,
  AttachmentSearchResult,
  AttachmentTranslationRequest,
  AttachmentTranslationResponse
} from '../types/attachments'
import type {
  FolderDeleteResponse,
  FolderMessage,
  FolderMessageActionResponse,
  FolderMessageListResponse,
  MailFolder,
  MailFolderInput,
  MailFolderListResponse,
  MailFolderUpdate
} from '../types/folders'
import type {
  MailSavedSearch,
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate
} from '../types/savedSearches'
import type { NullableQueryParam, QueryParam } from './queryTypes'
import {
  mailDetailQueryOptions,
  mailRealtimeQueryOptions,
  mailReferenceQueryOptions
} from './mailQueryPolicies'
import {
  optimisticFolderFromUpdate,
  removeFolderFromFolderList,
  upsertFolderInFolderList
} from './optimisticFolderUpdates'
import {
  findCachedFolderMessage,
  optimisticFolderMessageForTarget,
  removeFolderMessageFromFolderList,
  upsertFolderMessageInFolderList,
  type FolderMessageListCache
} from './optimisticFolderMessageUpdates'

type FolderMutationContext = {
  previousFolderLists: Array<[readonly unknown[], InfiniteData<MailFolderListResponse> | undefined]>
}

type FolderMessageMutationContext = {
  previousFolderMessageLists: FolderMessageListCache
}

export function useRichTemplatesQuery() {
  return useQuery<CommunicationTemplate[]>({
    queryKey: ['communications-rich-templates'],
    queryFn: async () => {
      const res = await fetchRichTemplates()
      return res.templates
    },
    ...mailReferenceQueryOptions
  })
}

export function useSubscriptionsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<SubscriptionListResponse, Error, SubscriptionSource[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-subscriptions', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchSubscriptions(toValue(accountId), 25, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useTopSendersQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<SenderStatsListResponse, Error, SenderStats[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-top-senders', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchTopSenders(toValue(accountId), 25, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useMailBlockersQuery() {
  return useQuery<MailArchitectureBlocker[]>({
    queryKey: ['communications-mail-blockers'],
    queryFn: async () => fetchMailBlockers(),
    ...mailReferenceQueryOptions
  })
}

export function useMailCertificatesQuery() {
  return useQuery<MailCertificate[]>({
    queryKey: ['communications-certificates'],
    queryFn: async () => {
      const res = await fetchMailCertificates()
      return res.items
    },
    ...mailReferenceQueryOptions
  })
}

export function useExpiringMailCertificatesQuery(days: QueryParam<number> = 90) {
  return useQuery<MailCertificate[]>({
    queryKey: computed(() => ['communications-certificates-expiring', toValue(days)]),
    queryFn: async () => {
      const res = await fetchExpiringMailCertificates(toValue(days))
      return res.items
    },
    ...mailReferenceQueryOptions
  })
}

export function useCreateMailCertificateMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailCertificate, Error, MailCertificateCreateRequest>({
    mutationFn: async (request) => createMailCertificate(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-certificates'] })
      queryClient.invalidateQueries({ queryKey: ['communications-certificates-expiring'] })
    }
  })
}

export function useSavedSearchesQuery(isSmartFolder?: QueryParam<boolean>, accountId?: QueryParam<string>) {
  return useInfiniteQuery<SavedSearchListResponse, Error, MailSavedSearch[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-saved-searches', toValue(isSmartFolder), toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      return fetchSavedSearches(toValue(isSmartFolder), toValue(accountId), 100, pageParam)
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useMailFoldersQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<MailFolderListResponse, Error, MailFolder[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-folders', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      return fetchMailFolders(toValue(accountId), 500, pageParam)
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useFolderMessagesQuery(
  folderId: NullableQueryParam<string>,
  enabled: QueryParam<boolean> = true
) {
  return useInfiniteQuery<FolderMessageListResponse, Error, FolderMessage[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-folder-messages', toValue(folderId)]),
    initialPageParam: null,
    enabled: computed(() => !!toValue(folderId) && toValue(enabled)),
    queryFn: async ({ pageParam }) => {
      const id = toValue(folderId)
      if (!id) {
        return { items: [], next_cursor: null, has_more: false }
      }
      return fetchFolderMessages(id, 250, pageParam)
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailRealtimeQueryOptions
  })
}

export function useAttachmentSearchQuery(
  request: MaybeRefOrGetter<AttachmentSearchRequest>,
  enabled: QueryParam<boolean> = true
) {
  return useInfiniteQuery<AttachmentSearchResponse, Error, AttachmentSearchResult[], readonly unknown[], string | null>({
    queryKey: computed(() => {
      const value = toValue(request)
      return [
        'communications-attachment-search',
        value.account_id,
        value.q,
        value.content_type,
        value.scan_status,
        value.limit
      ]
    }),
    initialPageParam: null,
    enabled: computed(() => toValue(enabled)),
    queryFn: async ({ pageParam }) => {
      return searchAttachments({ ...toValue(request), cursor: pageParam })
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...mailDetailQueryOptions
  })
}

export function useAttachmentArchiveInspectionQuery(
  attachmentId: NullableQueryParam<string>,
  enabled: QueryParam<boolean> = true
) {
  return useQuery<AttachmentArchiveInspectionResponse | null>({
    queryKey: computed(() => ['communications-attachment-archive-inspection', toValue(attachmentId)]),
    queryFn: async () => {
      const id = toValue(attachmentId)
      if (!id) return null
      return inspectAttachmentArchive(id)
    },
    enabled: computed(() => Boolean(toValue(attachmentId)) && toValue(enabled)),
    ...mailDetailQueryOptions
  })
}

export function useAttachmentPreviewQuery(
  attachmentId: NullableQueryParam<string>,
  enabled: QueryParam<boolean> = true
) {
  return useQuery<AttachmentPreviewResponse | null>({
    queryKey: computed(() => ['communications-attachment-preview', toValue(attachmentId)]),
    queryFn: async () => {
      const id = toValue(attachmentId)
      if (!id) return null
      return previewAttachment(id)
    },
    enabled: computed(() => Boolean(toValue(attachmentId)) && toValue(enabled)),
    ...mailDetailQueryOptions
  })
}

export function useTranslateAttachmentMutation() {
  return useMutation<
    AttachmentTranslationResponse,
    Error,
    { attachmentId: string; request: AttachmentTranslationRequest }
  >({
    mutationFn: async ({ attachmentId, request }) => translateAttachment(attachmentId, request)
  })
}

export function useCreateRichTemplateMutation() {
  const queryClient = useQueryClient()
  return useMutation<RichTemplateUpsertResponse, Error, RichTemplateUpsertRequest>({
    mutationFn: async (request) => saveRichTemplate(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-rich-templates'] })
    }
  })
}

export function useDeleteRichTemplateMutation() {
  const queryClient = useQueryClient()
  return useMutation<RichTemplateDeleteResponse, Error, string>({
    mutationFn: async (templateId) => deleteRichTemplate(templateId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-rich-templates'] })
    }
  })
}

export function useRenderRichTemplateMutation() {
  return useMutation<RichTemplateRenderResponse, Error, RichTemplateRenderRequest>({
    mutationFn: async (request) => renderRichTemplate(request)
  })
}

export function usePreviewRichTemplateMailMergeMutation() {
  return useMutation<
    RichTemplateMailMergePreviewResponse,
    Error,
    RichTemplateMailMergePreviewRequest
  >({
    mutationFn: async (request) => previewRichTemplateMailMerge(request)
  })
}

export function useCreateSavedSearchMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSavedSearch, Error, SavedSearchInput>({
    mutationFn: async (request: SavedSearchInput) => createSavedSearch(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-saved-searches'] })
    }
  })
}

export function useUpdateSavedSearchMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailSavedSearch,
    Error,
    { savedSearchId: string; request: SavedSearchUpdate }
  >({
    mutationFn: async ({ savedSearchId, request }) => updateSavedSearch(savedSearchId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-saved-searches'] })
    }
  })
}

export function useDeleteSavedSearchMutation() {
  const queryClient = useQueryClient()
  return useMutation<SavedSearchDeleteResponse, Error, string>({
    mutationFn: async (savedSearchId: string) => deleteSavedSearch(savedSearchId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-saved-searches'] })
    }
  })
}

export function useCreateMailFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailFolder, Error, MailFolderInput, FolderMutationContext>({
    mutationFn: async (request: MailFolderInput) => createMailFolder(request),
    onMutate: async () => {
      await queryClient.cancelQueries({ queryKey: ['communications-folders'] })
      return {
        previousFolderLists: queryClient.getQueriesData<InfiniteData<MailFolderListResponse>>({
          queryKey: ['communications-folders']
        })
      }
    },
    onError: (_error, _request, context) => {
      restoreFolderLists(queryClient, context)
    },
    onSuccess: (folder) => {
      patchFolderLists(queryClient, folder)
      queryClient.invalidateQueries({ queryKey: ['communications-folders'] })
    }
  })
}

export function useUpdateMailFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailFolder,
    Error,
    { folderId: string; request: MailFolderUpdate },
    FolderMutationContext
  >({
    mutationFn: async ({ folderId, request }) => updateMailFolder(folderId, request),
    onMutate: async ({ folderId, request }) => {
      await queryClient.cancelQueries({ queryKey: ['communications-folders'] })
      const previousFolderLists = queryClient.getQueriesData<InfiniteData<MailFolderListResponse>>({
        queryKey: ['communications-folders']
      })
      const optimisticFolder = findCachedFolder(previousFolderLists, folderId)

      if (optimisticFolder) {
        patchFolderLists(
          queryClient,
          optimisticFolderFromUpdate(optimisticFolder, request, new Date().toISOString())
        )
      }

      return { previousFolderLists }
    },
    onError: (_error, _variables, context) => {
      restoreFolderLists(queryClient, context)
    },
    onSuccess: (folder, variables) => {
      patchFolderLists(queryClient, folder)
      queryClient.invalidateQueries({ queryKey: ['communications-folders'] })
      queryClient.invalidateQueries({ queryKey: ['communications-folder-messages', variables.folderId] })
    }
  })
}

export function useDeleteMailFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation<FolderDeleteResponse, Error, string, FolderMutationContext>({
    mutationFn: async (folderId: string) => deleteMailFolder(folderId),
    onMutate: async (folderId) => {
      await queryClient.cancelQueries({ queryKey: ['communications-folders'] })
      const previousFolderLists = queryClient.getQueriesData<InfiniteData<MailFolderListResponse>>({
        queryKey: ['communications-folders']
      })

      for (const [queryKey, data] of previousFolderLists) {
        queryClient.setQueryData(queryKey, removeFolderFromFolderList(data, folderId))
      }

      return { previousFolderLists }
    },
    onError: (_error, _folderId, context) => {
      restoreFolderLists(queryClient, context)
    },
    onSuccess: (_result, folderId) => {
      queryClient.invalidateQueries({ queryKey: ['communications-folders'] })
      queryClient.invalidateQueries({ queryKey: ['communications-folder-messages', folderId] })
    }
  })
}

function patchFolderLists(
  queryClient: ReturnType<typeof useQueryClient>,
  folder: MailFolder
): void {
  for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<MailFolderListResponse>>({
    queryKey: ['communications-folders']
  })) {
    queryClient.setQueryData(queryKey, upsertFolderInFolderList(data, queryKey, folder))
  }
}

function restoreFolderLists(
  queryClient: ReturnType<typeof useQueryClient>,
  context: FolderMutationContext | undefined
): void {
  if (!context) return
  for (const [queryKey, data] of context.previousFolderLists) {
    queryClient.setQueryData(queryKey, data)
  }
}

function findCachedFolder(
  folderLists: Array<[readonly unknown[], InfiniteData<MailFolderListResponse> | undefined]>,
  folderId: string
): MailFolder | undefined {
  for (const [, data] of folderLists) {
    for (const page of data?.pages ?? []) {
      const folder = page.items.find((item) => item.folder_id === folderId)
      if (folder) return folder
    }
  }
  return undefined
}

export function useCopyMessageToFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    FolderMessageActionResponse,
    Error,
    { folderId: string; messageId: string },
    FolderMessageMutationContext
  >({
    mutationFn: async ({ folderId, messageId }) => copyMessageToFolder(folderId, messageId),
    onMutate: async ({ folderId, messageId }) => {
      await queryClient.cancelQueries({ queryKey: ['communications-folder-messages'] })
      const previousFolderMessageLists = queryClient.getQueriesData<InfiniteData<FolderMessageListResponse>>({
        queryKey: ['communications-folder-messages']
      })
      const sourceMessage = findCachedFolderMessage(previousFolderMessageLists, messageId)

      if (sourceMessage) {
        patchFolderMessageLists(
          queryClient,
          optimisticFolderMessageForTarget(sourceMessage, folderId, new Date().toISOString())
        )
      }

      return { previousFolderMessageLists }
    },
    onError: (_error, _variables, context) => {
      restoreFolderMessageLists(queryClient, context)
    },
    onSuccess: (result, variables) => {
      patchFolderMessageLists(queryClient, result.message)
      queryClient.invalidateQueries({ queryKey: ['communications-folder-messages', variables.folderId] })
      queryClient.invalidateQueries({ queryKey: ['communications-message', variables.messageId] })
    }
  })
}

export function useMoveMessageToFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    FolderMessageActionResponse,
    Error,
    { folderId: string; messageId: string },
    FolderMessageMutationContext
  >({
    mutationFn: async ({ folderId, messageId }) => moveMessageToFolder(folderId, messageId),
    onMutate: async ({ folderId, messageId }) => {
      await queryClient.cancelQueries({ queryKey: ['communications-folder-messages'] })
      const previousFolderMessageLists = queryClient.getQueriesData<InfiniteData<FolderMessageListResponse>>({
        queryKey: ['communications-folder-messages']
      })
      const sourceMessage = findCachedFolderMessage(previousFolderMessageLists, messageId)

      for (const [queryKey, data] of previousFolderMessageLists) {
        if (queryKey[1] !== folderId) {
          queryClient.setQueryData(queryKey, removeFolderMessageFromFolderList(data, messageId))
        }
      }

      if (sourceMessage) {
        patchFolderMessageLists(
          queryClient,
          optimisticFolderMessageForTarget(sourceMessage, folderId, new Date().toISOString())
        )
      }

      return { previousFolderMessageLists }
    },
    onError: (_error, _variables, context) => {
      restoreFolderMessageLists(queryClient, context)
    },
    onSuccess: (result, variables) => {
      patchFolderMessageMoveLists(queryClient, result.message, variables.messageId)
      queryClient.invalidateQueries({ queryKey: ['communications-folder-messages'] })
      queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-message', variables.messageId] })
    }
  })
}

function patchFolderMessageLists(
  queryClient: ReturnType<typeof useQueryClient>,
  folderMessage: FolderMessageActionResponse['message']
): void {
  for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<FolderMessageListResponse>>({
    queryKey: ['communications-folder-messages']
  })) {
    const updated = upsertFolderMessageInFolderList(data, queryKey, folderMessage)
    if (updated !== data) {
      queryClient.setQueryData(queryKey, updated)
    }
  }
}

function patchFolderMessageMoveLists(
  queryClient: ReturnType<typeof useQueryClient>,
  folderMessage: FolderMessageActionResponse['message'],
  messageId: string
): void {
  for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<FolderMessageListResponse>>({
    queryKey: ['communications-folder-messages']
  })) {
    const updated =
      queryKey[1] === folderMessage.folder_id
        ? upsertFolderMessageInFolderList(data, queryKey, folderMessage)
        : removeFolderMessageFromFolderList(data, messageId)

    if (updated !== data) {
      queryClient.setQueryData(queryKey, updated)
    }
  }
}

function restoreFolderMessageLists(
  queryClient: ReturnType<typeof useQueryClient>,
  context: FolderMessageMutationContext | undefined
): void {
  if (!context) return
  for (const [queryKey, data] of context.previousFolderMessageLists) {
    queryClient.setQueryData(queryKey, data)
  }
}
