<script setup lang="ts">
import { AttachmentChip, Badge, Icon } from '@/shared/ui'
import '../communicationDomainElements.css'
import {
  communicationCallProviderIconName,
  type CommunicationCallActiveModel
} from '../communicationDomainElements'

defineProps<{
  activeCall: CommunicationCallActiveModel
}>()
</script>

<template>
	<section class="communication-call-viewer" aria-label="Call viewer">
		<header class="communication-call-viewer__header">
			<div class="communication-call-viewer__identity">
				<span class="communication-call-viewer__provider">
					<Icon :icon="communicationCallProviderIconName(activeCall.providerKind)" size="1.2rem" />
				</span>
				<div class="communication-call-viewer__title-block">
					<h2 class="communication-conversation__title">{{ activeCall.title }}</h2>
					<p class="communication-conversation__subtitle">
						{{ activeCall.providerLabel }} · {{ activeCall.subtitle }}
					</p>
				</div>
			</div>
			<Badge :variant="activeCall.statusTone">{{ activeCall.statusLabel }}</Badge>
			<p class="communication-call-viewer__summary">{{ activeCall.summary }}</p>
		</header>

		<section class="communication-call-viewer__facts" aria-label="Call facts">
			<Badge variant="neutral">{{ activeCall.startedAtLabel }}</Badge>
			<Badge variant="neutral">{{ activeCall.durationLabel }}</Badge>
			<Badge variant="info">{{ activeCall.participantCountLabel }}</Badge>
			<Badge v-if="activeCall.recurrenceLabel" variant="accent">{{ activeCall.recurrenceLabel }}</Badge>
			<Badge variant="warning">{{ activeCall.recordingStatusLabel }}</Badge>
			<Badge variant="success">{{ activeCall.transcriptStatusLabel }}</Badge>
		</section>

		<div class="communication-call-viewer__body">
			<section class="communication-call-transcript" aria-label="Call transcript">
				<article
					v-for="moment in activeCall.moments"
					:key="moment.id"
					class="communication-call-moment"
				>
					<span class="communication-call-moment__time">{{ moment.timestamp }}</span>
					<div class="communication-call-moment__body">
						<strong>{{ moment.speaker }}</strong>
						<p>{{ moment.text }}</p>
						<small v-if="moment.evidenceLabel">{{ moment.evidenceLabel }}</small>
					</div>
					<Badge v-if="moment.tone" :variant="moment.tone">{{ moment.tone }}</Badge>
				</article>
			</section>
		</div>

		<footer class="communication-call-recordings" aria-label="Call recordings">
			<strong class="communication-call-recordings__title">
				<Icon icon="tabler:record-mail" size="1rem" />
				Recordings
			</strong>
			<div class="communication-call-recordings__items">
				<AttachmentChip
					v-for="recording in activeCall.recordings"
					:key="recording.id"
					:name="recording.title"
					:meta="`${recording.meta} · ${recording.statusLabel}`"
					:icon="recording.icon"
					:tone="recording.tone"
				/>
			</div>
		</footer>
	</section>
</template>
