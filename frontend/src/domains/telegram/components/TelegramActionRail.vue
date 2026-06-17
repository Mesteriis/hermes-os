<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramChatGroupFilter } from '../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  groupFilters: TelegramChatGroupFilter[]
  activeGroupFilter: string
  isTelegramBusy: boolean
  hasSelectedTelegramChat: boolean
  isInspectorOpen: boolean
  isActiveFolderMember?: boolean
  canAddActiveFolder?: boolean
  addActiveFolderTitle?: string
  canRemoveActiveFolder?: boolean
  removeActiveFolderTitle?: string
  canReassignActiveFolder?: boolean
  reassignActiveFolderTitle?: string
  canMoveToActiveFolder?: boolean
}>()

const emit = defineEmits<{
  'syncChats': []
  'syncHistory': []
  'startRuntime': []
  'stopRuntime': []
  'restartRuntime': []
  'selectGroupFilter': [filter: TelegramChatGroupFilter]
  'addToActiveFolder': [providerFolderId: number]
  'removeFromActiveFolder': [providerFolderId: number]
  'moveToActiveFolder': [providerFolderId: number]
  'toggleInspector': []
}>()

function activeTelegramFolder(): TelegramChatGroupFilter | null {
  return props.groupFilters.find(
    (group) =>
      group.id === props.activeGroupFilter &&
      group.source === 'telegram' &&
      typeof group.provider_folder_id === 'number'
  ) ?? null
}
</script>

<template>
  <section class="telegram-action-rail" :aria-label="t('Telegram actions')">
    <div class="telegram-action-cluster">
      <button
        type="button"
        :disabled="isTelegramBusy || !hasSelectedTelegramChat"
        @click="emit('syncChats')"
      >
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Sync Chats') }}
      </button>
      <button
        type="button"
        :disabled="isTelegramBusy || !hasSelectedTelegramChat"
        @click="emit('syncHistory')"
      >
        <Icon icon="tabler:history" width="16" height="16" />{{ t('Sync History') }}
      </button>
      <button
        type="button"
        :disabled="isTelegramBusy || !hasSelectedTelegramChat"
        @click="emit('startRuntime')"
      >
        <Icon icon="tabler:player-play" width="16" height="16" />{{ t('Start Runtime') }}
      </button>
      <button
        type="button"
        :disabled="isTelegramBusy || !hasSelectedTelegramChat"
        @click="emit('stopRuntime')"
      >
        <Icon icon="tabler:player-stop" width="16" height="16" />{{ t('Stop Runtime') }}
      </button>
      <button
        type="button"
        :disabled="isTelegramBusy || !hasSelectedTelegramChat"
        @click="emit('restartRuntime')"
      >
        <Icon icon="tabler:reload" width="16" height="16" />{{ t('Restart Runtime') }}
      </button>
    </div>
    <div class="telegram-group-filter-strip" :aria-label="t('Chat Groups')">
      <template v-for="group in groupFilters" :key="group.id">
        <button
          v-if="group.count > 0 || group.id === 'local:all'"
          type="button"
          :class="{ active: activeGroupFilter === group.id }"
          :title="group.source === 'telegram' ? t('Telegram folder') : t('Local group')"
          @click="emit('selectGroupFilter', group)"
        >
          <Icon :icon="group.icon" width="15" height="15" />
          <span>{{ t(group.label) }}</span>
          <em>{{ group.count }}</em>
          <small v-if="group.source === 'telegram'">TG</small>
        </button>
      </template>
    </div>
    <button
      v-if="activeTelegramFolder()"
      type="button"
      class="telegram-folder-action"
      :title="addActiveFolderTitle"
      :disabled="isTelegramBusy || !hasSelectedTelegramChat || !canAddActiveFolder || isActiveFolderMember"
      @click="emit('addToActiveFolder', activeTelegramFolder()!.provider_folder_id!)"
    >
      <Icon icon="tabler:folder-plus" width="16" height="16" />{{ t('Add to Active Folder') }}
    </button>
    <button
      v-if="activeTelegramFolder()"
      type="button"
      class="telegram-folder-action"
      :title="removeActiveFolderTitle"
      :disabled="isTelegramBusy || !hasSelectedTelegramChat || !canRemoveActiveFolder || !isActiveFolderMember"
      @click="emit('removeFromActiveFolder', activeTelegramFolder()!.provider_folder_id!)"
    >
      <Icon icon="tabler:folder-minus" width="16" height="16" />{{ t('Remove from Active Folder') }}
    </button>
    <button
      v-if="activeTelegramFolder()"
      type="button"
      class="telegram-folder-action"
      :title="reassignActiveFolderTitle"
      :disabled="isTelegramBusy || !hasSelectedTelegramChat || !canReassignActiveFolder || !canMoveToActiveFolder"
      @click="emit('moveToActiveFolder', activeTelegramFolder()!.provider_folder_id!)"
    >
      <Icon icon="tabler:folders" width="16" height="16" />{{ t('Move to Active Folder') }}
    </button>
    <button
      type="button"
      class="telegram-inspector-toggle"
      :class="{ active: isInspectorOpen }"
      @click="emit('toggleInspector')"
    >
      <Icon icon="tabler:layout-sidebar-right" width="16" height="16" />{{ t('Details') }}
    </button>
  </section>
</template>

<style scoped>
.telegram-action-rail {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: var(--color-surface, #fff);
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  flex-wrap: wrap;
}
.telegram-action-cluster {
  display: flex;
  gap: 4px;
}
.telegram-action-cluster button,
.telegram-inspector-toggle,
.telegram-folder-action {
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
.telegram-action-cluster button:disabled,
.telegram-folder-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.telegram-group-filter-strip {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}
.telegram-group-filter-strip button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 12px;
  background: var(--color-surface, #fff);
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text, #555);
}
.telegram-group-filter-strip button.active {
  background: var(--color-primary, #0066cc);
  color: #fff;
  border-color: var(--color-primary, #0066cc);
}
.telegram-group-filter-strip button em,
.telegram-group-filter-strip button small {
  font-style: normal;
  font-size: 10px;
  opacity: 0.7;
}
.telegram-inspector-toggle.active {
  background: var(--color-primary-subtle, #e3f2fd);
}
</style>
