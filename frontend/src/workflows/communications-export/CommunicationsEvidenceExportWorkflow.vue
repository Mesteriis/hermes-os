<script setup lang="ts">
import CommunicationsEvidenceExportPanel from './presentation/CommunicationsEvidenceExportPanel.vue'
import { useCommunicationsEvidenceExport } from './queries/useCommunicationsEvidenceExport'

const props = defineProps<{
	canExport: boolean
	candidateMessageId?: Uint8Array
}>()

const workflow = useCommunicationsEvidenceExport(() => props.canExport)

function addCandidate(): void {
	if (props.candidateMessageId) workflow.addMessage(props.candidateMessageId)
}
</script>

<template>
	<CommunicationsEvidenceExportPanel
		:model="workflow.model.value"
		@add-candidate="addCandidate"
		@clear="workflow.clear"
		@download="workflow.download"
		@refresh="workflow.refresh"
		@start="workflow.start"
	/>
</template>
