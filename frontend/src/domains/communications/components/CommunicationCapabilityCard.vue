<script setup lang="ts">
import { computed } from 'vue'
import { Badge, Card, CardContent, CardHeader, Icon, StatusIcon } from '@/shared/ui'
import type { CommunicationCapabilityCardModel } from './communicationDomainElements'
import { communicationCapabilityStatusPresentation } from './communicationDomainElements'
import './communicationDomainElements.css'

const props = defineProps<{
  capability: CommunicationCapabilityCardModel
}>()

const status = computed(() => communicationCapabilityStatusPresentation(props.capability.status))
</script>

<template>
	<Card
		as="section"
		class="communication-domain-card"
		variant="default"
		density="compact"
		:signal="capability.status === 'blocked'"
		:signal-tone="status.signalTone"
	>
		<CardHeader>
			<div class="communication-domain-card__header">
				<div class="communication-domain-card__identity">
					<StatusIcon :status="status.statusIcon" :label="status.label" />
					<div class="communication-domain-card__title-group">
						<h3 class="communication-domain-card__title">{{ capability.title }}</h3>
						<p class="communication-domain-card__subtitle">{{ capability.surfaceLabel }}</p>
					</div>
				</div>
				<Badge :variant="status.badgeTone">{{ status.label }}</Badge>
			</div>
		</CardHeader>
		<CardContent>
			<div class="communication-domain-card__body">
				<p class="communication-domain-card__description">{{ capability.description }}</p>
				<div v-if="capability.metricItems?.length" class="communication-domain-card__stats">
					<div
						v-for="metric in capability.metricItems"
						:key="metric.label"
						class="communication-domain-card__stat"
					>
						<strong>{{ metric.value }}</strong>
						<span>{{ metric.label }}</span>
					</div>
				</div>
				<div class="communication-domain-card__footer">
					<Icon :icon="capability.icon" size="1rem" />
					<span>Communications workspace surface</span>
				</div>
			</div>
		</CardContent>
	</Card>
</template>
