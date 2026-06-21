import { computed, onMounted, ref, watch } from 'vue'
import {
  useBulkMessageActionMutation,
  useConversationsQuery,
  useDeleteDraftMutation,
  useDraftsQuery,
  useMailboxHealthQuery,
  useMailListQuery,
  useMessageQuery,
  useStateCountsQuery,
  useSyncStatusesQuery,
  useThreadMessagesQuery
} from '../queries/useCommunicationsQuery'
import { useFolderMailList } from '../queries/folderMailList'
import { useOutboxStatusStrip } from '../queries/outboxStatusStrip'
import { useMailResourceOverview } from './useMailResourceOverview'
import { draftToComposeForm } from '../helpers/communicationPageModels'
import {
  communicationSectionWorkflowState,
  communicationWorkflowStateSectionId,
  useCommunicationsStore
} from '../stores/communications'
import type {
  BulkMessageActionRequest,
  CommunicationSectionId,
  CommunicationDraft,
  CommunicationThreadSummary
} from '../types/communications'
import type { CommunicationSavedSearch } from '../types/savedSearches'
import { useMailSyncActions } from './useMailSyncActions'
import { useThreadReplyActions } from './useThreadReplyActions'
import { useSelectedMessageActions } from './useSelectedMessageActions'

type BulkActionCommand = Omit<BulkMessageActionRequest, 'message_ids'>

export function useCommunicationsPageController() {
  const store = useCommunicationsStore()
  const isAccountSetupOpen = ref(false)
  const inspectorVisible = ref(true)
  const activeSavedSearchId = ref('')
  const activeFolderId = ref('')
  const savedSearchChannelKind = ref<string>()

  const {
    data: mailListData,
    error: mailListError,
    isLoading: isMailListLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
    refetch: refetchMailList
  } = useMailListQuery(
    () => store.selectedMailAccountId || undefined,
    () => store.mailStateFilter || undefined,
    () => savedSearchChannelKind.value,
    () => store.messageSearchQuery || undefined,
    () => store.mailLocalStateFilter
  )
  const {
    data: messageDetailData,
    refetch: refetchMessageDetail
  } = useMessageQuery(() => store.selectedCommunicationMessageId || null)
  const {
    data: stateCountsData,
    refetch: refetchStateCounts
  } = useStateCountsQuery(() => store.selectedMailAccountId || undefined, () => store.mailLocalStateFilter)
  const {
    data: syncStatusesData,
    refetch: refetchSyncStatuses
  } = useSyncStatusesQuery()
  const {
    data: draftsData,
    refetch: refetchDrafts,
    hasNextPage: hasDraftNextPage,
    isFetchingNextPage: isFetchingDraftNextPage,
    fetchNextPage: fetchNextDraftPage
  } = useDraftsQuery(() => store.selectedMailAccountId || undefined)
  const {
    data: mailboxHealthData,
    refetch: refetchMailboxHealth
  } = useMailboxHealthQuery(() => store.selectedMailAccountId || undefined)
  const resourceOverview = useMailResourceOverview(() => store.selectedMailAccountId || undefined)
  const {
    data: conversationsData,
    isLoading: isThreadListLoading,
    hasNextPage: hasThreadNextPage,
    isFetchingNextPage: isFetchingThreadNextPage,
    fetchNextPage: fetchNextThreadPage
  } = useConversationsQuery(() => store.selectedMailAccountId || undefined)
  const {
    data: threadMessagesData,
    isLoading: isSelectedThreadLoading,
    error: selectedThreadMessagesError
  } = useThreadMessagesQuery(
    () => store.selectedMailAccountId || null,
    () => store.selectedThread?.subject ?? null
  )
  const deleteDraftMutation = useDeleteDraftMutation()
  const bulkMessageActionMutation = useBulkMessageActionMutation()
  const {
    outboxItems,
    outboxErrorMessage,
    isOutboxLoading,
    isLoadingMoreOutbox,
    hasMoreOutboxItems,
    isUndoingOutbox,
    undoOutbox,
    loadMoreOutboxItems,
    prefetchMoreOutboxItems
  } = useOutboxStatusStrip(() => store.selectedMailAccountId || undefined, {
    onStatus: (message) => store.setMailActionStatus(message),
    onError: (message) => store.setMailActionError(message)
  })

  const mailList = computed(() => mailListData.value ?? [])
  const messageDetail = computed(() => messageDetailData.value ?? null)
  const stateCounts = computed(() => stateCountsData.value ?? [])
  const drafts = computed(() => draftsData.value ?? [])
  const hasMoreDrafts = computed(() => Boolean(hasDraftNextPage.value))
  const isLoadingMoreDrafts = computed(() => isFetchingDraftNextPage.value)
  const mailboxHealth = computed(() => mailboxHealthData.value ?? null)
  const {
    areResourcesLoading,
    blockers,
    handleLoadMoreSubscriptions,
    handleLoadMoreTopSenders,
    hasMoreSubscriptions,
    hasMoreTopSenders,
    isLoadingMoreSubscriptions,
    isLoadingMoreTopSenders,
    subscriptions,
    topSenders
  } = resourceOverview
  const selectedThreadMessages = computed(() => threadMessagesData.value?.items ?? [])
  const selectedThreadErrorMessage = computed(() => {
    if (!selectedThreadMessagesError.value) return ''
    return selectedThreadMessagesError.value instanceof Error
      ? selectedThreadMessagesError.value.message
      : 'Failed to load conversation'
  })
  const hasRail = computed(() => inspectorVisible.value && messageDetail.value !== null)
  const selectedBulkCount = computed(() => store.selectedMessageIds.length)
  const isBulkActionRunning = computed(() => bulkMessageActionMutation.isPending.value)
  const mailListErrorMessage = computed(() => {
    if (!mailListError.value) return ''
    return mailListError.value instanceof Error ? mailListError.value.message : 'Failed to load messages'
  })
  const folderMail = useFolderMailList(() => activeFolderId.value)
  const visibleMailList = computed(() => activeFolderId.value ? folderMail.messages.value : mailList.value)
  const visibleMailListErrorMessage = computed(() => activeFolderId.value ? folderMail.errorMessage.value : mailListErrorMessage.value)
  const isVisibleMailListLoading = computed(() => activeFolderId.value ? folderMail.isLoading.value : isMailListLoading.value)
  const isNavigatorListLoading = computed(() =>
    !activeFolderId.value && store.communicationsNavigatorMode === 'threads'
      ? isThreadListLoading.value
      : isVisibleMailListLoading.value
  )
  const hasVisibleNextPage = computed(() => activeFolderId.value ? Boolean(folderMail.hasNextPage.value) : Boolean(hasNextPage.value))
  const isFetchingVisibleNextPage = computed(() => activeFolderId.value ? folderMail.isFetchingNextPage.value : isFetchingNextPage.value)
  const activeSectionId = computed<CommunicationSectionId>(() =>
    communicationWorkflowStateSectionId(store.mailStateFilter)
  )

  watch(visibleMailList, (items) => store.setMessages(items))
  watch(messageDetailData, (detail) => store.setMessageDetail(detail ?? null))
  watch(stateCountsData, (counts) => store.setStateCounts(counts ?? []))
  watch(syncStatusesData, (statuses) => store.setMailSyncStatuses(statuses ?? []))
  watch(draftsData, (items) => store.setDrafts(items ?? []))
  watch(mailboxHealthData, (health) => store.setMailboxHealth(health ?? null))
  watch(conversationsData, (threads) => {
    store.setThreads((threads ?? []).map((thread) => ({
      thread_id: thread.thread_id,
      subject: thread.subject,
      message_count: thread.message_count,
      participant_count: thread.participant_count,
      last_activity_at: thread.last_activity_at,
      has_open_action: thread.has_open_action,
      has_attachments: thread.has_attachments,
      dominant_workflow_state: thread.dominant_workflow_state
    })))
  })

  function resetSelectedMessageContext() {
    store.selectMessage(-1)
    store.clearSelectedThread()
    store.setMessageDetail(null)
    store.setCommunicationMessageInsight(null)
  }

  function handleSearchQueryUpdate(query: string) {
    activeSavedSearchId.value = ''
    activeFolderId.value = ''
    store.setMessageSearchQuery(query)
    resetSelectedMessageContext()
  }

  function handleLoadMoreMessages() {
    if (activeFolderId.value) {
      if (folderMail.hasNextPage.value && !folderMail.isFetchingNextPage.value) void folderMail.fetchNextPage()
      return
    }
    if (!hasNextPage.value || isFetchingNextPage.value) return
    void fetchNextPage()
  }

  function handleLoadMoreThreads() {
    if (!hasThreadNextPage.value || isFetchingThreadNextPage.value) return
    void fetchNextThreadPage()
  }

  function handleLoadMoreDrafts() {
    if (!hasDraftNextPage.value || isFetchingDraftNextPage.value) return
    void fetchNextDraftPage()
  }

  function handleSelectMessage(index: number) {
    store.selectMessage(index)
    store.setActiveMessageContextTab('message')
    store.setCommunicationMessageInsight(null)
  }

  function handleSelectThread(thread: CommunicationThreadSummary) {
    store.selectThread(thread)
    store.setActiveMessageContextTab('message')
  }

  function handleOpenThreadMessage(messageId: string) {
    store.selectMessageId(messageId)
    store.setActiveMessageContextTab('message')
    store.setCommunicationMessageInsight(null)
  }

  const {
    handleReplyToThreadMessage,
    handleSaveThreadReplyDraft,
    handleSendThreadReply,
    isThreadReplySending
  } = useThreadReplyActions(store)

  async function handleBulkAction(command: BulkActionCommand) {
    const messageIds = [...store.selectedMessageIds]
    if (messageIds.length === 0) return
    store.setIsMailActionRunning(true)
    try {
      const result = await bulkMessageActionMutation.mutateAsync({
        ...command,
        message_ids: messageIds
      })
      store.setMailActionStatus(`${result.updated_count} messages updated`)
      store.clearMessageSelection()
      await Promise.all([refetchMailList(), refetchStateCounts()])
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Bulk action failed')
    } finally {
      store.setIsMailActionRunning(false)
    }
  }

  const {
    handleAddLabel,
    handleDeleteFromProvider,
    handleAnalyze,
    handleApplyAiReply,
    handleBilingualReplySend,
    handleCreateNote,
    handleCreateTask,
    handleExportMessage,
    handleMarkMessageRead,
    handleMarkMessageUnread,
    handleForwardMessage,
    handleGenerateAiReply,
    handleMute,
    handleNewMessage,
    handleRedirectMessage,
    handleRemoveLabel,
    handleReply,
    handleReplyAll,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSnoozeMessage,
    handleToggleImportant,
    handleTogglePin,
    handleTranslate
  } = useSelectedMessageActions(store, {
    getMessageDetail: () => messageDetail.value?.message ?? null,
    refetchMessageDetail
  })

  function selectSection(sectionId: CommunicationSectionId) {
    const workflowState = communicationSectionWorkflowState(sectionId)
    if (workflowState === null) return
    activeSavedSearchId.value = ''
    activeFolderId.value = ''
    savedSearchChannelKind.value = undefined
    store.setStateFilter(workflowState)
    store.setLocalStateFilter('active')
    resetSelectedMessageContext()
  }

  function handleSavedSearchSelect(savedSearch: CommunicationSavedSearch) {
    activeSavedSearchId.value = savedSearch.saved_search_id
    activeFolderId.value = ''
    savedSearchChannelKind.value = savedSearch.channel_kind ?? undefined
    store.setMessageSearchQuery(savedSearch.query)
    store.setStateFilter(savedSearch.workflow_state ?? '')
    store.setLocalStateFilter(savedSearch.local_state)
    resetSelectedMessageContext()
  }

  function handleSavedSearchDeleted(savedSearch: CommunicationSavedSearch) {
    if (activeSavedSearchId.value !== savedSearch.saved_search_id) return
    activeSavedSearchId.value = ''
    savedSearchChannelKind.value = undefined
    store.setMessageSearchQuery('')
    store.setStateFilter('')
    store.setLocalStateFilter('active')
    resetSelectedMessageContext()
  }

  function handleFolderSelect(folderId: string) {
    activeFolderId.value = activeFolderId.value === folderId ? '' : folderId
    activeSavedSearchId.value = ''
    resetSelectedMessageContext()
  }

  function handleFolderDeleted() {
    activeFolderId.value = ''
    resetSelectedMessageContext()
  }

  function handleOpenDraft(draft: CommunicationDraft) {
    store.openCompose(draftToComposeForm(draft))
  }

  async function handleDeleteDraft(draftId: string) {
    try {
      await deleteDraftMutation.mutateAsync(draftId)
      await refetchDrafts()
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Delete draft failed')
    }
  }

  const {
    clearSyncStatus,
    handleUpdateSyncSettings,
    handleSyncNow,
    isSyncSettingsLoading,
    isSyncSettingsSaving,
    selectedMailSyncSettings,
    loadInitialData
  } = useMailSyncActions(store, {
    refetchMailList,
    refetchMailboxHealth,
    refetchStateCounts,
    refetchSyncStatuses
  })
  const selectedMailSyncSettingsValue = computed(() => selectedMailSyncSettings.value ?? null)

  onMounted(() => {
    void loadInitialData()
  })

  return {
    activeFolderId,
    activeSavedSearchId,
    activeSectionId,
    areResourcesLoading,
    blockers,
    clearSyncStatus,
    drafts,
    hasMoreDrafts,
    hasMoreSubscriptions,
    hasMoreTopSenders,
    isLoadingMoreDrafts,
    isLoadingMoreSubscriptions,
    isLoadingMoreTopSenders,
    handleAnalyze,
    handleBilingualReplySend,
    handleBulkAction,
    handleCreateNote,
    handleCreateTask,
    handleDeleteDraft,
    handleFolderDeleted,
    handleFolderSelect,
    handleAddLabel,
    handleApplyAiReply,
    handleGenerateAiReply,
    handleUpdateSyncSettings,
    handleLoadMoreMessages,
    handleLoadMoreThreads,
    handleLoadMoreDrafts,
    handleLoadMoreSubscriptions,
    handleLoadMoreTopSenders,
    handleMute,
    handleNewMessage,
    handleOpenDraft,
    handleOpenThreadMessage,
    handleForwardMessage,
    handleReply,
    handleReplyToThreadMessage,
    handleReplyAll,
    handleRedirectMessage,
    handleSavedSearchDeleted,
    handleSavedSearchSelect,
    handleRemoveLabel,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSaveThreadReplyDraft,
    handleSearchQueryUpdate,
    handleSelectMessage,
    handleSelectThread,
    handleSendThreadReply,
    handleSnoozeMessage,
    handleMarkMessageRead,
    handleMarkMessageUnread,
    handleDeleteFromProvider,
    handleSyncNow,
    handleToggleImportant,
    handleTogglePin,
    handleTranslate,
    handleExportMessage,
    hasMoreOutboxItems,
    hasRail,
    hasThreadNextPage,
    hasVisibleNextPage,
    isAccountSetupOpen,
    isBulkActionRunning,
    isFetchingThreadNextPage,
    isFetchingVisibleNextPage,
    isLoadingMoreOutbox,
    isNavigatorListLoading,
    isOutboxLoading,
    isSelectedThreadLoading,
    savedSearchChannelKind,
    isSyncSettingsLoading,
    isSyncSettingsSaving,
    isThreadReplySending,
    isUndoingOutbox,
    loadMoreOutboxItems,
    mailboxHealth,
    messageDetail,
    outboxErrorMessage,
    outboxItems,
    prefetchMoreOutboxItems,
    refetchMailList,
    selectedBulkCount,
    selectedMailSyncSettings: selectedMailSyncSettingsValue,
    selectedThreadErrorMessage,
    selectedThreadMessages,
    selectSection,
    stateCounts,
    store,
    subscriptions,
    topSenders,
    undoOutbox,
    visibleMailList,
    visibleMailListErrorMessage
  }
}
