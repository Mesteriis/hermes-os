<script setup lang="ts">
import { Icon } from '@iconify/vue'
import type { SidebarItemId } from '../../../../shared/stores/sidebar'

defineProps<{
  itemId: SidebarItemId
  label: string
  icon: string
  hidden: boolean
  statusLabel: string
  moveTargetOptions?: Array<{ value: string; label: string }>
  moveTargetValue?: string
  moveTargetPlaceholder?: string
  showDividerControl?: boolean
  dividerActive?: boolean
  dividerDisabled?: boolean
  dividerLabel?: string
  showLabel: string
  hideLabel: string
}>()

defineEmits<{
  moveToGroup: [itemId: SidebarItemId, targetGroupId: string]
  moveUp: [itemId: SidebarItemId]
  moveDown: [itemId: SidebarItemId]
  toggleDivider: [itemId: SidebarItemId]
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <div class="sidebar-config-item" :class="{ hidden }">
    <div class="sidebar-config-item-main">
      <span class="round-icon cyan">
        <Icon :icon="icon" aria-hidden="true" />
      </span>
      <div>
        <strong>{{ label }}</strong>
        <small>{{ statusLabel }}</small>
      </div>
    </div>
    <div class="sidebar-config-item-controls">
      <select
        v-if="moveTargetOptions"
        class="hermes-select-control"
        :value="moveTargetValue"
        @change="$emit('moveToGroup', itemId, ($event.target as HTMLSelectElement).value)"
      >
        <option v-if="moveTargetPlaceholder" :value="moveTargetValue" disabled>
          {{ moveTargetPlaceholder }}
        </option>
        <option v-for="option in moveTargetOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
      <button type="button" class="hermes-btn hermes-btn--icon" @click="$emit('moveUp', itemId)">
        ↑
      </button>
      <button type="button" class="hermes-btn hermes-btn--icon" @click="$emit('moveDown', itemId)">
        ↓
      </button>
      <button
        v-if="showDividerControl"
        type="button"
        class="hermes-btn hermes-btn--icon"
        :class="{ active: dividerActive }"
        :disabled="dividerDisabled"
        @click="$emit('toggleDivider', itemId)"
      >
        {{ dividerLabel }}
      </button>
      <button
        type="button"
        class="hermes-btn hermes-btn--icon"
        :class="{ active: !hidden }"
        @click="$emit('toggleHidden', itemId)"
      >
        {{ hidden ? showLabel : hideLabel }}
      </button>
    </div>
  </div>
</template>

<style scoped>
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

.sidebar-config-item.hidden {
  opacity: 0.5;
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

.round-icon.cyan {
  background: color-mix(in srgb, var(--hh-accent) 15%, transparent);
  color: var(--hh-accent);
}
</style>
