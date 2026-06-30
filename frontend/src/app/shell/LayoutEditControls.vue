<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useLayoutEditControlsSurface } from '../queries/useLayoutEditControlsSurface'

const { editor, handleAddWidget, handleCancel, handleReset, handleSave } = useLayoutEditControlsSurface()
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

