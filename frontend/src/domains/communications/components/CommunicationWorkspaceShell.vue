<script setup lang="ts">
import CommunicationConversationPane from './CommunicationConversationPane.vue'
import CommunicationHermesInspector from './CommunicationHermesInspector.vue'
import CommunicationInboxList from './CommunicationInboxList.vue'
import type {
  CommunicationConversationModel,
  CommunicationHermesInspectorSectionModel,
  CommunicationInboxItemModel
} from './communicationDomainElements'
import './communicationDomainElements.css'

defineProps<{
  inboxItems: readonly CommunicationInboxItemModel[]
  conversation: CommunicationConversationModel
  inspectorSections: readonly CommunicationHermesInspectorSectionModel[]
}>()
</script>

<template>
	<section class="communication-workspace-shell">
		<CommunicationInboxList :items="inboxItems" />
		<CommunicationConversationPane :conversation="conversation" />
		<CommunicationHermesInspector :sections="inspectorSections" />
	</section>
</template>
