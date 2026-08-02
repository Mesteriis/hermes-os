<script setup lang="ts">
import type { CanonicalCommunicationDetailModel } from './canonicalCommunicationDetailModel'
import './canonicalCommunicationDetail.css'

defineProps<{ model: CanonicalCommunicationDetailModel }>()

const emit = defineEmits<{
	close: []
	loadMoreAttachments: []
	loadMoreEvidence: []
	loadMoreParticipants: []
	loadMoreReferences: []
	selectAttachment: [attachmentKey: string]
}>()
</script>

<template>
	<aside
		v-if="model.status !== 'idle'"
		class="canonical-communication-detail"
		:aria-busy="model.status === 'loading'"
		aria-labelledby="canonical-communication-detail-title"
	>
		<header>
			<div>
				<span>Canonical detail</span>
				<h2 id="canonical-communication-detail-title">
					{{ model.messageLabel || 'Loading message…' }}
				</h2>
				<p>{{ model.conversationLabel }}</p>
			</div>
			<button type="button" aria-label="Close canonical message detail" @click="emit('close')">
				Close
			</button>
		</header>

		<p v-if="model.statusMessage" class="canonical-communication-detail__status" role="status">
			{{ model.statusMessage }}
		</p>

		<template v-if="model.status === 'ready'">
			<div class="canonical-communication-detail__facts">
				<span>{{ model.directionLabel }}</span>
				<span>{{ model.bodyStateLabel }}</span>
				<span>{{ model.lifecycleLabel }}</span>
				<span>{{ model.observedRangeLabel }}</span>
			</div>

			<div class="canonical-communication-detail__grid">
				<section>
					<h3>Participants <span>{{ model.participants.length }}</span></h3>
					<ul>
						<li v-for="row in model.participants" :key="row.key">
							<strong>{{ row.primaryLabel }}</strong>
							<span>{{ row.secondaryLabel }}</span>
							<small>{{ row.metaLabel }}</small>
						</li>
					</ul>
					<button
						v-if="model.hasMoreParticipants"
						type="button"
						:disabled="model.loadingMore"
						@click="emit('loadMoreParticipants')"
					>Load more participants</button>
				</section>

				<section>
					<h3>Attachments <span>{{ model.attachments.length }}</span></h3>
					<ul>
						<li v-for="row in model.attachments" :key="row.key">
							<button
								type="button"
								:disabled="!row.previewEligible"
								@click="emit('selectAttachment', row.key)"
							>
								<strong>{{ row.primaryLabel }}</strong>
								<span>{{ row.secondaryLabel }}</span>
								<small>{{ row.metaLabel }}</small>
								<em>{{ row.previewLabel }}</em>
							</button>
						</li>
					</ul>
					<button
						v-if="model.hasMoreAttachments"
						type="button"
						:disabled="model.loadingMore"
						@click="emit('loadMoreAttachments')"
					>Load more attachments</button>
				</section>

				<section>
					<h3>References <span>{{ model.references.length }}</span></h3>
					<ul>
						<li v-for="row in model.references" :key="row.key">
							<strong>{{ row.primaryLabel }}</strong>
							<span>{{ row.secondaryLabel }}</span>
							<small>{{ row.metaLabel }}</small>
						</li>
					</ul>
					<button
						v-if="model.hasMoreReferences"
						type="button"
						:disabled="model.loadingMore"
						@click="emit('loadMoreReferences')"
					>Load more references</button>
				</section>

				<section>
					<h3>Evidence history <span>{{ model.evidence.length }}</span></h3>
					<ul>
						<li v-for="row in model.evidence" :key="row.key">
							<strong>{{ row.primaryLabel }}</strong>
							<span>{{ row.secondaryLabel }}</span>
							<small>{{ row.metaLabel }}</small>
						</li>
					</ul>
					<button
						v-if="model.hasMoreEvidence"
						type="button"
						:disabled="model.loadingMore"
						@click="emit('loadMoreEvidence')"
					>Load more evidence</button>
				</section>
			</div>
		</template>
	</aside>
</template>
