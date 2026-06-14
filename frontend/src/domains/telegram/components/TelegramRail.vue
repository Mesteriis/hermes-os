<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramChat } from '../types/telegram'
import type { TelegramRailTab } from '../types/telegram'

const { t } = useI18n()

defineProps<{
  selectedTelegramChat: TelegramChat | null
  activeRailTab: TelegramRailTab
}>()

const emit = defineEmits<{
  'update:activeRailTab': [tab: TelegramRailTab]
  'close': []
}>()
</script>

<template>
  <aside class="stacked-rail telegram-rail">
    <section class="panel telegram-context-panel">
      <header class="telegram-inspector-head">
        <div>
          <h2>{{ t('Details') }}</h2>
          <p>{{ selectedTelegramChat?.title ?? t('No chat selected') }}</p>
        </div>
        <button type="button" :title="t('Close')" @click="emit('close')">
          <Icon icon="tabler:x" width="17" height="17" />
        </button>
      </header>

      <nav class="inspector-tabs telegram-rail-tabs">
        <button
          type="button"
          :class="{ active: activeRailTab === 'context' }"
          @click="emit('update:activeRailTab', 'context')"
        >{{ t('Context') }}</button>
        <button
          type="button"
          :class="{ active: activeRailTab === 'members' }"
          @click="emit('update:activeRailTab', 'members')"
        >{{ t('Members') }}</button>
        <button
          type="button"
          :class="{ active: activeRailTab === 'about' }"
          @click="emit('update:activeRailTab', 'about')"
        >{{ t('About') }}</button>
      </nav>

      <div class="telegram-inspector-placeholder" :aria-label="t('Details panel placeholder')"></div>
    </section>
  </aside>
</template>

<style scoped>
.telegram-rail {
  border-left: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
  min-width: 280px;
  max-width: 360px;
  display: flex;
  flex-direction: column;
}
.telegram-context-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.telegram-inspector-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-inspector-head h2 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}
.telegram-inspector-head p {
  font-size: 11px;
  margin: 2px 0 0;
  color: var(--color-text-secondary, #777);
}
.telegram-inspector-head button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-inspector-head button:hover {
  background: var(--color-bg, #f5f5f5);
}
.inspector-tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.inspector-tabs button {
  flex: 1;
  padding: 8px 12px;
  border: none;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  border-bottom: 2px solid transparent;
}
.inspector-tabs button.active {
  color: var(--color-primary, #0066cc);
  border-bottom-color: var(--color-primary, #0066cc);
  font-weight: 500;
}
.inspector-tabs button:hover {
  background: var(--color-bg, #f5f5f5);
}
.telegram-inspector-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 16px;
  color: var(--color-text-secondary, #aaa);
  font-size: 13px;
}
</style>
