<script setup lang="ts">
import { Icon } from '@iconify/vue'
import type {
  ResolvedSidebarRootEntry,
  SidebarItemId,
  SidebarRootItemId
} from '../../../../shared/stores/sidebar'
import SidebarItemEditor from './SidebarItemEditor.vue'

defineProps<{
  entries: ResolvedSidebarRootEntry[]
  hiddenItemIds: SidebarItemId[]
  rootItemCount: number
  groupOptions: Array<{ value: string; label: string }>
  rootLabel: string
  sidebarRootLabel: string
  expandableGroupLabel: string
  itemsLabel: string
  hiddenLabel: string
  rootDomainLabel: string
  moveToGroupLabel: string
  showLabel: string
  hideLabel: string
}>()

defineEmits<{
  moveGroup: [groupId: string, direction: -1 | 1]
  removeGroup: [groupId: string]
  moveRootItem: [rootId: SidebarRootItemId, direction: -1 | 1]
  moveItemToGroup: [itemId: SidebarItemId, targetGroupId: string]
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <section class="sidebar-config-group">
    <header>
      <label>
        <span>{{ rootLabel }}</span>
        <input :value="sidebarRootLabel" disabled autocomplete="off" />
      </label>
    </header>
    <div class="sidebar-config-items">
      <template v-for="(entry, rootIndex) in entries" :key="entry.rootId">
        <div v-if="entry.kind === 'group'" class="sidebar-config-item group-node">
          <div class="sidebar-config-item-main">
            <span class="round-icon green">
              <Icon :icon="entry.group.icon" aria-hidden="true" />
            </span>
            <div>
              <strong>{{ entry.group.label }}</strong>
              <small>{{ expandableGroupLabel }} · {{ entry.group.items.length }} {{ itemsLabel }}</small>
            </div>
          </div>
          <div class="sidebar-config-item-controls">
            <button
              type="button"
              class="hermes-btn hermes-btn--icon"
              :disabled="rootIndex === 0"
              @click="$emit('moveGroup', entry.group.id, -1)"
            >
              ↑
            </button>
            <button
              type="button"
              class="hermes-btn hermes-btn--icon"
              :disabled="rootIndex === rootItemCount - 1"
              @click="$emit('moveGroup', entry.group.id, 1)"
            >
              ↓
            </button>
            <button
              type="button"
              class="hermes-btn hermes-btn--icon hermes-btn--destructive"
              :disabled="entry.group.id === 'communications'"
              @click="$emit('removeGroup', entry.group.id)"
            >
              ✕
            </button>
          </div>
        </div>

        <SidebarItemEditor
          v-else
          :item-id="entry.item.itemId"
          :label="entry.item.label"
          :icon="entry.item.icon"
          :hidden="hiddenItemIds.includes(entry.item.itemId)"
          :status-label="hiddenItemIds.includes(entry.item.itemId) ? hiddenLabel : rootDomainLabel"
          :move-target-options="groupOptions"
          move-target-value="root"
          :move-target-placeholder="moveToGroupLabel"
          :show-label="showLabel"
          :hide-label="hideLabel"
          @move-to-group="(itemId, targetGroupId) => $emit('moveItemToGroup', itemId, targetGroupId)"
          @move-up="$emit('moveRootItem', entry.rootId, -1)"
          @move-down="$emit('moveRootItem', entry.rootId, 1)"
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

.sidebar-config-items {
  display: grid;
  gap: 4px;
  padding: 4px 0;
}

.sidebar-config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--hh-radius-sm);
  transition: background 100ms ease;
}

.sidebar-config-item:hover {
  background: var(--hh-hover-bg);
}

.sidebar-config-item.group-node {
  border-left: 2px solid var(--hh-accent);
}

.sidebar-config-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.sidebar-config-item-main strong {
  display: block;
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-config-item-main small {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.sidebar-config-item-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.round-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  font-size: 14px;
  flex-shrink: 0;
}

.round-icon.green {
  background: color-mix(in srgb, #22c55e 15%, transparent);
  color: #22c55e;
}
</style>
