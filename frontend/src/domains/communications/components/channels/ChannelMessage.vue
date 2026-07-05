<script setup lang="ts">
import '../communicationDomainElements.css'
import type { CommunicationChannelWorkspaceModel } from '../communicationDomainElements'
import ChannelAction from './ChannelAction.vue'
import ChannelViewer from './ChannelViewer.vue'

withDefaults(defineProps<{
  workspace: CommunicationChannelWorkspaceModel
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
	<article class="channel-message">
		<ChannelAction
			:action-groups="workspace.actionGroups"
			:inspector-visible="inspectorVisible"
			:show-inspector-toggle="showInspectorToggle"
			@toggle-inspector="emit('toggle-inspector')"
		/>
		<ChannelViewer :workspace="workspace" />
	</article>
</template>
