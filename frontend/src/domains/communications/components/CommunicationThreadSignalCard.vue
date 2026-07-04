<script setup lang="ts">
import { computed } from 'vue'
import { Badge, Card, CardContent, CardHeader, ProviderIcon } from '@/shared/ui'
import type { CommunicationThreadSignalCardModel } from './communicationDomainElements'
import { communicationThreadCardPresentation } from './communicationDomainElements'
import './communicationDomainElements.css'

const props = defineProps<{
  model: CommunicationThreadSignalCardModel
}>()

const presentation = computed(() => communicationThreadCardPresentation(props.model))
</script>

<template>
	<Card
		as="article"
		class="communication-domain-card"
		variant="interactive"
		density="compact"
		:signal="presentation.signal"
		:signal-tone="presentation.signalTone"
	>
		<CardHeader>
			<div class="communication-domain-card__header">
				<div class="communication-domain-card__identity">
					<ProviderIcon :provider="presentation.channelIcon" :label="presentation.channelLabel" />
					<div class="communication-domain-card__title-group">
						<p class="communication-domain-card__meta">{{ presentation.channelLabel }}</p>
						<h3 class="communication-domain-card__subject">{{ model.thread.subject }}</h3>
					</div>
				</div>
				<Badge :variant="presentation.status.badgeTone">{{ presentation.status.label }}</Badge>
			</div>
		</CardHeader>
		<CardContent>
			<div class="communication-domain-card__body">
				<p v-if="model.preview" class="communication-domain-card__preview">{{ model.preview }}</p>
				<p v-if="model.participantPreview" class="communication-domain-card__meta">
					{{ model.participantPreview }}
				</p>
				<div class="communication-domain-card__stats">
					<div
						v-for="fact in presentation.facts"
						:key="fact.label"
						class="communication-domain-card__stat"
					>
						<strong>{{ fact.value }}</strong>
						<span>{{ fact.label }}</span>
					</div>
				</div>
			</div>
		</CardContent>
	</Card>
</template>
