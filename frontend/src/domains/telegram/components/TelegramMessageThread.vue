<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { MessageAnalyzeResponse } from '../../communications/types/communications'
import type {
  TelegramAttachmentHint,
  TelegramCapabilitiesResponse,
  TelegramChat,
  TelegramMediaItem,
  TelegramMessage,
  TelegramRailTab,
  TelegramRuntimeStatus,
  TelegramThreadTab
} from '../types/telegram'
import {
  telegramAttachmentHintsForMessages,
  telegramLinkHintsForMessages,
  telegramMessagesChronological,
  telegramPinnedMessages,
  telegramVoiceAttachmentHintsForMessages
} from '../stores/telegram'
import TelegramComposer from './thread/TelegramComposer.vue'
import TelegramMessageList from './thread/TelegramMessageList.vue'
import TelegramSearchResultsPanel from './TelegramSearchResultsPanel.vue'
import TelegramSyncPanel from './thread/TelegramSyncPanel.vue'
import TelegramThreadSideSections from './thread/TelegramThreadSideSections.vue'
import TelegramThreadHeader from './thread/TelegramThreadHeader.vue'
import { useTelegramPinnedMessagesQuery } from '../queries/useTelegramSearchQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedTelegramChat: TelegramChat | null
  selectedTelegramMessages: TelegramMessage[]
  aiAnalysisResult: MessageAnalyzeResponse | null
  selectedCommunication: { message_id?: string } | null
  isTelegramLoading: boolean
  isTelegramActionSubmitting: boolean
  activeThreadTab: TelegramThreadTab
  telegramMessageTime: (message: TelegramMessage) => string
  telegramManualSendText: string
  selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null
  capabilities?: TelegramCapabilitiesResponse | null
  workspaceSearchQuery: string
  searchChats: TelegramChat[]
  searchResults: TelegramMessage[]
  searchResultTotal: number
  mediaGalleryItems: TelegramMediaItem[]
  isWorkspaceSearchLoading: boolean
  focusedTelegramMessage?: TelegramMessage | null
}>()

const emit = defineEmits<{
  'update:activeThreadTab': [tab: TelegramThreadTab]
  'update:telegramManualSendText': [value: string]
  railTabChange: [tab: TelegramRailTab]
  loadWorkspace: []
  syncHistory: []
  syncOlderHistory: []
  sendMessage: []
  downloadMedia: [attachment: TelegramAttachmentHint, message?: TelegramMessage]
  editMessage: [message: TelegramMessage]
  deleteMessage: [message: TelegramMessage]
  restoreMessage: [message: TelegramMessage]
  togglePinMessage: [message: TelegramMessage]
  addReaction: [payload: { message: TelegramMessage; emoji: string }]
  removeReaction: [payload: { message: TelegramMessage; emoji: string }]
  openSearchChat: [chat: TelegramChat]
  openSearchMessage: [message: TelegramMessage]
  openSearchMedia: [item: TelegramMediaItem]
  togglePinChat: []
  toggleArchiveChat: []
  toggleMuteChat: []
  toggleReadChat: []
}>()

const threadSearchQuery = ref('')
const isSearchOpen = ref(false)
const pinnedMessagesQuery = useTelegramPinnedMessagesQuery({
  telegramChatId: computed(() => props.selectedTelegramChat?.telegram_chat_id ?? null),
  limit: 100
})

const chronologicalMessages = computed(() => {
  const messages = props.selectedTelegramMessages.slice()
  const focusedMessage = props.focusedTelegramMessage
  if (
    focusedMessage &&
    props.selectedTelegramChat &&
    focusedMessage.account_id === props.selectedTelegramChat.account_id &&
    focusedMessage.provider_chat_id === props.selectedTelegramChat.provider_chat_id &&
    !messages.some((message) => message.message_id === focusedMessage.message_id)
  ) {
    messages.push(focusedMessage)
  }
  return telegramMessagesChronological(messages)
})
const fileHints = computed(() => telegramAttachmentHintsForMessages(chronologicalMessages.value))
const voiceHints = computed(() => telegramVoiceAttachmentHintsForMessages(chronologicalMessages.value))
const linkHints = computed(() => telegramLinkHintsForMessages(chronologicalMessages.value))
const pinnedMessages = computed(() => {
  const queryItems = pinnedMessagesQuery.data.value?.items ?? []
  return queryItems.length > 0
    ? telegramMessagesChronological(queryItems)
    : telegramPinnedMessages(chronologicalMessages.value)
})
const filteredMessages = computed(() =>
  chronologicalMessages.value.filter((message) => {
    const query = threadSearchQuery.value.trim().toLowerCase()
    if (!query) return true
    return [
      message.text,
      message.sender,
      message.sender_display_name ?? '',
      message.provider_message_id
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  })
)

type TabItem = {
  id: TelegramThreadTab
  label: string
  count: number
}

const tabs = computed<TabItem[]>(() => [
  { id: 'messages', label: t('Messages'), count: chronologicalMessages.value.length },
  { id: 'files', label: t('Files'), count: fileHints.value.length },
  { id: 'links', label: t('Links'), count: linkHints.value.length },
  { id: 'voice', label: t('Voice'), count: voiceHints.value.length },
  { id: 'topics', label: t('Topics'), count: topicCount(props.selectedTelegramChat) },
  { id: 'pinned', label: t('Pinned'), count: pinnedMessages.value.length },
  { id: 'timeline', label: t('Timeline'), count: chronologicalMessages.value.length }
])

function topicCount(chat: TelegramChat | null): number {
  const value = chat?.metadata.topics_count ?? chat?.metadata.topic_count
  return typeof value === 'number' ? value : 0
}

function updateDraftText(value: string) {
  emit('update:telegramManualSendText', value)
}

watch(
  () => props.focusedTelegramMessage?.message_id ?? null,
  (messageId) => {
    if (!messageId || !props.focusedTelegramMessage) return
    threadSearchQuery.value = props.focusedTelegramMessage.provider_message_id
    isSearchOpen.value = true
  }
)
</script>

<template>
  <section class="panel chat-pane telegram-chat-pane">
    <template v-if="selectedTelegramChat">
      <TelegramThreadHeader
        v-model:isSearchOpen="isSearchOpen"
        :selectedTelegramChat="selectedTelegramChat"
        :selectedTelegramRuntimeStatus="selectedTelegramRuntimeStatus"
        :capabilities="capabilities"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :isTelegramLoading="isTelegramLoading"
        @update:activeThreadTab="emit('update:activeThreadTab', $event)"
        @railTabChange="emit('railTabChange', $event)"
        @loadWorkspace="emit('loadWorkspace')"
        @syncHistory="emit('syncHistory')"
        @toggleReadChat="emit('toggleReadChat')"
        @togglePinChat="emit('togglePinChat')"
        @toggleArchiveChat="emit('toggleArchiveChat')"
        @toggleMuteChat="emit('toggleMuteChat')"
      />

      <TelegramSyncPanel
        v-if="!workspaceSearchQuery.trim()"
        v-model:threadSearchQuery="threadSearchQuery"
        :activeThreadTab="activeThreadTab"
        :isSearchOpen="isSearchOpen"
        :tabs="tabs"
        @update:activeThreadTab="emit('update:activeThreadTab', $event)"
      />

      <TelegramSearchResultsPanel
        v-if="workspaceSearchQuery.trim()"
        :query="workspaceSearchQuery"
        :chats="searchChats"
        :results="searchResults"
        :total="searchResultTotal"
        :mediaItems="mediaGalleryItems"
        :isLoading="isWorkspaceSearchLoading"
        @openChat="(chat) => emit('openSearchChat', chat)"
        @openMessage="(message) => emit('openSearchMessage', message)"
        @openMedia="(item) => emit('openSearchMedia', item)"
      />
      <TelegramMessageList
        v-else-if="activeThreadTab === 'messages'"
        :selectedTelegramChat="selectedTelegramChat"
        :filteredMessages="filteredMessages"
        :threadSearchQuery="threadSearchQuery"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :aiAnalysisResult="aiAnalysisResult"
        :selectedCommunication="selectedCommunication"
        :telegramMessageTime="telegramMessageTime"
        :capabilities="capabilities"
        @syncOlderHistory="emit('syncOlderHistory')"
        @downloadMedia="(attachment, message) => emit('downloadMedia', attachment, message)"
        @editMessage="(message) => emit('editMessage', message)"
        @deleteMessage="(message) => emit('deleteMessage', message)"
        @restoreMessage="(message) => emit('restoreMessage', message)"
        @togglePinMessage="(message) => emit('togglePinMessage', message)"
        @addReaction="(payload) => emit('addReaction', payload)"
        @removeReaction="(payload) => emit('removeReaction', payload)"
        @openSearchMessage="(message) => emit('openSearchMessage', message)"
      />
      <TelegramThreadSideSections
        v-else
        :activeThreadTab="activeThreadTab"
        :chronologicalMessages="chronologicalMessages"
        :fileHints="fileHints"
        :voiceHints="voiceHints"
        :mediaGalleryItems="mediaGalleryItems"
        :linkHints="linkHints"
        :pinnedMessages="pinnedMessages"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :telegramMessageTime="telegramMessageTime"
        @downloadMedia="(attachment, message) => emit('downloadMedia', attachment, message)"
        @openMessage="(message) => emit('openSearchMessage', message)"
      />

      <TelegramComposer
        v-if="!workspaceSearchQuery.trim()"
        :text="telegramManualSendText"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :selectedAccountId="selectedTelegramChat.account_id"
        :selectedProviderChatId="selectedTelegramChat.provider_chat_id"
        @update:text="updateDraftText"
        @sendMessage="emit('sendMessage')"
        @syncHistory="emit('syncHistory')"
      />
    </template>
    <div v-else class="empty-panel fill">
      {{ t('Select a Telegram chat to inspect messages and compose replies.') }}
    </div>
  </section>
</template>

<style scoped>
.telegram-chat-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--color-bg, #f9f9f9);
}
.empty-panel.fill {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: 13px;
  color: var(--color-text-secondary, #999);
}
</style>
