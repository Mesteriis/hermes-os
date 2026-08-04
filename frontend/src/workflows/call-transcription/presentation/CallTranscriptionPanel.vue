<script setup lang="ts">
import type { CallTranscriptionPanelModel } from './callTranscriptionPanelModel'
import './callTranscriptionPanel.css'

defineProps<{ model: CallTranscriptionPanelModel }>()
const emit = defineEmits<{ retry: [] }>()
</script>

<template>
	<section class="call-transcription" :aria-busy="model.busy" aria-labelledby="call-transcription-title">
		<header>
			<div>
				<span>Consent-bound workflow</span>
				<h2 id="call-transcription-title">Call transcription</h2>
				<p>Audio and transcript bytes remain behind owner custody; only bounded status metadata uses realtime.</p>
			</div>
			<strong>{{ model.status }}</strong>
		</header>

		<p class="call-transcription__status" :role="['error', 'rejected'].includes(model.status) ? 'alert' : 'status'">
			{{ model.statusMessage }}
		</p>

		<div v-if="model.status === 'unavailable'" class="call-transcription__skeleton" aria-label="Call Transcription unavailable">
			<span />
			<span />
			<span />
		</div>
		<div v-else-if="model.busy" class="call-transcription__skeleton" aria-hidden="true">
			<span />
			<span />
			<span />
		</div>
		<div v-else-if="model.status === 'ready'" class="call-transcription__artifact">
			<p>{{ model.detectedLanguage }} · {{ model.durationLabel }}</p>
			<pre>{{ model.transcriptText }}</pre>
		</div>

		<button v-if="model.canRetry" type="button" @click="emit('retry')">Retry transcription</button>
	</section>
</template>
