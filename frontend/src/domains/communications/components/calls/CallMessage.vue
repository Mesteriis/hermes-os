<script setup lang="ts">
import '../communicationDomainElements.css'
import type { CommunicationCallsSurfaceModel } from '../communicationDomainElements'
import CallAction from './CallAction.vue'
import CallViewer from './CallViewer.vue'

withDefaults(defineProps<{
  surface: CommunicationCallsSurfaceModel
  inspectorVisible?: boolean
  showInspectorToggle?: boolean
}>(), {
  inspectorVisible: true,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'toggle-inspector': []
}>()
</script>

<template>
	<article class="call-message">
		<CallAction
			:action-groups="surface.actionGroups"
			:inspector-visible="inspectorVisible"
			:show-inspector-toggle="showInspectorToggle"
			@toggle-inspector="emit('toggle-inspector')"
		/>
		<CallViewer :active-call="surface.activeCall" />
	</article>
</template>
