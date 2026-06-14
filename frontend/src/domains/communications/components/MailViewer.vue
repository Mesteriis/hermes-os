<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import Tabs from '../../../shared/ui/Tabs.vue'
import MessageBodyTab from './MessageBodyTab.vue'
import MessageHeadersTab from './MessageHeadersTab.vue'
import MessageAttachmentsTab from './MessageAttachmentsTab.vue'
import MessageRelatedTab from './MessageRelatedTab.vue'
import MessageTimelineTab from './MessageTimelineTab.vue'
import type { MailMessageDetailResponse, MailMessageInsight, MessageContextTab } from '../types/communications'
import { senderLabel, senderEmail, messageTime } from '../stores/communications'

const props = defineProps<{
  detail: MailMessageDetailResponse | null
  insight: MailMessageInsight | null
  activeTab: MessageContextTab
}>()

const emit = defineEmits<{
  'update:activeTab': [tab: MessageContextTab]
  reply: []
  createTask: []
  createNote: []
  translate: []
  analyze: []
  togglePin: []
  toggleImportant: []
  mute: []
  exportMd: []
  openCompose: []
}>()

const message = computed(() => props.detail?.message ?? null)
const sender = computed(() => message.value ? senderLabel(message.value.sender) : '')
const email = computed(() => message.value ? senderEmail(message.value.sender) : '')
const time = computed(() => message.value ? messageTime(message.value.projected_at ?? message.value.occurred_at) : '')

const tabs = [
  { id: 'message' as MessageContextTab, label: 'Message' },
  { id: 'attachments' as MessageContextTab, label: 'Attachments' },
  { id: 'headers' as MessageContextTab, label: 'Headers' },
  { id: 'related' as MessageContextTab, label: 'Related' },
  { id: 'timeline' as MessageContextTab, label: 'Timeline' }
]

function setTab(tabId: string) {
  emit('update:activeTab', tabId as MessageContextTab)
}
</script>

<template>
  <div class="mail-viewer">
    <!-- Empty state -->
    <div v-if="!detail" class="viewer-empty">
      <Icon icon="tabler:mail" class="empty-icon" />
      <p>Select a message to view</p>
    </div>

    <!-- Message detail -->
    <div v-else class="viewer-content">
      <!-- Header -->
      <div class="viewer-header">
        <div class="header-actions-top">
          <Button variant="ghost" size="sm" @click="emit('togglePin')">
            <Icon icon="tabler:pin" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('toggleImportant')">
            <Icon icon="tabler:star" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('mute')">
            <Icon icon="tabler:bell-off" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('openCompose')">
            <Icon icon="tabler:mail-forward" />
          </Button>
        </div>
        <h2 class="viewer-subject">{{ message?.subject }}</h2>
        <div class="viewer-sender-row">
          <div class="sender-info">
            <span class="sender-name">{{ sender }}</span>
            <span class="sender-email">{{ email }}</span>
          </div>
          <span class="viewer-time">{{ time }}</span>
        </div>
      </div>

      <!-- Tabs -->
      <Tabs :tabs="tabs.map(t => ({ id: t.id, label: t.label }))" :active="activeTab" @select="setTab" />

      <!-- Tab content -->
      <div class="viewer-body">
        <MessageBodyTab
          v-if="activeTab === 'message'"
          :detail="detail"
          :insight="insight"
          @reply="emit('reply')"
          @create-task="emit('createTask')"
          @create-note="emit('createNote')"
          @translate="emit('translate')"
          @analyze="emit('analyze')"
        />
        <MessageAttachmentsTab v-else-if="activeTab === 'attachments'" :detail="detail" />
        <MessageHeadersTab v-else-if="activeTab === 'headers'" :detail="detail" />
        <MessageRelatedTab
          v-else-if="activeTab === 'related'"
          :detail="detail"
          @toggle-pin="emit('togglePin')"
          @toggle-important="emit('toggleImportant')"
          @mute="emit('mute')"
          @export-md="emit('exportMd')"
        />
        <MessageTimelineTab v-else-if="activeTab === 'timeline'" :detail="detail" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.mail-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.viewer-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--hh-text-secondary, #6b7280);
  gap: 0.75rem;
}

.empty-icon {
  width: 48px;
  height: 48px;
  opacity: 0.3;
}

.viewer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.viewer-header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.header-actions-top {
  display: flex;
  gap: 0.25rem;
  justify-content: flex-end;
}

.viewer-subject {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.3;
}

.viewer-sender-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.sender-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.sender-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.sender-email {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.viewer-time {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
}

.viewer-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
}
</style>
