// Historical pre-clean-room page orchestration. It is not part of the active client graph.
import { computed, onMounted, ref, watch } from 'vue'
import {
  useBulkMessageActionMutation,
  useConversationsQuery,
  useDeleteDraftMutation,
  useDraftsQuery,
  useEmailAccountsQuery,
  useMailboxHealthQuery,
  useMailListQuery,
  useMessageQuery,
  useStateCountsQuery,
  useSyncStatusesQuery,
  useThreadMessagesQuery
} from './useCommunicationsQuery'
import { useCommunicationActionNotifications } from './communicationActionNotifications'
import { useFolderMailList } from './folderMailList'
import { useOutboxStatusStrip } from './outboxStatusStrip'
import { useMailResourceOverview } from '../views/useMailResourceOverview'
import { isMailProviderKind } from '../helpers/mailProviderKinds'
import {
  communicationSectionWorkflowState,
  communicationWorkflowStateSectionId,
  useCommunicationsStore
} from '../stores/communications'
import type {
  BulkMessageActionRequest,
  CommunicationAccountOption,
  CommunicationSectionId,
  CommunicationThreadSummary,
  EmailAccountView
} from '../types/communications'
import type { CommunicationSavedSearch } from '../types/savedSearches'
import { useMailSyncActions } from '../views/useMailSyncActions'
import { useThreadReplyActions } from '../views/useThreadReplyActions'
import { useSelectedMessageActions } from '../views/useSelectedMessageActions'
import { useMailComposeActions } from '../views/useMailComposeActions'

type BulkActionCommand = Omit<BulkMessageActionRequest, 'message_ids'>

export function useCommunicationsPageSurface() {
  const store = useCommunicationsStore()
  const notifications = useCommunicationActionNotifications()
  const isAccountSetupOpen = ref(false)
  const inspectorVisible = ref(true)
  const activeSavedSearchId = ref('')
  const activeFolderId = ref('')
  const savedSearchChannelKind = ref<string>()
  const mailChannelKind = computed(() => {
    const value = savedSearchChannelKind.value?.trim()
    return value === 'email' || value === 'mail' ? value : 'mail'
  })

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
    () => mailChannelKind.value,
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
    data: emailAccountsData
  } = useEmailAccountsQuery()
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
    onStatus: (message) => notifications.info('Outbox updated', message),
    onError: (message) => notifications.error('Outbox failed', message)
  })

  const mailList = computed(() => mailListData.value ?? [])
  const messageDetail = computed(() => messageDetailData.value ?? null)
  const stateCounts = computed(() => stateCountsData.value ?? [])
  const mailComposeAccountOptions = computed<CommunicationAccountOption[]>(() =>
    (emailAccountsData.value?.items ?? [])
      .filter(isEmailAccountView)
      .map(emailAccountToComposeOption)
  )
  const sendCapableMailComposeAccountOptions = computed(() =>
    mailComposeAccountOptions.value.filter((option) => option.can_send)
  )
  const defaultMailAccountId = computed(() => {
    const options = sendCapableMailComposeAccountOptions.value
    if (options.length === 0) return ''

    const selectedAccountId = store.selectedMailAccountId.trim()
    if (selectedAccountId && options.some((option) => option.account_id === selectedAccountId)) {
      return selectedAccountId
    }

    const messageAccountId = messageDetail.value?.message.account_id?.trim() ?? store.selectedCommunication?.account_id?.trim() ?? ''
    if (messageAccountId && options.some((option) => option.account_id === messageAccountId)) {
      return messageAccountId
    }

    return options[0]?.account_id ?? ''
  })

  function isProviderFlagMutationAvailable(accountId: string | null | undefined): boolean {
    const normalizedAccountId = accountId?.trim() ?? ''
    if (!normalizedAccountId) return false

    return (emailAccountsData.value?.items ?? []).some((view) =>
      isEmailAccountView(view) &&
      view.account.account_id === normalizedAccountId &&
      view.capabilities.mutate_flags
    )
  }
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
      const status = `${result.updated_count} messages updated`
      store.setMailActionStatus(status)
      notifications.success('Mail action completed', status)
      store.clearMessageSelection()
      await Promise.all([refetchMailList(), refetchStateCounts()])
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Bulk action failed'
      store.setMailActionError(message)
      notifications.error('Mail action failed', message)
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
    handleMarkMessageNotSpam,
    handleMarkMessageSpam,
    handleMarkMessageUnread,
    handleForwardMessage,
    handleGenerateAiReply,
    handleMute,
    handleNewMessage,
    handleRedirectMessage,
    handleRemoveLabel,
    handleRetryAi,
    handleReply,
    handleReplyAll,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSnoozeMessage,
    handleToggleImportant,
    handleToggleStar,
    handleTogglePin,
    handleTranslate
  } = useSelectedMessageActions(store, {
    getDefaultMailAccountId: () => defaultMailAccountId.value,
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

  function notifyMailActionError(message: string) {
    store.setMailActionError(message)
    notifications.error('Mail action failed', message)
  }

  async function handleDeleteDraft(draftId: string) {
    try {
      await deleteDraftMutation.mutateAsync(draftId)
      await refetchDrafts()
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Delete draft failed'
      store.setMailActionError(message)
      notifications.error('Draft delete failed', message)
    }
  }

  const {
    handleComposeFiles,
    handleOpenDraft,
    handleRemoveComposeAttachment,
    handleSaveComposeDraft,
    handleSendCompose
  } = useMailComposeActions(store, {
    getDefaultMailAccountId: () => defaultMailAccountId.value,
    getMailComposeAccountOptions: () => mailComposeAccountOptions.value,
    getSendCapableMailComposeAccountOptions: () => sendCapableMailComposeAccountOptions.value,
    refetchDrafts,
    refetchMailList
  })

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
    handleComposeFiles,
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
    handleRetryAi,
    handleRemoveComposeAttachment,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSaveThreadReplyDraft,
    handleSearchQueryUpdate,
    handleSelectMessage,
    handleSelectThread,
    handleSendThreadReply,
    handleSnoozeMessage,
    handleMarkMessageRead,
    handleMarkMessageNotSpam,
    handleMarkMessageSpam,
    handleMarkMessageUnread,
    handleDeleteFromProvider,
    handleSaveComposeDraft,
    handleSendCompose,
    handleSyncNow,
    handleToggleImportant,
    handleToggleStar,
    handleTogglePin,
    handleTranslate,
    handleExportMessage,
    hasMoreOutboxItems,
    hasRail,
    hasThreadNextPage,
    hasVisibleNextPage,
    isAccountSetupOpen,
    isProviderFlagMutationAvailable,
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
    mailComposeAccountOptions,
    mailboxHealth,
    messageDetail,
    notifyMailActionError,
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

function isEmailAccountView(view: EmailAccountView): boolean {
  return isMailProviderKind(view.account.provider_kind)
}

function emailAccountToComposeOption(view: EmailAccountView): CommunicationAccountOption {
  const account = view.account
  const label = firstNonEmpty(
    account.label,
    account.display_name,
    account.email,
    account.external_account_id,
    account.account_id
  )
  return {
    account_id: account.account_id,
    label,
    provider_kind: account.provider_kind,
    email: firstNonEmpty(account.email, account.external_account_id),
    can_send: view.capabilities.send,
    send_unavailable_reason: view.capabilities.send ? '' : 'Sending is not configured for this account'
  }
}

function firstNonEmpty(...values: Array<string | null | undefined>): string {
  const value = values.find(
    (candidate): candidate is string => typeof candidate === 'string' && candidate.trim().length > 0
  )
  return value?.trim() ?? ''
}
