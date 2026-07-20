// Historical pre-clean-room workspace runtime. Not part of the active client graph.
import { computed, onScopeDispose, ref, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { getActivePinia } from 'pinia'
import {
  senderEmail,
  senderLabel,
} from '../stores/communications'
import type {
  CommunicationConversationModel,
} from '../components/communicationDomainElements'
import type { MailListItemModel } from '../components/mail/mailElements'
import type { MailInspectorModel } from '../components/mail/mailInspector'
import type { MessengerAttachmentModel, MessengerListItemModel } from '../components/messengers/messengerElements'
import {
  selectMailWorkspaceAction,
} from './communicationMailWorkspaceActions'
import {
  attachmentCount,
  conversationMessage,
  emptyConversation,
  mailSyncStatusIsActive,
  messageTranslation,
} from './communicationWorkspaceMailPresentation'
import { mailItem } from './communicationMailWorkspaceModels'
import {
  routeToAccountId,
  routeToChannelId,
  type PrimaryChannelId,
} from './communicationWorkspaceRoutes'
import {
  handleVisibleMailItemIdsChange as syncVisibleMailSelection,
} from './visibleMailSelection'
import {
  messengerInspectorModel,
  telegramMessengerConversation as buildTelegramMessengerConversation,
  telegramMessengerListItem,
  whatsappMessengerConversation as buildWhatsappMessengerConversation,
  whatsappMessengerListItem,
} from './communicationMessengerWorkspaceModels'
import {
  useTelegramChatsQuery,
  useTelegramMessagesInfiniteQuery,
  useSendTelegramMessageMutation,
} from './telegramBusinessQueries'
import type { TelegramConversationRuntimeAction, TelegramConversationRuntimeActionRunner } from '../../../shared/communications/types/telegramRuntimeActions'
import { useNotificationsStore, type NotificationItem } from '../../../shared/stores/notifications'
import { useToast } from '@/shared/ui'
import { useCommunicationsPageSurface } from './useCommunicationsPageSurface'
import {
  useWhatsappBusinessConversationsQuery,
  useWhatsappBusinessMessagesQuery,
} from './whatsappBusinessQueries'
import { useMarkMessageReadMutation } from './mailActionQueries'
import { useMailImportMutation } from './mailImportQueries'
import { useDelayedMessageRead } from './useDelayedMessageRead'
import { useMessageAiStateQuery } from './mailCoreQueries'
import { buildMailInspector } from './mailInspectorPresentation'
import { prepareMailImport } from '../forms/mailImport'
import { messengerComposerPlainText } from '../components/messengers/messengerComposer'
import { createTelegramInitialHistorySynchronizer } from './telegramInitialHistorySync'
import { createTelegramChatAvatarSynchronizer } from './telegramChatAvatarSync'
import { useTelegramWorkspaceAutoRead } from './useTelegramWorkspaceAutoRead'
import {
  latestTelegramInboundMessageId,
  telegramTdlibMessageId,
} from './telegramWorkspacePresentation'
import {
  telegramAttachmentDownloadExtras,
  type TelegramAttachmentDownloadExtras,
} from './telegramAttachmentDownload'

export function useCommunicationsWorkspaceViewSurface(
  selectedRouteId?: MaybeRefOrGetter<string | undefined>,
  telegramRuntimeActionRunner?: MaybeRefOrGetter<TelegramConversationRuntimeActionRunner | undefined>
) {
  const pageSurface = useCommunicationsPageSurface()
  const notificationsStore = getActivePinia() ? useNotificationsStore() : null
  const toast = useToast()
  const selectedTelegramChatId = ref('')
  const telegramMessagesVisibleForChatId = ref('')
  const selectedTelegramMessageId = ref('')
  const selectedWhatsappProviderChatId = ref('')
  const isTelegramConversationActionRunning = ref(false)
  const isTelegramInitialHistorySyncing = ref(false)
  const isTelegramProviderHistoryLoading = ref(false)
  const exhaustedTelegramHistoryChatKeys = new Set<string>()
  const telegramChatAvatarSynchronizer = createTelegramChatAvatarSynchronizer()
  const activeChannelId = computed<PrimaryChannelId>(
    () => routeToChannelId(toValue(selectedRouteId)) ?? 'mail'
  )
  const selectedRouteAccountId = computed(() =>
    routeToAccountId(toValue(selectedRouteId))
  )
  const selectedTelegramAccountId = computed(() =>
    activeChannelId.value === 'telegram'
      ? selectedRouteAccountId.value ?? undefined
      : undefined
  )
  const selectedWhatsappAccountId = computed(() =>
    activeChannelId.value === 'whatsapp'
      ? selectedRouteAccountId.value ?? null
      : null
  )
  const markMessageReadMutation = useMarkMessageReadMutation()
  const mailImportMutation = useMailImportMutation()
  const selectedMailMessageId = computed(() =>
    activeChannelId.value === 'mail'
      ? pageSurface.store.selectedCommunicationMessageId
      : ''
  )
  const messageAiStateQuery = useMessageAiStateQuery(() =>
    selectedMailMessageId.value || null
  )

  useDelayedMessageRead(selectedMailMessageId, async (messageId) => {
    const message = pageSurface.visibleMailList.value.find(
      (item) => item.message_id === messageId
    )
    if (!message || message.is_read) return
    await markMessageReadMutation.mutateAsync(messageId)
  }, (error) => {
    const message = error instanceof Error ? error.message : 'Provider read-state sync failed'
    pageSurface.store.setMailActionError(message)
  })

  watch(
    selectedRouteAccountId,
    (accountId) => {
      if (activeChannelId.value !== 'mail') return
      pageSurface.store.setSelectedMailAccountId(accountId ?? '')
    },
    { immediate: true }
  )

  watch(
    activeChannelId,
    (channelId) => {
      if (channelId !== 'mail') return
      pageSurface.store.setStateFilter('')
      pageSurface.store.setLocalStateFilter('all')
    },
    { immediate: true }
  )

  watch(
    pageSurface.visibleMailList,
    (messages) => {
      if (
        pageSurface.store.selectedCommunicationMessageId ||
        messages.length === 0
      )
        return
      pageSurface.handleSelectMessage(0)
    },
    { immediate: true }
  )

  watch(
    () => notificationsStore?.pendingNotificationTarget ?? null,
    (notification) => openNotificationTarget(notification),
    { immediate: true }
  )

  const mailItems = computed<MailListItemModel[]>(() =>
    pageSurface.visibleMailList.value.map((message) =>
      mailItem(message, pageSurface.store.selectedCommunicationMessageId)
    )
  )
  const mailSyncStatus = computed(() => {
    const statuses = pageSurface.store.mailSyncStatuses
    const selectedAccountId = pageSurface.store.selectedMailAccountId
    if (selectedAccountId) {
      return (
        statuses.find((status) => status.account_id === selectedAccountId) ??
        null
      )
    }

    return (
      statuses.find((status) => mailSyncStatusIsActive(status.status)) ??
      statuses[0] ??
      null
    )
  })

  const conversation = computed<CommunicationConversationModel>(() => {
    const detail = pageSurface.messageDetail.value?.message ?? null
    const attachments = pageSurface.messageDetail.value?.attachments ?? []
    const summary =
      pageSurface.store.selectedCommunication ??
      pageSurface.visibleMailList.value[0] ??
      null
    const source = detail ?? summary

    if (!source) return emptyConversation()

    return {
      id: source.conversation_id ?? source.message_id,
      channelKind: source.channel_kind,
      title: source.subject || senderLabel(source.sender),
      subtitle: senderEmail(source.sender),
      workflowState: source.workflow_state,
      facts: [
        { label: 'workflow', value: source.workflow_state },
        ...(source.ai_state ? [{ label: 'ai', value: source.ai_state }] : []),
        {
          label: 'attachments',
          value: attachmentCount(source, attachments.length),
        },
        { label: 'importance', value: source.importance_score ?? 'n/a' },
      ],
      messages: [
        conversationMessage(
          source,
          attachments,
          messageTranslation(
            pageSurface.store.mailMessageInsight,
            source.message_id
          ),
          pageSurface.isProviderFlagMutationAvailable(source.account_id)
        ),
      ],
      draftPreview: pageSurface.store.composeForm.body,
    }
  })

  const mailInspector = computed<MailInspectorModel>(() => {
    const detail = pageSurface.messageDetail.value?.message ?? null
    const attachments = pageSurface.messageDetail.value?.attachments ?? []
    const summary =
      pageSurface.store.selectedCommunication ??
      pageSurface.visibleMailList.value[0] ??
      null

    return buildMailInspector(
      detail ?? summary,
      attachments.length,
      messageAiStateQuery.data.value ?? null
    )
  })

  const telegramChatsQuery = useTelegramChatsQuery(
    selectedTelegramAccountId,
    200
  )
  const telegramChats = computed(() => telegramChatsQuery.data.value ?? [])
  const selectedTelegramChat = computed(
    () =>
      telegramChats.value.find(
        (chat) => chat.telegram_chat_id === selectedTelegramChatId.value
      ) ?? null
  )
  const telegramMessagesQuery = useTelegramMessagesInfiniteQuery(
    () => selectedTelegramChat.value?.account_id ?? null,
    () => selectedTelegramChat.value?.provider_chat_id ?? null,
    100
  )
  const telegramMessages = computed(
    () => (telegramMessagesQuery.data.value?.pages.flatMap((page) => page.items) ?? [])
      .sort((left, right) => {
        const leftAt = left.occurred_at ?? left.projected_at
        const rightAt = right.occurred_at ?? right.projected_at
        return leftAt.localeCompare(rightAt) || left.message_id.localeCompare(right.message_id)
      })
  )
  const syncSelectedTelegramHistoryIfNeeded = createTelegramInitialHistorySynchronizer({
    resolveRunner: () => telegramRuntimeActionRunner ? toValue(telegramRuntimeActionRunner) : undefined,
    refetchMessages: () => telegramMessagesQuery.refetch(),
    messageCount: () => telegramMessages.value.length,
    setError: showTelegramActionError,
    setSyncing: (value) => { isTelegramInitialHistorySyncing.value = value },
  })
  const selectedTelegramMessage = computed(
    () => telegramMessages.value.find(
      (message) => message.message_id === selectedTelegramMessageId.value
    ) ?? null
  )
  const sendTelegramMessageMutation = useSendTelegramMessageMutation()
  const telegramMessengerItems = computed(() =>
    telegramChats.value.map((chat) =>
      telegramMessengerListItem(
        chat,
        selectedTelegramChatId.value,
        telegramChatAvatarSynchronizer.sourceFor(chat.telegram_chat_id)
      )
    )
  )
  const telegramMessengerConversation = computed(() =>
    buildTelegramMessengerConversation(
      selectedTelegramChat.value,
      telegramMessages.value,
      selectedTelegramMessageId.value
    )
  )
  const telegramMessengerInspector = computed(() =>
    messengerInspectorModel('telegram', telegramMessengerConversation.value)
  )

  useTelegramWorkspaceAutoRead(
    () => activeChannelId.value === 'telegram' ? selectedTelegramChat.value : null,
    () => telegramMessagesVisibleForChatId.value === selectedTelegramChat.value?.telegram_chat_id,
    () => telegramRuntimeActionRunner ? toValue(telegramRuntimeActionRunner) : undefined,
    async (chat, runner) => {
      if (selectedTelegramChat.value?.telegram_chat_id !== chat.telegram_chat_id) return
      await runTelegramConversationAction('mark_read', runner)
    },
    (error) => {
      showTelegramActionError(
        error instanceof Error ? error.message : 'Telegram read-state sync failed.'
      )
    }
  )

  const whatsappConversationsQuery = useWhatsappBusinessConversationsQuery(
    selectedWhatsappAccountId,
    200
  )
  const whatsappConversations = computed(
    () => whatsappConversationsQuery.data.value ?? []
  )
  const selectedWhatsappConversation = computed(
    () =>
      whatsappConversations.value.find(
        (conversation) =>
          conversation.provider_chat_id === selectedWhatsappProviderChatId.value
      ) ?? null
  )
  const whatsappMessagesQuery = useWhatsappBusinessMessagesQuery(
    () => selectedWhatsappConversation.value?.account_id ?? null,
    () => selectedWhatsappConversation.value?.provider_chat_id ?? null,
    200
  )
  const whatsappMessages = computed(
    () => whatsappMessagesQuery.data.value ?? []
  )
  const whatsappMessengerItems = computed(() =>
    whatsappConversations.value.map((conversation) =>
      whatsappMessengerListItem(
        conversation,
        selectedWhatsappProviderChatId.value
      )
    )
  )
  const whatsappMessengerConversation = computed(() =>
    buildWhatsappMessengerConversation(
      selectedWhatsappConversation.value,
      whatsappMessages.value
    )
  )
  const whatsappMessengerInspector = computed(() =>
    messengerInspectorModel('whatsapp', whatsappMessengerConversation.value)
  )

  watch(
    telegramChats,
    (items) => {
      if (
        items.some(
          (chat) => chat.telegram_chat_id === selectedTelegramChatId.value
        )
      )
        return

      selectedTelegramChatId.value = items[0]?.telegram_chat_id ?? ''
    },
    { immediate: true }
  )

  watch(
    whatsappConversations,
    (items) => {
      if (
        items.some(
          (conversation) =>
            conversation.provider_chat_id ===
            selectedWhatsappProviderChatId.value
        )
      )
        return

      selectedWhatsappProviderChatId.value = items[0]?.provider_chat_id ?? ''
    },
    { immediate: true }
  )
  watch(
    telegramMessages,
    (items) => {
      if (items.some((message) => message.message_id === selectedTelegramMessageId.value)) return
      selectedTelegramMessageId.value = items[0]?.message_id ?? ''
    },
    { immediate: true }
  )
  watch(
    telegramChats,
    (chats) => { void telegramChatAvatarSynchronizer.sync(chats) },
    { immediate: true }
  )
  onScopeDispose(() => telegramChatAvatarSynchronizer.dispose())
  watch(
    () => ({
      activeChannel: activeChannelId.value,
      chat: selectedTelegramChat.value,
      hasFetchedMessages: telegramMessagesQuery.isFetched.value,
      messageCount: telegramMessages.value.length,
    }),
    ({ activeChannel, chat, hasFetchedMessages }) => {
      if (activeChannel !== 'telegram' || !chat || !hasFetchedMessages) return
      void syncSelectedTelegramHistoryIfNeeded(chat)
    },
    { immediate: true }
  )
  return {
    activeChannelId,
    conversation,
    isMailActionRunning: computed(() => pageSurface.store.isMailActionRunning),
    isMailImporting: computed(() => mailImportMutation.isPending.value),
    mailActionError: computed(() => pageSurface.store.mailActionError),
    mailActionStatus: computed(() => pageSurface.store.mailActionStatus),
    mailInspector,
    mailItems,
    mailSyncStatus,
    pageSurface,
    refreshMail,
    selectMailAction,
    handleVisibleMailItemIdsChange,
    importMailFile,
    selectMailMessage,
    selectTelegramConversation,
    markTelegramMessagesVisible,
    selectTelegramMessage,
    submitTelegramMessage,
    runTelegramConversationAction,
    downloadTelegramAttachment,
    isTelegramActionRunning: computed(() =>
      sendTelegramMessageMutation.isPending.value
      || isTelegramConversationActionRunning.value
      || isTelegramInitialHistorySyncing.value
    ),
    isTelegramListLoading: computed(() => telegramChatsQuery.isLoading.value),
    isTelegramListRefreshing: computed(() => telegramChatsQuery.isFetching.value),
    telegramListError: computed(() => telegramChatsQuery.error.value?.message ?? ''),
    refreshTelegramConversations,
    loadOlderTelegramMessages,
    isTelegramLoadingOlder: computed(() =>
      telegramMessagesQuery.isFetchingNextPage.value || isTelegramProviderHistoryLoading.value
    ),
    selectWhatsappConversation,
    telegramMessengerConversation,
    telegramMessengerInspector,
    telegramMessengerItems,
    selectedTelegramChat,
    selectedTelegramMessage,
    whatsappMessengerConversation,
    whatsappMessengerInspector,
    whatsappMessengerItems,
    title: 'Communications',
    description:
      'Unified evidence-first workspace for mail, messenger channels, provider commands and review pressure.',
  }

  function selectTelegramConversation(item: MessengerListItemModel): void {
    selectedTelegramChatId.value = item.id
    telegramMessagesVisibleForChatId.value = ''
  }

  function markTelegramMessagesVisible(): void {
    const chat = selectedTelegramChat.value
    if (chat && telegramMessages.value.length > 0) {
      telegramMessagesVisibleForChatId.value = chat.telegram_chat_id
    }
  }

  async function refreshTelegramConversations(): Promise<void> {
    await telegramChatsQuery.refetch()
    if (selectedTelegramChat.value) {
      await telegramMessagesQuery.refetch()
      exhaustedTelegramHistoryChatKeys.delete(
        `${selectedTelegramChat.value.account_id}:${selectedTelegramChat.value.provider_chat_id}`
      )
    }
  }

  async function loadOlderTelegramMessages(): Promise<void> {
    const chat = selectedTelegramChat.value
    if (!chat || telegramMessagesQuery.isFetchingNextPage.value) return
    const chatKey = `${chat.account_id}:${chat.provider_chat_id}`
    if (exhaustedTelegramHistoryChatKeys.has(chatKey)) return

    if (telegramMessagesQuery.hasNextPage.value) {
      await telegramMessagesQuery.fetchNextPage()
      return
    }

    const runner = telegramRuntimeActionRunner ? toValue(telegramRuntimeActionRunner) : undefined
    const oldestMessage = telegramMessages.value[0]
    const previousMessageIds = new Set(telegramMessages.value.map((message) => message.message_id))
    const oldestProviderMessageId = telegramTdlibMessageId(oldestMessage?.provider_message_id)
    if (!runner || !oldestProviderMessageId) return

    isTelegramProviderHistoryLoading.value = true
    try {
      await runTelegramConversationAction(
        'sync_older',
        runner,
        undefined,
        undefined,
        { historyFromMessageId: oldestProviderMessageId }
      )
      await telegramMessagesQuery.refetch()
      if (telegramMessagesQuery.hasNextPage.value) {
        await telegramMessagesQuery.fetchNextPage()
      }
      const loadedNewMessage = telegramMessages.value.some(
        (message) => !previousMessageIds.has(message.message_id)
      )
      if (!loadedNewMessage && !telegramMessagesQuery.hasNextPage.value) {
        exhaustedTelegramHistoryChatKeys.add(chatKey)
      }
    } finally {
      isTelegramProviderHistoryLoading.value = false
    }
  }

  function selectTelegramMessage(messageId: string): void {
    selectedTelegramMessageId.value = messageId
  }

  async function submitTelegramMessage(value: string): Promise<void> {
    const chat = selectedTelegramChat.value
    const text = messengerComposerPlainText(value)
    if (!chat || !text) return

    try {
      await sendTelegramMessageMutation.mutateAsync({
        account_id: chat.account_id,
        provider_chat_id: chat.provider_chat_id,
        text,
      })
    } catch (error) {
      showTelegramActionError(
        error instanceof Error ? error.message : 'Telegram message send failed.'
      )
    }
  }

  async function runTelegramConversationAction(
    action: TelegramConversationRuntimeAction,
    runner: TelegramConversationRuntimeActionRunner | undefined,
    file?: File, caption?: string,
    extras: TelegramAttachmentDownloadExtras & { historyFromMessageId?: number } = {},
  ): Promise<void> {
    const chat = selectedTelegramChat.value
    if (!chat || !runner) return

    isTelegramConversationActionRunning.value = true
    try {
      await runner({
        action,
        accountId: chat.account_id,
        providerChatId: chat.provider_chat_id,
        telegramChatId: chat.telegram_chat_id,
        lastReadInboxProviderMessageId: latestTelegramInboundMessageId(telegramMessages.value),
        file,
        caption,
        ...extras,
      })
    } catch (error) {
      showTelegramActionError(
        error instanceof Error ? error.message : 'Telegram provider command failed.'
      )
    } finally {
      isTelegramConversationActionRunning.value = false
    }
  }

  async function downloadTelegramAttachment(attachment: MessengerAttachmentModel, runner: TelegramConversationRuntimeActionRunner | undefined): Promise<void> {
    const extras = telegramAttachmentDownloadExtras(attachment)
    if (!extras) return void showTelegramActionError(
      'This Telegram attachment cannot be downloaded from the provider.'
    )
    await runTelegramConversationAction('download_media', runner, undefined, undefined, extras)
  }

  function showTelegramActionError(message: string): void {
    if (!message) return
    toast.error('Telegram action failed', message)
  }

  function selectWhatsappConversation(item: MessengerListItemModel): void {
    selectedWhatsappProviderChatId.value = item.id
  }

  function selectMailMessage(item: MailListItemModel): void {
    const messageIndex = pageSurface.visibleMailList.value.findIndex(
      (message) => message.message_id === item.id
    )
    if (messageIndex < 0) return
    pageSurface.handleSelectMessage(messageIndex)
  }

  function handleVisibleMailItemIdsChange(itemIds: string[]): void {
    syncVisibleMailSelection(pageSurface, itemIds)
  }

  async function importMailFile(file: File): Promise<void> {
    const accountId = pageSurface.store.selectedMailAccountId.trim()
    if (!accountId) {
      pageSurface.store.setMailActionError('Select a mailbox before importing mail.')
      return
    }

    try {
      const request = await prepareMailImport(file)
      const result = await mailImportMutation.mutateAsync({ accountId, ...request })
      await pageSurface.refetchMailList()
      const reasons = [...new Set(result.failures.map((failure) => failure.reason))]
      const skipped = result.failed_count > 0
        ? ` ${result.failed_count} skipped${reasons.length ? ` (${reasons.join(', ')})` : ''}.`
        : ''
      pageSurface.store.setMailActionStatus(
        `Imported ${result.imported_count} ${result.imported_count === 1 ? 'message' : 'messages'} locally.${skipped}`
      )
    } catch (error) {
      pageSurface.store.setMailActionError(
        error instanceof Error ? error.message : 'Mail import failed.'
      )
    }
  }

  function openNotificationTarget(notification: NotificationItem | null): void {
    if (notification?.targetView !== 'communications-mail') return

    if (notification.targetId) {
      pageSurface.store.selectMessageId(notification.targetId)
      pageSurface.store.setActiveMessageContextTab('message')
      pageSurface.store.setCommunicationMessageInsight(null)
    }

    notificationsStore?.consumePendingNotificationTarget()
  }

  function refreshMail(): void {
    void pageSurface.refetchMailList()
  }

  async function selectMailAction(actionId: string): Promise<void> {
    await selectMailWorkspaceAction(pageSurface, actionId)
  }
}

export { mailInspectorSummary } from './mailInspectorPresentation'
