<script setup lang="ts">
import { computed, watch } from 'vue'
import { useTelegramSendActions } from '../queries/useTelegramSendActions'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import TelegramActionRail from '../components/TelegramActionRail.vue'
import TelegramChatList from '../components/TelegramChatList.vue'
import TelegramCommandHeader from '../components/TelegramCommandHeader.vue'
import TelegramMessageThread from '../components/TelegramMessageThread.vue'
import TelegramRail from '../components/TelegramRail.vue'
import TelegramStatusMessages from '../components/TelegramStatusMessages.vue'
import { filterTelegramChats, filterTelegramChatsByGroup, telegramChatFilterCounts, telegramFilterTabs, telegramMessageTime, telegramMessagesChronological } from '../stores/telegram'
import { useTelegramStore } from '../stores/telegram'
import { telegramMediaSearchSourceLabel } from '../stores/telegramMediaSearch'
import { formatTelegramDateTime, isTelegramChatArchived, isTelegramChatMuted, isTelegramChatPinned, openTelegramSearchChatInThread, openTelegramSearchMediaInThread, openTelegramSearchMessageInThread, runTelegramChatReadToggleAction, runTelegramForwardMessageAction, runTelegramChatToggleAction } from './dialogActionHelpers'
import type { TelegramAttachmentHint, TelegramChat, TelegramMediaItem, TelegramMessage } from '../types/telegram'
import {
  telegramQueryKeys,
  useAddTelegramReactionMutation,
  useArchiveTelegramChatMutation,
  useDeleteTelegramMessageMutation,
  useDownloadTelegramMediaMutation,
  useEditTelegramMessageMutation,
  useForwardTelegramMessageMutation,
  useMarkReadTelegramChatMutation,
  useMarkUnreadTelegramChatMutation,
  useMuteTelegramChatMutation,
  usePinTelegramChatMutation,
  usePinTelegramMessageMutation,
  useRemoveTelegramReactionMutation,
  useRestoreTelegramMessageMutation,
  useSyncTelegramChatsMutation,
  useSyncTelegramHistoryMutation,
  useTelegramAccountsQuery,
  useTelegramAccountCapabilitiesQuery,
  useTelegramCapabilitiesQuery,
  useTelegramChatDetailQuery,
  useTelegramChatMembersQuery,
  useTelegramChatsQuery,
  useTelegramMessagesQuery,
  useUnarchiveTelegramChatMutation,
  useUnmuteTelegramChatMutation,
  useUnpinTelegramChatMutation
} from '../queries/useTelegramQuery'
import { useRestartTelegramRuntimeMutation, useStartTelegramRuntimeMutation, useStopTelegramRuntimeMutation, useTelegramRuntimeStatusQuery } from '../queries/useTelegramRuntimeQuery'
import { useTelegramFolderFilters } from '../queries/useTelegramFolderFilters'
import { useTelegramDialogSearchQuery, useTelegramMediaSearchQuery, useTelegramMessageSearchQuery } from '../queries/useTelegramSearchQuery'
import { telegramOldestTdlibMessageId } from '../api/telegramWorkspace'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
const { t } = useI18n()
const store = useTelegramStore()
const realtimeStatus = useRealtimeStatusStore()
const queryClient = useQueryClient()
const { data: globalCapabilities } = useTelegramCapabilitiesQuery()
const { data: accounts } = useTelegramAccountsQuery()
const chatsQuery = useTelegramChatsQuery(undefined, 500)
const recentMessagesQuery = useTelegramMessagesQuery(undefined, undefined, 200)
const telegramChats = computed(() => chatsQuery.data.value ?? [])
const telegramPreviewMessages = computed(() => recentMessagesQuery.data.value ?? [])
const selectedTelegramChat = computed<TelegramChat | null>(() => {
  const currentChatId = store.selectedTelegramChatId
  return telegramChats.value.find((chat) => chat.provider_chat_id === currentChatId) ??
    telegramChats.value[0] ??
    null
})
const selectedChatAccountId = computed(() => selectedTelegramChat.value?.account_id ?? null)
const selectedChatProviderId = computed(() => selectedTelegramChat.value?.provider_chat_id)
const selectedChatTelegramId = computed(() => selectedTelegramChat.value?.telegram_chat_id ?? null)
const selectedMessagesQuery = useTelegramMessagesQuery(
  selectedChatAccountId,
  computed(() => selectedChatProviderId.value ?? null),
  100
)
const selectedChatDetailQuery = useTelegramChatDetailQuery(selectedChatTelegramId)
const selectedChatMembersQuery = useTelegramChatMembersQuery(selectedChatTelegramId, 50)
const { data: selectedRuntimeStatus } = useTelegramRuntimeStatusQuery(selectedChatAccountId)
const selectedAccountId = computed(() => selectedTelegramChat.value?.account_id ?? accounts.value?.[0]?.account_id ?? null)
const { data: accountCapabilities } = useTelegramAccountCapabilitiesQuery(selectedAccountId)
const capabilities = computed(() => accountCapabilities.value ?? globalCapabilities.value ?? null)
const workspaceSearchQuery = computed(() => store.telegramSearchQuery.trim())
const telegramDialogSearchQuery = useTelegramDialogSearchQuery({ q: workspaceSearchQuery, accountId: selectedAccountId, limit: 20 })
const telegramMessageSearchQuery = useTelegramMessageSearchQuery({ q: workspaceSearchQuery, accountId: selectedAccountId, limit: 50 })
const telegramMediaSearchQuery = useTelegramMediaSearchQuery({
  q: workspaceSearchQuery, accountId: selectedChatAccountId, providerChatId: computed(() => selectedChatProviderId.value ?? null), limit: 100,
})
const selectedTelegramMessages = computed<TelegramMessage[]>(() =>
  telegramMessagesChronological(selectedMessagesQuery.data.value ?? [])
)
const telegramDialogSearchResults = computed(() => telegramDialogSearchQuery.data.value?.items ?? [])
const telegramSearchResults = computed(() => telegramMessageSearchQuery.data.value?.items ?? [])
const telegramSearchTotal = computed(() => telegramMessageSearchQuery.data.value?.total ?? 0)
const telegramMediaGalleryItems = computed(() => telegramMediaSearchQuery.data.value?.items ?? [])
const mediaSearchSourceLabel = computed(() => telegramMediaSearchSourceLabel(telegramMediaSearchQuery.data.value))
const isWorkspaceSearchLoading = computed(
  () =>
    telegramDialogSearchQuery.isLoading.value ||
    telegramDialogSearchQuery.isFetching.value ||
    telegramMessageSearchQuery.isLoading.value ||
    telegramMessageSearchQuery.isFetching.value ||
    telegramMediaSearchQuery.isLoading.value ||
    telegramMediaSearchQuery.isFetching.value
)
const filteredTelegramChats = computed(() =>
  filterTelegramChats(
    filterTelegramChatsByGroup(telegramChats.value, store.activeTelegramGroupFilter),
    telegramPreviewMessages.value,
    store.telegramSearchQuery,
    store.activeTelegramFilter
  )
)
const chatFilterCounts = computed(() =>
  telegramChatFilterCounts(telegramChats.value, telegramPreviewMessages.value)
)
const { groupFilters: chatGroupFilters } = useTelegramFolderFilters(
  telegramChats,
  selectedAccountId,
  computed(() => store.activeTelegramGroupFilter),
  (fallbackFilterId) => {
    store.activeTelegramGroupFilter = fallbackFilterId
  }
)
const isTelegramLoading = computed(() =>
  chatsQuery.isLoading.value || recentMessagesQuery.isLoading.value || selectedMessagesQuery.isLoading.value
)
watch(
  telegramChats,
  (chats) => {
    if (!chats.length) {
      store.selectedTelegramChatId = ''
      return
    }
    if (!chats.some((chat) => chat.provider_chat_id === store.selectedTelegramChatId)) {
      store.selectedTelegramChatId = chats[0]?.provider_chat_id ?? ''
    }
  },
  { immediate: true }
)
const syncChatsMutation = useSyncTelegramChatsMutation()
const syncHistoryMutation = useSyncTelegramHistoryMutation()
const { replyTo, sendOrReply, uploadMedia } = useTelegramSendActions(
  () => selectedTelegramChat.value,
  () => store.isTelegramBusy,
  () => store.telegramManualSendText,
  {
    setActionSubmitting: store.setTelegramActionSubmitting,
    setActionMessage: store.setTelegramActionMessage,
    setError: store.setTelegramError,
    resetSendForm: store.resetSendForm,
    setSelectedChatId: (id) => { store.selectedTelegramChatId = id },
  }
)
const startRuntimeMutation = useStartTelegramRuntimeMutation()
const stopRuntimeMutation = useStopTelegramRuntimeMutation()
const restartRuntimeMutation = useRestartTelegramRuntimeMutation()
const editMessageMutation = useEditTelegramMessageMutation()
const forwardMessageMutation = useForwardTelegramMessageMutation()
const deleteMessageMutation = useDeleteTelegramMessageMutation()
const restoreMessageMutation = useRestoreTelegramMessageMutation()
const pinMessageMutation = usePinTelegramMessageMutation()
const addReactionMutation = useAddTelegramReactionMutation()
const removeReactionMutation = useRemoveTelegramReactionMutation()
const downloadMediaMutation = useDownloadTelegramMediaMutation()
const pinChatMutation = usePinTelegramChatMutation()
const unpinChatMutation = useUnpinTelegramChatMutation()
const archiveChatMutation = useArchiveTelegramChatMutation()
const unarchiveChatMutation = useUnarchiveTelegramChatMutation()
const muteChatMutation = useMuteTelegramChatMutation()
const unmuteChatMutation = useUnmuteTelegramChatMutation()
const markReadChatMutation = useMarkReadTelegramChatMutation()
const markUnreadChatMutation = useMarkUnreadTelegramChatMutation()
const runtimeLabel = computed(() => {
  if (!selectedRuntimeStatus.value) {
    return capabilities.value?.account_scope?.runtime_kind ?? capabilities.value?.runtime_mode ?? t('Runtime Status')
  }
  return selectedRuntimeStatus.value.status
})
const filterTabs = computed(() => telegramFilterTabs(t))

function selectTelegramChat(chat: TelegramChat) {
  store.clearTelegramFocusedMessage()
  store.selectTelegramChat(chat)
}
const telegramAutoHistorySyncKeys = new Set<string>()
watch(
  selectedTelegramChat,
  (chat) => {
    void autoSyncTelegramHistory(chat)
  },
  { immediate: true }
)
async function autoSyncTelegramHistory(chat: TelegramChat | null) {
  if (!chat || store.isTelegramBusy) return
  const syncKey = `${chat.account_id}:${chat.provider_chat_id}`
  if (telegramAutoHistorySyncKeys.has(syncKey)) return
  if (chat.chat_kind !== 'private' && hasProjectedMessagesForChat(chat)) return

  telegramAutoHistorySyncKeys.add(syncKey)
  store.setTelegramHistorySyncing(true)
  store.setTelegramError('')
  try {
    const result = await syncHistoryMutation.mutateAsync({
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      mode: chat.chat_kind === 'private' ? 'full' : 'latest',
      limit: 100
    })
    store.setTelegramActionMessage(`Telegram history synced: ${result.synced_count}`)
  } catch (err) {
    telegramAutoHistorySyncKeys.delete(syncKey)
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramHistorySyncing(false)
  }
}
function hasProjectedMessagesForChat(chat: TelegramChat): boolean {
  return selectedTelegramMessages.value.some(
    (message) =>
      message.account_id === chat.account_id &&
      message.provider_chat_id === chat.provider_chat_id
  )
}
async function setTelegramRuntime(action: 'start' | 'stop' | 'restart') {
  if (store.isTelegramBusy || !selectedAccountId.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const mutation = action === 'start' ? startRuntimeMutation : action === 'stop' ? stopRuntimeMutation : restartRuntimeMutation
    const status = await mutation.mutateAsync({ account_id: selectedAccountId.value })
    store.setTelegramActionMessage(`Telegram runtime ${status.status}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function syncTelegramChats() {
  if (store.isTelegramBusy || !selectedAccountId.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await syncChatsMutation.mutateAsync({ account_id: selectedAccountId.value })
    store.setTelegramActionMessage(`Telegram chats synced: ${result.synced_count}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function syncSelectedTelegramHistory() {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await syncHistoryMutation.mutateAsync({
      account_id: selectedTelegramChat.value.account_id,
      provider_chat_id: selectedTelegramChat.value.provider_chat_id,
      mode: selectedTelegramChat.value.chat_kind === 'private' ? 'full' : 'latest',
      limit: 100
    })
    store.selectedTelegramChatId = result.provider_chat_id
    store.setTelegramActionMessage(`Telegram history synced: ${result.synced_count}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function syncOlderTelegramHistory() {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  const fromMessageId = telegramOldestTdlibMessageId(selectedTelegramMessages.value)
  if (fromMessageId === null) {
    await autoSyncTelegramHistory(selectedTelegramChat.value)
    return
  }
  store.setTelegramHistorySyncing(true)
  store.setTelegramError('')
  try {
    const result = await syncHistoryMutation.mutateAsync({
      account_id: selectedTelegramChat.value.account_id,
      provider_chat_id: selectedTelegramChat.value.provider_chat_id,
      from_message_id: fromMessageId,
      mode: 'older',
      limit: 100
    })
    const baseMessage = `Telegram history synced: ${result.synced_count}`
    store.setTelegramActionMessage(
      result.has_more ? baseMessage : `${baseMessage}; ${t('no older Telegram messages')}`
    )
  } finally {
    store.setTelegramHistorySyncing(false)
  }
}
async function sendTelegramManualMessage() {
  await sendOrReply()
}
async function editTelegramMessage(message: TelegramMessage) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await editMessageMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: message.provider_chat_id ?? selectedTelegramChat.value.provider_chat_id,
      provider_message_id: message.provider_message_id,
      new_text: message.text
    })
    store.setTelegramActionMessage(`Edit recorded: version ${result.version_number ?? '?'}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function deleteTelegramMessage(message: TelegramMessage) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    await deleteMessageMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: message.provider_chat_id ?? selectedTelegramChat.value.provider_chat_id,
      provider_message_id: message.provider_message_id
    })
    store.setTelegramActionMessage('Message tombstone recorded')
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function restoreTelegramMessage(message: TelegramMessage) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    await restoreMessageMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: message.provider_chat_id ?? selectedTelegramChat.value.provider_chat_id,
      provider_message_id: message.provider_message_id
    })
    store.setTelegramActionMessage('Message visibility restored')
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function togglePinnedTelegramMessage(message: TelegramMessage) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const isPinned = Boolean(message.metadata?.is_pinned ?? message.metadata?.pinned)
    const result = await pinMessageMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: message.provider_chat_id ?? selectedTelegramChat.value.provider_chat_id,
      provider_message_id: message.provider_message_id,
      is_pinned: !isPinned
    })
    store.setTelegramActionMessage(result.status === 'pinned' ? 'Message pinned' : 'Message unpinned')
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function forwardTelegramMessage(message: TelegramMessage) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  await runTelegramForwardMessageAction({
    chat: selectedTelegramChat.value, message, mutation: forwardMessageMutation,
    sourceChatUnavailableMessage: t('Telegram source chat is not available for forwarding.'),
    setSubmitting: store.setTelegramActionSubmitting, setActionMessage: store.setTelegramActionMessage, setError: store.setTelegramError,
    setSelectedChatId: (id) => { store.selectedTelegramChatId = id },
  })
}
async function addReactionToMessage(payload: { message: TelegramMessage; emoji: string }) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    await addReactionMutation.mutateAsync({
      messageId: payload.message.message_id,
      request: {
        account_id: payload.message.account_id,
        provider_chat_id: payload.message.provider_chat_id ?? selectedTelegramChat.value.provider_chat_id,
        provider_message_id: payload.message.provider_message_id,
        reaction_emoji: payload.emoji,
        sender_id: 'hermes-owner',
        sender_display_name: 'Owner'
      }
    })
    store.setTelegramActionMessage(`Reaction added: ${payload.emoji}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function removeReactionFromMessage(payload: { message: TelegramMessage; emoji: string }) {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    await removeReactionMutation.mutateAsync({
      messageId: payload.message.message_id,
      request: {
        account_id: payload.message.account_id,
        provider_chat_id: payload.message.provider_chat_id ?? selectedTelegramChat.value.provider_chat_id,
        provider_message_id: payload.message.provider_message_id,
        reaction_emoji: payload.emoji,
        sender_id: 'hermes-owner',
        sender_display_name: 'Owner'
      }
    })
    store.setTelegramActionMessage(`Reaction removed: ${payload.emoji}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function downloadTelegramMedia(attachment: TelegramAttachmentHint, message?: TelegramMessage) {
  if (store.isTelegramBusy) return
  if (!selectedTelegramChat.value) {
    store.setTelegramError(t('Select a Telegram chat before downloading media.'))
    return
  }
  if (attachment.tdlibFileId === null) {
    store.setTelegramError(t('Telegram attachment does not include TDLib file metadata.'))
    return
  }
  const sourceMessage =
    message ??
    selectedTelegramMessages.value.find(
      (item) => item.message_id === attachment.messageId
    ) ??
    null
  const providerMessageId = sourceMessage?.provider_message_id ?? attachment.providerMessageId ?? null
  if (!providerMessageId) {
    store.setTelegramError(t('Telegram source message is not available for this attachment.'))
    return
  }
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await downloadMediaMutation.mutateAsync({
      account_id: sourceMessage?.account_id ?? selectedTelegramChat.value.account_id,
      provider_chat_id: sourceMessage?.provider_chat_id || selectedTelegramChat.value.provider_chat_id,
      provider_message_id: providerMessageId,
      tdlib_file_id: attachment.tdlibFileId,
      provider_attachment_id: attachment.providerAttachmentId,
      filename: attachment.fileName,
      content_type: attachment.mimeType ?? undefined,
      priority: 16
    })
    store.setTelegramActionMessage(`Telegram media download started: ${result.tdlib_file_id}`)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}
async function togglePinnedTelegramChat() {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  const chat = selectedTelegramChat.value
  await runTelegramChatToggleAction({
    chat,
    isActive: isTelegramChatPinned(chat),
    activateMutation: pinChatMutation,
    deactivateMutation: unpinChatMutation,
    activateMessage: 'Chat pinned locally',
    deactivateMessage: 'Chat unpinned locally',
    setSubmitting: store.setTelegramActionSubmitting,
    setActionMessage: store.setTelegramActionMessage,
    setError: store.setTelegramError,
  })
}
async function toggleArchivedTelegramChat() {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  const chat = selectedTelegramChat.value
  await runTelegramChatToggleAction({
    chat,
    isActive: isTelegramChatArchived(chat),
    activateMutation: archiveChatMutation,
    deactivateMutation: unarchiveChatMutation,
    activateMessage: 'Chat archived locally',
    deactivateMessage: 'Chat restored from archive locally',
    setSubmitting: store.setTelegramActionSubmitting,
    setActionMessage: store.setTelegramActionMessage,
    setError: store.setTelegramError,
  })
}
async function toggleMutedTelegramChat() {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  const chat = selectedTelegramChat.value
  await runTelegramChatToggleAction({
    chat,
    isActive: isTelegramChatMuted(chat),
    activateMutation: muteChatMutation,
    deactivateMutation: unmuteChatMutation,
    activateMessage: 'Chat muted locally',
    deactivateMessage: 'Chat unmuted locally',
    setSubmitting: store.setTelegramActionSubmitting,
    setActionMessage: store.setTelegramActionMessage,
    setError: store.setTelegramError,
  })
}
async function toggleReadTelegramChat() {
  if (store.isTelegramBusy || !selectedTelegramChat.value) return
  await runTelegramChatReadToggleAction(
    selectedTelegramChat.value,
    markReadChatMutation,
    markUnreadChatMutation,
    store.setTelegramActionSubmitting,
    store.setTelegramActionMessage,
    store.setTelegramError,
  )
}
const searchNavigationCallbacks = {
  setError: (value: string) => store.setTelegramError(value),
  selectChat: (chat: TelegramChat) => store.selectTelegramChat(chat),
  focusMessage: (message: TelegramMessage) => store.focusTelegramMessage(message),
  clearFocusedMessage: () => store.clearTelegramFocusedMessage(),
  setActiveThreadTab: (tab: 'messages') => { store.activeThreadTab = tab },
  setSearchQuery: (value: string) => {
    store.telegramSearchQuery = value
  }
}
function openTelegramNewMessage() {
  store.closeTelegramMenus()
  store.clearTelegramFocusedMessage()
  store.activeThreadTab = 'messages'
  if (!selectedTelegramChat.value) {
    store.setTelegramActionMessage(t('Select a synced Telegram chat before composing.'))
    return
  }
  store.resetSendForm()
  store.setTelegramActionMessage(`${t('Manual send target')}: ${selectedTelegramChat.value.title}`)
}
async function runTelegramQuickAction(action: 'create_note' | 'create_task' | 'create_contact' | 'create_document') { store.closeTelegramMenus() }
function openTelegramAccountSetup() {
  store.closeTelegramMenus()
  store.openTelegramInspector('about')
}
</script>
<template>
  <section class="telegram-page communications-page">
    <TelegramCommandHeader
      :runtimeLabel="runtimeLabel"
      :searchQuery="store.telegramSearchQuery"
      :filterTabs="filterTabs"
      :filterCounts="chatFilterCounts"
      :activeFilter="store.activeTelegramFilter"
      :isFiltersMenuOpen="store.isTelegramFiltersMenuOpen"
      :isNewMenuOpen="store.isTelegramNewMenuOpen"
      :isTelegramBusy="store.isTelegramBusy"
      :selectedTelegramChat="selectedTelegramChat"
      @update:searchQuery="store.telegramSearchQuery = $event"
      @toggleFiltersMenu="store.toggleTelegramFiltersMenu()"
      @toggleNewMenu="store.toggleTelegramNewMenu()"
      @selectFilter="store.selectTelegramFilter($event)"
      @syncChats="store.closeTelegramMenus(); void syncTelegramChats()"
      @addAccount="openTelegramAccountSetup"
      @newMessage="openTelegramNewMessage"
      @quickAction="(action) => void runTelegramQuickAction(action)"
    />
    <TelegramActionRail
      :groupFilters="chatGroupFilters"
      :activeGroupFilter="store.activeTelegramGroupFilter"
      :isTelegramBusy="store.isTelegramBusy"
      :hasSelectedTelegramChat="Boolean(selectedTelegramChat)"
      :isInspectorOpen="store.isTelegramInspectorOpen"
      @syncChats="void syncTelegramChats()"
      @syncHistory="void syncSelectedTelegramHistory()"
      @startRuntime="void setTelegramRuntime('start')"
      @stopRuntime="void setTelegramRuntime('stop')"
      @restartRuntime="void setTelegramRuntime('restart')"
      @selectGroupFilter="store.selectTelegramGroupFilter($event)"
      @toggleInspector="store.toggleTelegramInspector()"
    />
    <TelegramStatusMessages :actionMessage="store.telegramActionMessage" :error="store.telegramError" :realtimeStatusLabel="realtimeStatus.realtimeStatusLabel" :realtimeStatusDetail="realtimeStatus.realtimeStatusDetail" :realtimeStatusTone="realtimeStatus.realtimeStatusTone" />
    <div
      class="three-pane communications-grid telegram-grid"
      :class="{ 'inspector-open': store.isTelegramInspectorOpen }"
    >
      <TelegramChatList
        :telegramChats="filteredTelegramChats"
        :telegramMessages="telegramPreviewMessages"
        :selectedTelegramChatId="selectedTelegramChat?.provider_chat_id ?? ''"
        :isTelegramLoading="isTelegramLoading"
        :formatDateTime="formatTelegramDateTime"
        @selectChat="selectTelegramChat"
      />
      <TelegramMessageThread
        :selectedTelegramChat="selectedTelegramChat"
        :selectedTelegramMessages="selectedTelegramMessages"
        :aiAnalysisResult="null"
        :selectedCommunication="null"
        :isTelegramLoading="isTelegramLoading"
        :isTelegramActionSubmitting="store.isTelegramBusy"
        :activeThreadTab="store.activeThreadTab"
        :telegramMessageTime="telegramMessageTime"
        :telegramManualSendText="store.telegramManualSendText"
        :selectedTelegramRuntimeStatus="selectedRuntimeStatus ?? null"
        :capabilities="capabilities"
        :workspaceSearchQuery="workspaceSearchQuery"
        :searchChats="telegramDialogSearchResults"
        :searchResults="telegramSearchResults"
        :searchResultTotal="telegramSearchTotal"
        :mediaGalleryItems="telegramMediaGalleryItems"
        :mediaSearchSourceLabel="mediaSearchSourceLabel"
        :isWorkspaceSearchLoading="isWorkspaceSearchLoading"
        :focusedTelegramMessage="store.focusedTelegramMessage"
        :replyTo="replyTo"
        @update:activeThreadTab="store.activeThreadTab = $event"
        @update:telegramManualSendText="store.setTelegramManualSendText($event)"
        @railTabChange="(tab) => store.openTelegramInspector(tab)"
        @loadWorkspace="void queryClient.invalidateQueries({ queryKey: telegramQueryKeys.messages })"
        @syncHistory="void syncSelectedTelegramHistory()"
        @syncOlderHistory="void syncOlderTelegramHistory()"
        @sendMessage="void sendTelegramManualMessage()"
        @uploadMedia="(file) => void uploadMedia(file)"
        @downloadMedia="(attachment, message) => void downloadTelegramMedia(attachment, message)"
        @editMessage="(message) => void editTelegramMessage(message)"
        @deleteMessage="(message) => void deleteTelegramMessage(message)"
        @restoreMessage="(message) => void restoreTelegramMessage(message)"
        @forwardMessage="(message) => void forwardTelegramMessage(message)"
        @togglePinMessage="(message) => void togglePinnedTelegramMessage(message)"
        @addReaction="(payload) => void addReactionToMessage(payload)"
        @removeReaction="(payload) => void removeReactionFromMessage(payload)"
        @openSearchChat="(chat) => openTelegramSearchChatInThread(chat, searchNavigationCallbacks)"
        @openSearchMessage="(message) => openTelegramSearchMessageInThread(telegramChats, message, searchNavigationCallbacks)"
        @openSearchMedia="(item) => openTelegramSearchMediaInThread({
          item,
          currentChat: selectedTelegramChat,
          providerKind: accounts?.find((account) => account.account_id === selectedTelegramChat?.account_id)?.provider_kind ?? 'telegram_user',
          callbacks: searchNavigationCallbacks,
        })"
        @togglePinChat="void togglePinnedTelegramChat()"
        @toggleArchiveChat="void toggleArchivedTelegramChat()"
        @toggleMuteChat="void toggleMutedTelegramChat()"
        @toggleReadChat="void toggleReadTelegramChat()"
        @replyMessage="(message) => { replyTo = message; store.activeThreadTab = 'messages' }"
        @clearReply="replyTo = null"
      />
      <TelegramRail
        v-if="store.isTelegramInspectorOpen"
        :selectedTelegramChat="selectedTelegramChat"
        :selectedTelegramChatDetail="selectedChatDetailQuery.data.value ?? selectedTelegramChat"
        :selectedTelegramRuntimeStatus="selectedRuntimeStatus ?? null"
        :selectedTelegramMessages="selectedTelegramMessages"
        :chatMembers="selectedChatMembersQuery.data.value ?? []"
        :capabilities="capabilities"
        :isInspectorLoading="selectedChatDetailQuery.isLoading.value || selectedChatMembersQuery.isLoading.value"
        :activeRailTab="store.activeRailTab"
        @update:activeRailTab="store.activeRailTab = $event"
        @close="store.closeTelegramInspector()"
      />
    </div>
  </section>
</template>

<style scoped>
.telegram-page {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.communications-grid {
  display: flex;
  flex: 1;
  min-height: 0;
}
.three-pane {
  display: flex;
  flex: 1;
  overflow: hidden;
}
</style>
