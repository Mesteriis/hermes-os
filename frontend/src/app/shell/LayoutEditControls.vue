<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useLayoutEditorStore } from '../../shared/stores/layoutEditor'

const editor = useLayoutEditorStore()

function handleAddWidget(): void {
  editor.openAddWidgetDrawer()
}

function handleCancel(): void {
  editor.cancelLayoutEditing()
}

function handleReset(): void {
  editor.resetCurrentViewLayout()
}

function handleSave(): void {
  editor.saveLayoutSettings()
}
</script>

<template>
  <div v-if="editor.isLayoutEditing" class="layout-edit-controls">
    <button class="layout-edit-btn" @click="handleAddWidget">
      <Icon icon="tabler:plus" class="layout-edit-btn-icon" />
      Add Widget
    </button>
    <button class="layout-edit-btn layout-edit-btn-secondary" @click="handleCancel">
      Cancel
    </button>
    <button class="layout-edit-btn layout-edit-btn-secondary" @click="handleReset">
      Reset
    </button>
    <button class="layout-edit-btn layout-edit-btn-primary" @click="handleSave">
      <Icon icon="tabler:device-floppy" class="layout-edit-btn-icon" />
      Save
    </button>
  </div>
</template>

<style scoped>
.layout-edit-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1.25rem;
  background: var(--hh-panel-bg);
  border-bottom: 1px solid var(--hh-accent);
  flex-shrink: 0;
}

.layout-edit-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: 0.375rem;
  border: 1px solid var(--hh-border);
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
  cursor: pointer;
  transition: all 150ms ease;
}

.layout-edit-btn:hover {
  background: var(--hh-active-bg);
}

.layout-edit-btn-primary {
  background: var(--hh-accent);
  color: var(--hh-bg);
  border-color: var(--hh-accent);
}

.layout-edit-btn-primary:hover {
  filter: brightness(1.1);
}

.layout-edit-btn-secondary {
  border-color: transparent;
  background: transparent;
}

.layout-edit-btn-icon {
  width: 1rem;
  height: 1rem;
}
</style>
