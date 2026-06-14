<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramChat } from '../types/telegram'
import type { TelegramChatFilter, TelegramChatFilterCount } from '../types/telegram'

const { t } = useI18n()

interface TelegramFilterTab {
  id: TelegramChatFilter
  label: string
}

const props = defineProps<{
  runtimeLabel: string
  searchQuery: string
  filterTabs: TelegramFilterTab[]
  filterCounts: TelegramChatFilterCount[]
  activeFilter: TelegramChatFilter
  isFiltersMenuOpen: boolean
  isNewMenuOpen: boolean
  isTelegramBusy: boolean
  selectedTelegramChat: TelegramChat | null
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  'toggleFiltersMenu': []
  'toggleNewMenu': []
  'selectFilter': [filter: TelegramChatFilter]
  'syncChats': []
  'addAccount': []
  'newMessage': []
  'quickAction': [action: 'create_note' | 'create_task' | 'create_contact' | 'create_document']
}>()

function filterCount(filter: TelegramChatFilter): number {
  const item = props.filterCounts?.find((f: TelegramChatFilterCount) => f.filter === filter)
  return item?.count ?? 0
}
</script>

<template>
  <header class="communications-command-header telegram-command-header">
    <div class="command-title telegram-command-title">
      <h1>{{ t('Communications') }} <span>/</span> {{ t('Telegram') }}</h1>
      <p>{{ runtimeLabel }}</p>
    </div>
    <label class="command-search">
      <Icon icon="tabler:search" width="18" height="18" />
      <input
        :value="searchQuery"
        :placeholder="t('Search conversations...')"
        autocomplete="off"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <div class="command-menu">
      <button
        type="button"
        :class="{ active: isFiltersMenuOpen || activeFilter !== 'all' }"
        @click="emit('toggleFiltersMenu')"
      >
        <Icon icon="tabler:filter" width="17" height="17" />{{ t('Filters') }}
      </button>
      <div v-if="isFiltersMenuOpen" class="command-popover filter-command-popover">
        <button
          v-for="tab in filterTabs"
          :key="tab.id"
          type="button"
          :class="{ active: activeFilter === tab.id }"
          @click="emit('selectFilter', tab.id)"
        >
          <span>{{ t(tab.label) }}</span><em>{{ filterCount(tab.id) }}</em>
        </button>
        <button
          type="button"
          :disabled="isTelegramBusy || !selectedTelegramChat"
          @click="emit('syncChats')"
        >
          <span><Icon icon="tabler:refresh" width="15" height="15" />{{ t('Sync Chats') }}</span>
        </button>
      </div>
    </div>
    <div class="command-menu telegram-add-account">
      <button type="button" @click="emit('addAccount')">
        <Icon icon="tabler:user-plus" width="17" height="17" />{{ t('Add Account') }}
      </button>
    </div>
    <div class="command-menu new-command">
      <button type="button" class="primary-button" @click="emit('toggleNewMenu')">
        {{ t('New') }}<Icon icon="tabler:plus" width="17" height="17" />
      </button>
      <div v-if="isNewMenuOpen" class="command-popover new-command-popover">
        <button type="button" @click="emit('newMessage')">
          <Icon icon="tabler:send" width="16" height="16" />{{ t('New Message') }}
        </button>
        <button type="button" @click="emit('quickAction', 'create_note')">
          <Icon icon="tabler:notes" width="16" height="16" />{{ t('New Note') }}
        </button>
        <button type="button" @click="emit('quickAction', 'create_task')">
          <Icon icon="tabler:square-check" width="16" height="16" />{{ t('New Task') }}
        </button>
        <button type="button" @click="emit('quickAction', 'create_contact')">
          <Icon icon="tabler:user-plus" width="16" height="16" />{{ t('New Contact') }}
        </button>
        <button type="button" @click="emit('quickAction', 'create_document')">
          <Icon icon="tabler:file-plus" width="16" height="16" />{{ t('New Document') }}
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.telegram-command-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: var(--color-surface, #fff);
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  flex-wrap: wrap;
}
.command-title h1 {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}
.command-title h1 span {
  opacity: 0.4;
  padding: 0 4px;
}
.command-title p {
  font-size: 11px;
  margin: 0;
  color: var(--color-text-secondary, #777);
}
.command-search {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--color-bg, #f5f5f5);
  border-radius: 8px;
  padding: 4px 10px;
  flex: 1;
  max-width: 280px;
}
.command-search input {
  border: none;
  background: transparent;
  font-size: 12px;
  outline: none;
  width: 100%;
  color: var(--color-text, #333);
}
.command-menu {
  position: relative;
}
.command-menu button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-surface, #fff);
  font-size: 12px;
  cursor: pointer;
  color: var(--color-text, #333);
}
.command-menu button.active {
  background: var(--color-primary-subtle, #e3f2fd);
  border-color: var(--color-primary, #0066cc);
}
.primary-button {
  background: var(--color-primary, #0066cc) !important;
  color: #fff !important;
  border-color: var(--color-primary, #0066cc) !important;
}
.command-popover {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 100;
  min-width: 180px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  margin-top: 4px;
}
.command-popover button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: 4px;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  color: var(--color-text, #333);
}
.command-popover button:hover {
  background: var(--color-bg, #f5f5f5);
}
.command-popover button em {
  font-style: normal;
  font-size: 11px;
  opacity: 0.6;
}
</style>
