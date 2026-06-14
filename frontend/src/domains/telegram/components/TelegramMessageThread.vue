<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { MessageAnalyzeResponse } from '../../communications/types/communications'
import type {
  TelegramAttachmentHint,
  TelegramChat,
  TelegramMessage,
  TelegramRailTab,
  TelegramRuntimeStatus,
  TelegramThreadTab
} from '../types/telegram'
import {
  telegramAttachmentHintsForMessages,
  telegramLinkHintsForMessages,
  telegramMessagesChronological,
  telegramPinnedMessages
} from '../stores/telegram'
import TelegramComposer from './thread/TelegramComposer.vue'
import TelegramMessageList from './thread/TelegramMessageList.vue'
import TelegramSyncPanel from './thread/TelegramSyncPanel.vue'
import TelegramThreadSideSections from './thread/TelegramThreadSideSections.vue'
import TelegramThreadHeader from './thread/TelegramThreadHeader.vue'

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
  telegramManualSendForm: { text: string }
  selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null
}>()

const emit = defineEmits<{
  'update:activeThreadTab': [tab: TelegramThreadTab]
  railTabChange: [tab: TelegramRailTab]
  loadWorkspace: []
  syncHistory: []
  syncOlderHistory: []
  sendMessage: []
  downloadMedia: [attachment: TelegramAttachmentHint, message?: TelegramMessage]
}>()

const threadSearchQuery = ref('')
const isSearchOpen = ref(false)

const chronologicalMessages = computed(() =>
  telegramMessagesChronological(props.selectedTelegramMessages)
)
const fileHints = computed(() => telegramAttachmentHintsForMessages(chronologicalMessages.value))
const linkHints = computed(() => telegramLinkHintsForMessages(chronologicalMessages.value))
const pinnedMessages = computed(() => telegramPinnedMessages(chronologicalMessages.value))
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
  { id: 'topics', label: t('Topics'), count: topicCount(props.selectedTelegramChat) },
  { id: 'pinned', label: t('Pinned'), count: pinnedMessages.value.length },
  { id: 'timeline', label: t('Timeline'), count: chronologicalMessages.value.length }
])

function topicCount(chat: TelegramChat | null): number {
  const value = chat?.metadata.topics_count ?? chat?.metadata.topic_count
  return typeof value === 'number' ? value : 0
}

function updateDraftText(value: string) {
  props.telegramManualSendForm.text = value
}
</script>

<template>
  <section class="panel chat-pane telegram-chat-pane">
    <template v-if="selectedTelegramChat">
      <TelegramThreadHeader
        v-model:isSearchOpen="isSearchOpen"
        :selectedTelegramChat="selectedTelegramChat"
        :selectedTelegramRuntimeStatus="selectedTelegramRuntimeStatus"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :isTelegramLoading="isTelegramLoading"
        @update:activeThreadTab="emit('update:activeThreadTab', $event)"
        @railTabChange="emit('railTabChange', $event)"
        @loadWorkspace="emit('loadWorkspace')"
        @syncHistory="emit('syncHistory')"
      />

      <TelegramSyncPanel
        v-model:threadSearchQuery="threadSearchQuery"
        :activeThreadTab="activeThreadTab"
        :isSearchOpen="isSearchOpen"
        :tabs="tabs"
        @update:activeThreadTab="emit('update:activeThreadTab', $event)"
      />

      <TelegramMessageList
        v-if="activeThreadTab === 'messages'"
        :selectedTelegramChat="selectedTelegramChat"
        :filteredMessages="filteredMessages"
        :threadSearchQuery="threadSearchQuery"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :aiAnalysisResult="aiAnalysisResult"
        :selectedCommunication="selectedCommunication"
        :telegramMessageTime="telegramMessageTime"
        @syncOlderHistory="emit('syncOlderHistory')"
        @downloadMedia="(attachment, message) => emit('downloadMedia', attachment, message)"
      />
      <TelegramThreadSideSections
        v-else
        :activeThreadTab="activeThreadTab"
        :chronologicalMessages="chronologicalMessages"
        :fileHints="fileHints"
        :linkHints="linkHints"
        :pinnedMessages="pinnedMessages"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
        :telegramMessageTime="telegramMessageTime"
        @downloadMedia="(attachment, message) => emit('downloadMedia', attachment, message)"
      />

      <TelegramComposer
        :text="telegramManualSendForm.text"
        :isTelegramActionSubmitting="isTelegramActionSubmitting"
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
