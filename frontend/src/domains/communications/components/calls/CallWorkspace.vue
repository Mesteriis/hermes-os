<script setup lang="ts">
import { ref } from 'vue'
import '../communicationDomainElements.css'
import type { CommunicationCallItemModel, CommunicationCallsSurfaceModel } from '../communicationDomainElements'
import CallInspector from './CallInspector.vue'
import CallList from './CallList.vue'
import CallMessage from './CallMessage.vue'

defineProps<{
  surface: CommunicationCallsSurfaceModel
}>()

const emit = defineEmits<{
  'select-call': [item: CommunicationCallItemModel]
}>()

const isInspectorVisible = ref(true)

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
}
</script>

<template>
	<section
		:class="[
			'communication-calls-workspace',
			!isInspectorVisible && 'communication-calls-workspace--inspector-hidden'
		]"
		aria-label="Calls workspace"
	>
		<CallList
			:provider-value="surface.providerValue"
			:provider-options="surface.providerOptions"
			:permanent-meetings="surface.permanentMeetings"
			:calls="surface.calls"
			@select="emit('select-call', $event)"
		/>
		<section class="communication-calls-workspace__reader" aria-label="Open call">
			<CallMessage
				:surface="surface"
				:inspector-visible="isInspectorVisible"
				@toggle-inspector="handleToggleInspector"
			/>
		</section>
		<CallInspector v-if="isInspectorVisible" :model="surface.inspector" />
	</section>
</template>
