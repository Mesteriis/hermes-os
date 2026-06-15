<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type { TelegramThreadTab } from '../../types/telegram'

const { t } = useI18n()

defineProps<{
  activeThreadTab: TelegramThreadTab
  isSearchOpen: boolean
  threadSearchQuery: string
  tabs: Array<{
    id: TelegramThreadTab
    label: string
    count: number
  }>
}>()

const emit = defineEmits<{
  'update:activeThreadTab': [tab: TelegramThreadTab]
  'update:threadSearchQuery': [value: string]
}>()
</script>

<template>
  <label v-if="isSearchOpen" class="telegram-thread-search">
    <Icon icon="tabler:search" width="16" height="16" />
    <input
      :value="threadSearchQuery"
      :placeholder="t('Search in this chat...')"
      autocomplete="off"
      @input="emit('update:threadSearchQuery', ($event.target as HTMLInputElement).value)"
    />
  </label>

  <nav class="message-context-tabs telegram-thread-tabs">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      type="button"
      :class="{ active: activeThreadTab === tab.id }"
      @click="emit('update:activeThreadTab', tab.id)"
    >
      {{ tab.label }}
      <em v-if="tab.count > 0">{{ tab.count }}</em>
    </button>
  </nav>
</template>

<style scoped>
.telegram-thread-search {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}
.telegram-thread-search input {
  border: none;
  background: transparent;
  font-size: 12px;
  outline: none;
  flex: 1;
  color: var(--color-text, #333);
}
.message-context-tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
  overflow-x: auto;
}
.message-context-tabs button {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border: none;
  background: transparent;
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  border-bottom: 2px solid transparent;
  white-space: nowrap;
}
.message-context-tabs button.active {
  color: var(--color-primary, #0066cc);
  border-bottom-color: var(--color-primary, #0066cc);
  font-weight: 500;
}
.message-context-tabs button:hover {
  background: var(--color-bg, #f5f5f5);
}
.message-context-tabs button em {
  font-style: normal;
  font-size: 10px;
  opacity: 0.6;
  background: var(--color-bg, #f0f0f0);
  border-radius: 8px;
  padding: 0 5px;
}
</style>
