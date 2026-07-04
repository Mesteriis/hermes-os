<script setup lang="ts">
import { ref } from 'vue'
import '../communicationDomainElements.css'
import MessengerInspector from './MessengerInspector.vue'
import MessengerList from './MessengerList.vue'
import MessengerMessage from './MessengerMessage.vue'
import type { MessengerConversationModel, MessengerInspectorModel, MessengerListItemModel } from './messengerElements'

defineProps<{
  items: readonly MessengerListItemModel[]
  conversation: MessengerConversationModel
  inspector: MessengerInspectorModel
}>()

const emit = defineEmits<{
  'select-conversation': [item: MessengerListItemModel]
}>()

const isInspectorVisible = ref(true)

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
}
</script>

<template>
	<section
		:class="[
			'communication-workspace-shell communication-workspace-shell--messenger',
			!isInspectorVisible && 'communication-workspace-shell--messenger-inspector-hidden'
		]"
	>
		<MessengerList
			:items="items"
			:selected-id="conversation.id"
			@select="emit('select-conversation', $event)"
		/>
		<section class="communication-messenger-workspace-reader" aria-label="Open dialog">
			<MessengerMessage
				:conversation="conversation"
				:inspector-visible="isInspectorVisible"
				@toggle-inspector="handleToggleInspector"
			/>
		</section>
		<MessengerInspector v-if="isInspectorVisible" :model="inspector" />
	</section>
</template>
