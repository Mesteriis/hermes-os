<script setup lang="ts">
import type { CallTranscriptionSourceV1 } from './api/callTranscription'
import CallTranscriptionPanel from './presentation/CallTranscriptionPanel.vue'
import { useCallTranscription } from './queries/useCallTranscription'

const props = defineProps<{
	canTranscribe: boolean
	selectedSource?: CallTranscriptionSourceV1
}>()

const workflow = useCallTranscription(
	() => props.canTranscribe,
	() => props.selectedSource,
)
</script>

<template>
	<CallTranscriptionPanel :model="workflow.model.value" @retry="workflow.retry" />
</template>
