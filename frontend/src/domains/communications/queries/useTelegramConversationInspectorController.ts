import { computed, ref, type Ref } from 'vue'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import type { TelegramConversationRuntimeAction, TelegramConversationRuntimeActionRunner } from '@/shared/communications/types/telegramRuntimeActions'
import {
  buildTelegramRuntimeActionRequest,
  buildTelegramHistoryPolicyRequest,
  buildTelegramFolderActionExtras,
  buildTelegramFolderReassignExtras,
  buildTelegramMediaDownloadExtras,
  buildTelegramReadReceiptPolicyRequest,
  buildTelegramTopicCloseRequest,
  buildTelegramTopicCreateRequest,
  buildTelegramUnreadCounterPolicyRequest,
  selectedTelegramProviderFolderId,
  telegramConversationCommandId,
} from '../components/messengers/telegramConversationInspectorActions'
import { telegramProviderFolders } from './telegramWorkspacePresentation'
import {
  useCloseTelegramTopicMutation,
  useCreateTelegramTopicMutation,
  useTelegramChatDetailQuery,
  useTelegramChatFoldersQuery,
  useTelegramChatMembersQuery,
  useTelegramAttachmentPreviewQuery,
  useTelegramMediaSearchQuery,
  useTelegramMessageSearchQuery,
  useTelegramPinnedMessagesQuery,
  useTelegramTopicsQuery,
  useTelegramTopicSearchQuery,
  useUpdateTelegramChatHistoryPolicyMutation,
  useUpdateTelegramChatReadReceiptPolicyMutation,
  useUpdateTelegramChatUnreadCounterPolicyMutation,
} from './telegramBusinessQueries'

export function useTelegramConversationInspectorController(
  telegramChat: Ref<TelegramChat>,
  runtimeActionRunner?: TelegramConversationRuntimeActionRunner,
) {
  const chatId = computed(() => telegramChat.value.telegram_chat_id)
  const accountId = computed(() => telegramChat.value.account_id)
  const providerChatId = computed(() => telegramChat.value.provider_chat_id)
  const detailQuery = useTelegramChatDetailQuery(chatId)
  const foldersQuery = useTelegramChatFoldersQuery(accountId)
  const membersQuery = useTelegramChatMembersQuery(chatId, 30)
  const mediaQuery = useTelegramMediaSearchQuery({ accountId, providerChatId, limit: () => 30 })
  const pinnedMessagesQuery = useTelegramPinnedMessagesQuery({ telegramChatId: chatId, limit: () => 30 })
  const topicsQuery = useTelegramTopicsQuery(chatId, 30)
  const topicSearch = ref('')
  const messageSearch = ref('')
  const topicSearchQuery = useTelegramTopicSearchQuery(chatId, topicSearch, 30)
  const messageSearchQuery = useTelegramMessageSearchQuery({ q: messageSearch, accountId, providerChatId, limit: () => 30 })
  const createTopicMutation = useCreateTelegramTopicMutation()
  const closeTopicMutation = useCloseTelegramTopicMutation()
  const historyPolicyMutation = useUpdateTelegramChatHistoryPolicyMutation()
  const readReceiptPolicyMutation = useUpdateTelegramChatReadReceiptPolicyMutation()
  const unreadCounterPolicyMutation = useUpdateTelegramChatUnreadCounterPolicyMutation()
  const newTopicTitle = ref('')
  const providerFolderId = ref<number | null>(null)
  const actionError = ref('')
  const actionStatus = ref('')
  const selectedMediaAttachmentId = ref<string | null>(null)
  const mediaPreviewQuery = useTelegramAttachmentPreviewQuery(selectedMediaAttachmentId)
  const providerFolders = computed(() => telegramProviderFolders(foldersQuery.data.value ?? []))
  type TelegramMediaAttachment = {
    provider_message_id: string
    tdlib_file_id: number | null
    provider_attachment_id?: string | null
    file_name?: string | null
    mime_type?: string | null
  }

  async function runAction(
    action: TelegramConversationRuntimeAction,
    extras: Partial<Parameters<TelegramConversationRuntimeActionRunner>[0]> = {},
  ): Promise<void> {
    if (!runtimeActionRunner) return
    actionError.value = ''
    actionStatus.value = ''
    try {
      await runtimeActionRunner(buildTelegramRuntimeActionRequest(telegramChat.value, action, extras))
      actionStatus.value = `Telegram ${action.replaceAll('_', ' ')} command queued.`
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : 'Telegram provider command failed.'
    }
  }

  function selectedFolderId(): number | undefined {
    return selectedTelegramProviderFolderId(providerFolderId.value)
  }

  function downloadMedia(media: {
    provider_message_id: string
    tdlib_file_id: number | null
    provider_attachment_id?: string | null
    file_name?: string | null
    mime_type?: string | null
  }): void {
    const tdlibFileId = media.tdlib_file_id
    if (tdlibFileId == null) return
    void runAction('download_media', buildTelegramMediaDownloadExtras({ ...media, tdlib_file_id: tdlibFileId }))
  }

  function runFolderAction(action: 'folder_add' | 'folder_remove'): void {
    const folderId = selectedFolderId()
    if (folderId == null) return
    void runAction(action, buildTelegramFolderActionExtras(folderId))
  }

  function reassignFolder(): void {
    const folderId = selectedFolderId()
    if (folderId == null) return
    void runAction('folder_reassign', buildTelegramFolderReassignExtras(folderId))
  }

  async function createTopic(): Promise<void> {
    const title = newTopicTitle.value.trim()
    if (!title) return
    await createTopicMutation.mutateAsync(buildTelegramTopicCreateRequest(
      telegramChat.value,
      title,
      telegramConversationCommandId(),
    ))
    newTopicTitle.value = ''
  }

  async function toggleTopic(topicId: string, isClosed: boolean): Promise<void> {
    await closeTopicMutation.mutateAsync(buildTelegramTopicCloseRequest(
      telegramChat.value,
      topicId,
      isClosed,
      telegramConversationCommandId(),
    ))
  }

  function handleSyncMembers(): void {
    void runAction('sync_members')
  }

  function handleJoinConversation(): void {
    void runAction('join')
  }

  function handleLeaveConversation(): void {
    void runAction('leave')
  }

  function handleCreateTopic(): void {
    void createTopic()
  }

  function handleToggleTopic(topicId: string, isClosed: boolean): void {
    void toggleTopic(topicId, isClosed)
  }

  function handleDownloadMedia(media: TelegramMediaAttachment): void {
    downloadMedia(media)
  }

  function handleAddFolder(): void {
    runFolderAction('folder_add')
  }

  function handleRemoveFolder(): void {
    runFolderAction('folder_remove')
  }

  function handleReplaceFolder(): void {
    reassignFolder()
  }

  async function toggleFullHistory(enabled: boolean): Promise<void> {
    actionError.value = ''
    try {
      await historyPolicyMutation.mutateAsync(buildTelegramHistoryPolicyRequest(telegramChat.value, enabled))
      if (enabled) {
        await runAction('sync_full')
      } else {
        actionStatus.value = 'Full Telegram history is disabled for this chat.'
      }
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : 'Telegram history policy update failed.'
    }
  }

  async function toggleReadReceiptReports(enabled: boolean): Promise<void> {
    actionError.value = ''
    try {
      await readReceiptPolicyMutation.mutateAsync(buildTelegramReadReceiptPolicyRequest(telegramChat.value, enabled))
      actionStatus.value = enabled
        ? 'Telegram read reports are enabled for this chat.'
        : 'Telegram read reports are disabled for this chat. Hermes will keep read state locally.'
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : 'Telegram read receipt policy update failed.'
    }
  }

  async function toggleUnreadCounter(hidden: boolean): Promise<void> {
    actionError.value = ''
    try {
      await unreadCounterPolicyMutation.mutateAsync(buildTelegramUnreadCounterPolicyRequest(telegramChat.value, hidden))
      actionStatus.value = hidden ? 'Unread counter is hidden for this chat.' : 'Unread counter is visible for this chat.'
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : 'Telegram unread counter policy update failed.'
    }
  }

  function handleFullHistoryChange(event: Event): void {
    const input = event.target
    void toggleFullHistory(input instanceof HTMLInputElement ? input.checked : false)
  }

  function handleReadReceiptReportsChange(event: Event): void {
    const input = event.target
    void toggleReadReceiptReports(input instanceof HTMLInputElement ? input.checked : false)
  }

  function handleUnreadCounterChange(event: Event): void {
    const input = event.target
    void toggleUnreadCounter(input instanceof HTMLInputElement ? input.checked : false)
  }

  function handleSetPreviewAttachment(mediaAttachmentId: string): void {
    selectedMediaAttachmentId.value = mediaAttachmentId
  }

  return {
    detailQuery,
    foldersQuery,
    membersQuery,
    mediaQuery,
    pinnedMessagesQuery,
    topicsQuery,
    topicSearch,
    messageSearch,
    topicSearchQuery,
    messageSearchQuery,
    createTopicMutation,
    closeTopicMutation,
    historyPolicyMutation,
    readReceiptPolicyMutation,
    unreadCounterPolicyMutation,
    newTopicTitle,
    providerFolderId,
    actionError,
    actionStatus,
    mediaPreviewQuery,
    providerFolders,
    runAction,
    selectedFolderId,
    downloadMedia,
    handleFullHistoryChange,
    handleReadReceiptReportsChange,
    handleUnreadCounterChange,
    handleSetPreviewAttachment,
    handleSyncMembers,
    handleJoinConversation,
    handleLeaveConversation,
    handleCreateTopic,
    handleToggleTopic,
    handleDownloadMedia,
    handleAddFolder,
    handleRemoveFolder,
    handleReplaceFolder,
    runFolderAction,
    reassignFolder,
    createTopic,
    toggleTopic,
    toggleFullHistory,
    toggleReadReceiptReports,
    toggleUnreadCounter,
  }
}
