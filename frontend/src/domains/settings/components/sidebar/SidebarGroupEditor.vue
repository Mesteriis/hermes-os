<script setup lang="ts">
import type { SidebarItemId, SidebarNavGroup } from '../../../../shared/stores/sidebar'
import SidebarItemEditor from './SidebarItemEditor.vue'

defineProps<{
  group: SidebarNavGroup
  groupIndex: number
  rootIndex: number
  rootItemCount: number
  itemLabels: Record<SidebarItemId, { label: string; icon: string }>
  hiddenItemIds: SidebarItemId[]
  groupOptions: Array<{ value: string; label: string }>
  groupLabelText: string
  defaultPlaceholder: string
  groupPlaceholder: string
  visibleDomainLabel: string
  hiddenLabel: string
  noItemsLabel: string
  moveToGroupLabel: string
  dividerLabel: string
  showLabel: string
  hideLabel: string
}>()

defineEmits<{
  rename: [groupId: string, label: string]
  moveGroup: [groupId: string, direction: -1 | 1]
  removeGroup: [groupId: string]
  moveItemToGroup: [itemId: SidebarItemId, targetGroupId: string]
  moveItem: [itemId: SidebarItemId, direction: -1 | 1]
  toggleDivider: [groupId: string, itemId: SidebarItemId]
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <section class="sidebar-config-group">
    <header>
      <label>
        <span>{{ groupLabelText }}</span>
        <input
          :value="group.label"
          :placeholder="groupIndex === 0 ? defaultPlaceholder : groupPlaceholder"
          autocomplete="off"
          @input="$emit('rename', group.id, ($event.target as HTMLInputElement).value)"
        />
      </label>
      <div class="sidebar-config-group-actions">
        <button
          type="button"
          class="hermes-btn hermes-btn--icon"
          :disabled="rootIndex <= 0"
          @click="$emit('moveGroup', group.id, -1)"
        >
          ↑
        </button>
        <button
          type="button"
          class="hermes-btn hermes-btn--icon"
          :disabled="rootIndex === rootItemCount - 1"
          @click="$emit('moveGroup', group.id, 1)"
        >
          ↓
        </button>
        <button
          type="button"
          class="hermes-btn hermes-btn--icon hermes-btn--destructive"
          :disabled="group.id === 'communications'"
          @click="$emit('removeGroup', group.id)"
        >
          ✕
        </button>
      </div>
    </header>
    <div class="sidebar-config-items">
      <div v-if="group.itemIds.length === 0" class="empty-panel">
        {{ noItemsLabel }}
      </div>
      <template v-else>
        <SidebarItemEditor
          v-for="(itemId, itemIndex) in group.itemIds"
          :key="itemId"
          :item-id="itemId"
          :label="itemLabels[itemId]?.label ?? itemId"
          :icon="itemLabels[itemId]?.icon ?? 'tabler:circle'"
          :hidden="hiddenItemIds.includes(itemId)"
          :status-label="hiddenItemIds.includes(itemId) ? hiddenLabel : visibleDomainLabel"
          :move-target-options="groupOptions"
          :move-target-value="group.id"
          :move-target-placeholder="moveToGroupLabel"
          :show-divider-control="true"
          :divider-active="group.separatorBeforeItemIds.includes(itemId)"
          :divider-disabled="itemIndex === 0"
          :divider-label="dividerLabel"
          :show-label="showLabel"
          :hide-label="hideLabel"
          @move-to-group="(itemId, targetGroupId) => $emit('moveItemToGroup', itemId, targetGroupId)"
          @move-up="$emit('moveItem', $event, -1)"
          @move-down="$emit('moveItem', $event, 1)"
          @toggle-divider="$emit('toggleDivider', group.id, $event)"
          @toggle-hidden="$emit('toggleHidden', $event)"
        />
      </template>
    </div>
  </section>
</template>

<style scoped>
.sidebar-config-group {
  margin-bottom: 12px;
}

.sidebar-config-group > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-bottom: 1px solid var(--hh-border);
}

.sidebar-config-group > header label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.sidebar-config-group > header input {
  max-width: 180px;
  height: 28px;
  padding: 0 8px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  outline: none;
}

.sidebar-config-group > header input:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.sidebar-config-group-actions {
  display: flex;
  gap: 4px;
}

.sidebar-config-items {
  display: grid;
  gap: 4px;
  padding: 4px 0;
}
</style>
