<script setup lang="ts">
import { ref } from 'vue'
import CommunicationHermesInspector from '../CommunicationHermesInspector.vue'
import type {
  CommunicationConversationModel,
  CommunicationHermesInspectorSectionModel
} from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailList from './MailList.vue'
import MailThread from './MailThread.vue'
import type { MailListItemModel } from './mailElements'

defineProps<{
  items: readonly MailListItemModel[]
  conversation: CommunicationConversationModel
	inspectorSections: readonly CommunicationHermesInspectorSectionModel[]
}>()

const isInspectorVisible = ref(true)

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
}
</script>

<template>
	<section class="communication-workspace-shell">
		<MailList :items="items" />
		<MailThread
			:conversation="conversation"
			:inspector-visible="isInspectorVisible"
			@toggle-inspector="handleToggleInspector"
		/>
		<CommunicationHermesInspector v-if="isInspectorVisible" :sections="inspectorSections" />
	</section>
</template>
