<script setup lang="ts">
import { ref } from 'vue'
import '../communicationDomainElements.css'
import type {
  CommunicationChannelDirectChatModel,
  CommunicationChannelRoomModel,
  CommunicationChannelWorkspaceModel
} from '../communicationDomainElements'
import ChannelInspector from './ChannelInspector.vue'
import ChannelList from './ChannelList.vue'
import ChannelMessage from './ChannelMessage.vue'

defineProps<{
  workspace: CommunicationChannelWorkspaceModel
}>()

const emit = defineEmits<{
  'select-room': [room: CommunicationChannelRoomModel]
  'select-direct-chat': [chat: CommunicationChannelDirectChatModel]
}>()

const isInspectorVisible = ref(true)

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
}
</script>

<template>
	<section
		:class="[
			'communication-channel-workspace',
			!isInspectorVisible && 'communication-channel-workspace--inspector-hidden'
		]"
		aria-label="Channels workspace"
	>
		<ChannelList
			:provider-value="workspace.providerValue"
			:provider-options="workspace.providerOptions"
			:rooms="workspace.rooms"
			:direct-chat-folders="workspace.directChatFolders"
			@select="emit('select-room', $event)"
			@select-direct-chat="emit('select-direct-chat', $event)"
		/>
		<ChannelMessage
			:workspace="workspace"
			:inspector-visible="isInspectorVisible"
			@toggle-inspector="handleToggleInspector"
		/>
		<ChannelInspector v-if="isInspectorVisible" :model="workspace.inspector" />
	</section>
</template>
