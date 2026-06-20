<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { telegramChatFolderIds } from '../folderMembership'
import type { TelegramChat, TelegramChatGroupFilter } from '../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  chat: TelegramChat | null
  groupFilters: TelegramChatGroupFilter[]
  isTelegramBusy: boolean
  canAddFolder: boolean
  canRemoveFolder: boolean
  addFolderTitle?: string
  removeFolderTitle?: string
}>()

const emit = defineEmits<{
  addFolder: [providerFolderId: number]
  removeFolder: [providerFolderId: number]
}>()

const folderFilters = computed(() => props.groupFilters.filter(
  (group) => group.source === 'telegram' && typeof group.provider_folder_id === 'number'
))
const currentFolderIds = computed(() => (props.chat ? telegramChatFolderIds(props.chat) : []))
const currentFolders = computed(() => folderFilters.value.filter(
  (group) => currentFolderIds.value.includes(group.provider_folder_id!)
))
const availableFolders = computed(() => folderFilters.value.filter(
  (group) => !currentFolderIds.value.includes(group.provider_folder_id!)
))
</script>

<template>
  <section
    v-if="chat && folderFilters.length > 0"
    class="telegram-folder-membership-bar"
    :aria-label="t('Telegram folder membership')"
  >
    <div class="telegram-folder-membership-section">
      <strong>{{ t('Folders') }}</strong>
      <div class="telegram-folder-membership-list">
        <button
          v-for="group in currentFolders"
          :key="`current-${group.id}`"
          type="button"
          class="telegram-folder-chip active"
          :title="removeFolderTitle"
          :disabled="isTelegramBusy || !canRemoveFolder"
          @click="emit('removeFolder', group.provider_folder_id!)"
        >
          <Icon icon="tabler:folder-minus" width="14" height="14" />
          <span>{{ t(group.label) }}</span>
        </button>
        <span v-if="currentFolders.length === 0" class="telegram-folder-empty">
          {{ t('No Telegram folders') }}
        </span>
      </div>
    </div>
    <div class="telegram-folder-membership-section">
      <strong>{{ t('Add Folder') }}</strong>
      <div class="telegram-folder-membership-list">
        <button
          v-for="group in availableFolders"
          :key="`available-${group.id}`"
          type="button"
          class="telegram-folder-chip"
          :title="addFolderTitle"
          :disabled="isTelegramBusy || !canAddFolder"
          @click="emit('addFolder', group.provider_folder_id!)"
        >
          <Icon icon="tabler:folder-plus" width="14" height="14" />
          <span>{{ t(group.label) }}</span>
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.telegram-folder-membership-bar {
  display: flex;
  gap: 12px;
  padding: 8px 16px 0;
  flex-wrap: wrap;
}
.telegram-folder-membership-section {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.telegram-folder-membership-section strong {
  font-size: 11px;
  color: var(--color-text-muted, #666);
}
.telegram-folder-membership-list {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.telegram-folder-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
  color: var(--color-text, #444);
  font-size: 11px;
  cursor: pointer;
}
.telegram-folder-chip.active {
  background: var(--color-primary-subtle, #eaf4ff);
}
.telegram-folder-chip:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.telegram-folder-empty {
  font-size: 11px;
  color: var(--color-text-muted, #777);
}
</style>
