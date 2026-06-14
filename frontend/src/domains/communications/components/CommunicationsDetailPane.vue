<script setup lang="ts">
import MailViewer from './MailViewer.vue'
import type { MailMessageDetailResponse, MailMessageInsight, MessageContextTab } from '../types/communications'

defineProps<{
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
</script>

<template>
  <main class="communications-detail-pane">
    <MailViewer
      :detail="detail"
      :insight="insight"
      :active-tab="activeTab"
      @update:active-tab="emit('update:activeTab', $event)"
      @reply="emit('reply')"
      @create-task="emit('createTask')"
      @create-note="emit('createNote')"
      @translate="emit('translate')"
      @analyze="emit('analyze')"
      @toggle-pin="emit('togglePin')"
      @toggle-important="emit('toggleImportant')"
      @mute="emit('mute')"
      @export-md="emit('exportMd')"
      @open-compose="emit('openCompose')"
    />
  </main>
</template>

<style scoped>
.communications-detail-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--hh-bg-primary, #ffffff);
  backdrop-filter: blur(var(--hh-panel-blur));
}
</style>
