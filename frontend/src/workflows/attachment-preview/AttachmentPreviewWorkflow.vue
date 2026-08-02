<script setup lang="ts">
import AttachmentPreviewPanel from './presentation/AttachmentPreviewPanel.vue'
import { useAttachmentPreview } from './queries/useAttachmentPreview'

const props = defineProps<{
	canPreview: boolean
	candidateAttachmentAnchorId?: Uint8Array
}>()

const workflow = useAttachmentPreview(
	() => props.canPreview,
	() => props.candidateAttachmentAnchorId,
)
</script>

<template>
	<AttachmentPreviewPanel :model="workflow.model.value" @retry="workflow.retry" />
</template>
