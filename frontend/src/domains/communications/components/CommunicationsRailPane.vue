<script setup lang="ts">
import CommunicationsContextInspector from './CommunicationsContextInspector.vue'
import CommunicationsContextRail from './CommunicationsContextRail.vue'
import type { InspectorMode, CommunicationMessageDetailResponse, ProjectItem, TaskItem } from '../types/communications'

defineProps<{
  detail: CommunicationMessageDetailResponse | null
  inspectorMode: InspectorMode
  projects: ProjectItem[]
  tasks: TaskItem[]
}>()

const emit = defineEmits<{
  'update:inspectorMode': [mode: InspectorMode]
}>()
</script>

<template>
  <aside class="communications-rail-pane">
    <CommunicationsContextInspector
      v-if="detail"
      :detail="detail"
      :inspector-mode="inspectorMode"
      @update:inspector-mode="emit('update:inspectorMode', $event)"
    />
    <CommunicationsContextRail v-else :detail="detail" :projects="projects" :tasks="tasks" />
  </aside>
</template>

<style scoped>
.communications-rail-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
}
</style>
