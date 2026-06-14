<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useNavigationStore } from '../../../shared/stores/navigation'

// Components
import TelegramActionRail from '../components/TelegramActionRail.vue'
import TelegramChatList from '../components/TelegramChatList.vue'
import TelegramCommandHeader from '../components/TelegramCommandHeader.vue'
import TelegramMessageThread from '../components/TelegramMessageThread.vue'
import TelegramRail from '../components/TelegramRail.vue'
import TelegramStatusMessages from '../components/TelegramStatusMessages.vue'

// Store
import { useTelegramStore, telegramChatGroupFilters, telegramMessageTime } from '../stores/telegram'

// TanStack Query
import { useTelegramCapabilitiesQuery } from '../queries/useTelegramQuery'

// API
import * as telegramApi from '../api/telegram'

const { t } = useI18n()
const navigationStore = useNavigationStore()
const store = useTelegramStore()

// Capabilities query
const { data: capabilities } = useTelegramCapabilitiesQuery()

// Computed
const runtimeLabel = computed(() => {
  if (!store.selectedTelegramRuntimeStatus) {
    return capabilities.value?.runtime_mode ?? t('Runtime Status')
  }
  return store.selectedTelegramRuntimeStatus.status
})

const filterTabs = computed(() => [
  { id: 'all' as const, label: 'All Chats' },
  { id: 'unread' as const, label: 'Unread' },
  { id: 'mentions' as const, label: 'Mentions' },
  { id: 'pinned' as const, label: 'Pinned' },
  { id: 'projects' as const, label: 'Projects' },
  { id: 'bots' as const, label: 'Bots' },
  { id: 'archived' as const, label: 'Archived' }
])

// Load data
onMounted(() => {
  void loadTelegramWorkspace()
})

async function loadTelegramWorkspace() {
  store.setTelegramLoading(true)
  try {
    const result = await telegramApi.loadTelegramWorkspace(
      store.selectedTelegramChatId,
      ''
    )
    store.setTelegramData(result)
    // Reset group filter if current group no longer exists
    const groups = telegramChatGroupFilters(store.telegramChats)
    if (!groups.some((g) => g.id === store.activeTelegramGroupFilter)) {
      store.activeTelegramGroupFilter = 'local:all'
    }
    const nextChat = store.telegramChats.find(
      (chat) => chat.provider_chat_id === store.selectedTelegramChatId
    ) ?? null
    void autoSyncTelegramHistory(nextChat)
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramLoading(false)
  }
}

function selectTelegramChat(chat: any) {
  store.selectTelegramChat(chat)
  void loadTelegramWorkspace()
}

const telegramAutoHistorySyncKeys = new Set<string>()

async function autoSyncTelegramHistory(chat: any) {
  if (!chat || store.isTelegramBusy) return
  const syncKey = `${chat.account_id}:${chat.provider_chat_id}`
  if (telegramAutoHistorySyncKeys.has(syncKey)) return
  if (chat.chat_kind !== 'private' && hasProjectedMessagesForChat(chat)) return

  telegramAutoHistorySyncKeys.add(syncKey)
  store.setTelegramHistorySyncing(true)
  store.setTelegramError('')
  try {
    const result = await telegramApi.syncTelegramSelectedHistory({
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      chat_kind: chat.chat_kind
    })
    if (result.error) {
      telegramAutoHistorySyncKeys.delete(syncKey)
      store.setTelegramError(result.error)
    } else {
      store.setTelegramActionMessage(result.message)
      await loadTelegramWorkspace()
    }
  } finally {
    store.setTelegramHistorySyncing(false)
  }
}

function hasProjectedMessagesForChat(chat: any): boolean {
  return store.telegramMessages.some(
    (message) =>
      message.account_id === chat.account_id &&
      message.provider_chat_id === chat.provider_chat_id
  )
}

async function startTelegramRuntime() {
  if (store.isTelegramBusy || !store.selectedTelegramChat) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await telegramApi.startTelegramRuntimeFromUi(
      store.selectedTelegramChat.account_id
    )
    if (result.error) {
      store.setTelegramError(result.error)
    } else if (result.status) {
      store.updateRuntimeStatus(result.status.account_id, result.status)
      store.setTelegramActionMessage(result.message)
    }
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}

async function syncTelegramChats() {
  if (store.isTelegramBusy || !store.selectedTelegramChat) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await telegramApi.syncTelegramChatsFromUi(
      store.selectedTelegramChat.account_id
    )
    if (result.error) {
      store.setTelegramError(result.error)
    } else {
      store.setTelegramActionMessage(result.message)
      await loadTelegramWorkspace()
    }
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}

async function syncSelectedTelegramHistory() {
  if (store.isTelegramBusy || !store.selectedTelegramChat) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await telegramApi.syncTelegramSelectedHistory({
      account_id: store.selectedTelegramChat.account_id,
      provider_chat_id: store.selectedTelegramChat.provider_chat_id,
      chat_kind: store.selectedTelegramChat.chat_kind
    })
    if (result.error) {
      store.setTelegramError(result.error)
    } else {
      store.selectedTelegramChatId = result.providerChatId
      store.setTelegramActionMessage(result.message)
      await loadTelegramWorkspace()
    }
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}

async function syncOlderTelegramHistory() {
  if (store.isTelegramBusy || !store.selectedTelegramChat) return
  const fromMessageId = telegramApi.telegramOldestTdlibMessageId(store.selectedTelegramMessages)
  if (fromMessageId === null) {
    await autoSyncTelegramHistory(store.selectedTelegramChat)
    return
  }
  store.setTelegramHistorySyncing(true)
  store.setTelegramError('')
  try {
    const result = await telegramApi.syncTelegramOlderHistory({
      account_id: store.selectedTelegramChat.account_id,
      provider_chat_id: store.selectedTelegramChat.provider_chat_id,
      from_message_id: fromMessageId
    })
    if (result.error) {
      store.setTelegramError(result.error)
    } else {
      store.setTelegramActionMessage(result.hasMore
        ? result.message
        : `${result.message}; ${t('no older Telegram messages')}`
      )
      await loadTelegramWorkspace()
    }
  } finally {
    store.setTelegramHistorySyncing(false)
  }
}

async function sendTelegramManualMessage() {
  if (store.isTelegramBusy || !store.selectedTelegramChat) return
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await telegramApi.sendTelegramManualMessage({
      account_id: store.selectedTelegramChat.account_id,
      provider_chat_id: store.selectedTelegramChat.provider_chat_id,
      text: store.telegramManualSendForm.text
    })
    if (result.error) {
      store.setTelegramError(result.error)
    } else {
      store.selectedTelegramChatId = result.providerChatId
      store.setTelegramActionMessage(result.message)
      store.telegramManualSendForm = { text: result.nextText }
      await loadTelegramWorkspace()
    }
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}

async function downloadTelegramMedia(attachment: any, message?: any) {
  if (store.isTelegramBusy) return
  if (!store.selectedTelegramChat) {
    store.setTelegramError(t('Select a Telegram chat before downloading media.'))
    return
  }
  if (attachment.tdlibFileId === null) {
    store.setTelegramError(t('Telegram attachment does not include TDLib file metadata.'))
    return
  }
  const sourceMessage =
    message ??
    store.selectedTelegramMessages.find(
      (item: any) => item.message_id === attachment.messageId
    ) ??
    null
  if (!sourceMessage) {
    store.setTelegramError(t('Telegram source message is not available for this attachment.'))
    return
  }
  store.setTelegramActionSubmitting(true)
  store.setTelegramActionMessage('')
  store.setTelegramError('')
  try {
    const result = await telegramApi.downloadTelegramMediaFromUi({
      account_id: sourceMessage.account_id,
      provider_chat_id: sourceMessage.provider_chat_id || store.selectedTelegramChat.provider_chat_id,
      provider_message_id: sourceMessage.provider_message_id,
      tdlib_file_id: attachment.tdlibFileId,
      provider_attachment_id: attachment.providerAttachmentId,
      filename: attachment.fileName,
      content_type: attachment.mimeType ?? undefined,
      priority: 16
    })
    if (result.error) {
      store.setTelegramError(result.error)
    } else {
      store.setTelegramActionMessage(result.message)
      await loadTelegramWorkspace()
    }
  } catch (err) {
    store.setTelegramError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setTelegramActionSubmitting(false)
  }
}

function openTelegramNewMessage() {
  store.closeTelegramMenus()
  store.activeThreadTab = 'messages'
  if (!store.selectedTelegramChat) {
    store.setTelegramActionMessage(t('Select a synced Telegram chat before composing.'))
    return
  }
  store.telegramManualSendForm = { text: '' }
  store.setTelegramActionMessage(`${t('Manual send target')}: ${store.selectedTelegramChat.title}`)
}

async function runTelegramQuickAction(action: 'create_note' | 'create_task' | 'create_contact' | 'create_document') {
  store.closeTelegramMenus()
  // Quick actions would use navigation store or a workflow action
}

function openTelegramAccountSetup() {
  // Would open account drawer
}

// Format helper
function formatDateTime(date: string | null): string {
  if (!date) return ''
  try {
    return new Date(date).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return ''
  }
}
</script>

<template>
  <section class="telegram-page communications-page">
    <TelegramCommandHeader
      :runtimeLabel="runtimeLabel"
      :searchQuery="store.telegramSearchQuery"
      :filterTabs="filterTabs"
      :filterCounts="store.chatFilterCounts"
      :activeFilter="store.activeTelegramFilter"
      :isFiltersMenuOpen="store.isTelegramFiltersMenuOpen"
      :isNewMenuOpen="store.isTelegramNewMenuOpen"
      :isTelegramBusy="store.isTelegramBusy"
      :selectedTelegramChat="store.selectedTelegramChat"
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
      :groupFilters="store.chatGroupFilters"
      :activeGroupFilter="store.activeTelegramGroupFilter"
      :isTelegramBusy="store.isTelegramBusy"
      :hasSelectedTelegramChat="Boolean(store.selectedTelegramChat)"
      :isInspectorOpen="store.isTelegramInspectorOpen"
      @syncChats="void syncTelegramChats()"
      @syncHistory="void syncSelectedTelegramHistory()"
      @startRuntime="void startTelegramRuntime()"
      @selectGroupFilter="store.selectTelegramGroupFilter($event)"
      @toggleInspector="store.toggleTelegramInspector()"
    />

    <TelegramStatusMessages
      :actionMessage="store.telegramActionMessage"
      :error="store.telegramError"
    />

    <div
      class="three-pane communications-grid telegram-grid"
      :class="{ 'inspector-open': store.isTelegramInspectorOpen }"
    >
      <TelegramChatList
        :telegramChats="store.filteredTelegramChats"
        :telegramMessages="store.telegramMessages"
        :selectedTelegramChatId="store.selectedTelegramChat?.provider_chat_id ?? ''"
        :isTelegramLoading="store.isTelegramLoading"
        :formatDateTime="formatDateTime"
        @selectChat="selectTelegramChat"
      />

      <TelegramMessageThread
        :selectedTelegramChat="store.selectedTelegramChat"
        :selectedTelegramMessages="store.selectedTelegramMessages"
        :aiAnalysisResult="null"
        :selectedCommunication="null"
        :isTelegramLoading="store.isTelegramLoading"
        :isTelegramActionSubmitting="store.isTelegramBusy"
        :activeThreadTab="store.activeThreadTab"
        :telegramMessageTime="telegramMessageTime"
        :telegramManualSendForm="store.telegramManualSendForm"
        :selectedTelegramRuntimeStatus="store.selectedTelegramRuntimeStatus"
        @update:activeThreadTab="store.activeThreadTab = $event"
        @railTabChange="(tab) => store.openTelegramInspector(tab)"
        @loadWorkspace="void loadTelegramWorkspace()"
        @syncHistory="void syncSelectedTelegramHistory()"
        @syncOlderHistory="void syncOlderTelegramHistory()"
        @sendMessage="void sendTelegramManualMessage()"
        @downloadMedia="(attachment, message) => void downloadTelegramMedia(attachment, message)"
      />

      <TelegramRail
        v-if="store.isTelegramInspectorOpen"
        :selectedTelegramChat="store.selectedTelegramChat"
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
