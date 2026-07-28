<script setup lang="ts">
import type { CommunicationsEvidenceExportPanelModel } from './communicationsEvidenceExportPanelModel'
import './communicationsEvidenceExportPanel.css'

defineProps<{ model: CommunicationsEvidenceExportPanelModel }>()

const emit = defineEmits<{
	addCandidate: []
	clear: []
	download: []
	refresh: []
	start: []
}>()
</script>

<template>
	<section class="communications-evidence-export" aria-labelledby="communications-evidence-export-title">
		<header>
			<div>
				<span>Owner-local workflow</span>
				<h2 id="communications-evidence-export-title">Evidence export</h2>
				<p>Build a provider-neutral JSONL artifact from explicit canonical messages.</p>
			</div>
			<strong>{{ model.progressLabel }}</strong>
		</header>

		<p
			v-if="model.statusMessage"
			class="communications-evidence-export__status"
			:role="model.status === 'error' || model.status === 'rejected' ? 'alert' : 'status'"
		>{{ model.statusMessage }}</p>

		<div class="communications-evidence-export__actions">
			<button
				type="button"
				:disabled="!model.canAddCandidate || model.busy"
				@click="emit('addCandidate')"
			>Add open message</button>
			<button
				type="button"
				:disabled="model.selectedCount === 0 || model.busy"
				@click="emit('clear')"
			>Clear selection</button>
			<button
				type="button"
				:disabled="model.selectedCount === 0 || model.busy || !model.available"
				@click="emit('start')"
			>Prepare export</button>
			<button
				v-if="model.canRefresh"
				type="button"
				@click="emit('refresh')"
			>Refresh status</button>
			<button
				v-if="model.canDownload"
				type="button"
				class="communications-evidence-export__download"
				@click="emit('download')"
			>Download JSONL</button>
		</div>
	</section>
</template>
