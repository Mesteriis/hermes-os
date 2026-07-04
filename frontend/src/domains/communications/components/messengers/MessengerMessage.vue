<script setup lang="ts">
import '../communicationDomainElements.css'
import MessengerAction from './MessengerAction.vue'
import type { MessengerConversationModel } from './messengerElements'
import MessengerViewer from './MessengerViewer.vue'

withDefaults(defineProps<{
  conversation: MessengerConversationModel
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
	<article class="messenger-message">
		<MessengerAction
			:inspector-visible="inspectorVisible"
			:show-inspector-toggle="showInspectorToggle"
			@toggle-inspector="emit('toggle-inspector')"
		/>
		<MessengerViewer :conversation="conversation" />
	</article>
</template>
