<script setup lang="ts">
import { computed } from 'vue'
import { Badge, Card, CardContent, CardHeader, ProviderIcon, StatusIcon } from '@/shared/ui'
import type { CommunicationChannelSurfaceCardModel } from './communicationDomainElements'
import {
  communicationChannelProviderIcon,
  communicationSurfaceStatusPresentation
} from './communicationDomainElements'
import './communicationDomainElements.css'

const props = defineProps<{
  surface: CommunicationChannelSurfaceCardModel
}>()

const status = computed(() => communicationSurfaceStatusPresentation(props.surface.status))
const provider = computed(() => communicationChannelProviderIcon(props.surface.channelId))
</script>

<template>
	<Card
		as="section"
		class="communication-domain-card"
		variant="raised"
		density="compact"
		:signal="surface.status === 'blocked'"
		:signal-tone="status.signalTone"
	>
		<CardHeader>
			<div class="communication-domain-card__header">
				<div class="communication-domain-card__identity">
					<ProviderIcon :provider="provider" :label="surface.label" />
					<div class="communication-domain-card__title-group">
						<h3 class="communication-domain-card__title">{{ surface.label }}</h3>
						<p v-if="surface.accountCountLabel" class="communication-domain-card__subtitle">
							{{ surface.accountCountLabel }}
						</p>
					</div>
				</div>
				<div class="communication-domain-card__status">
					<Badge :variant="status.badgeTone">{{ status.label }}</Badge>
				</div>
			</div>
		</CardHeader>
		<CardContent>
			<div class="communication-domain-card__body">
				<p class="communication-domain-card__description">{{ surface.description }}</p>
				<div v-if="surface.metricItems?.length" class="communication-domain-card__stats">
					<div
						v-for="metric in surface.metricItems"
						:key="metric.label"
						class="communication-domain-card__stat"
					>
						<strong>{{ metric.value }}</strong>
						<span>{{ metric.label }}</span>
					</div>
				</div>
				<ul v-if="surface.capabilityLabels?.length" class="communication-domain-card__list">
					<li
						v-for="capability in surface.capabilityLabels"
						:key="capability"
						class="communication-domain-card__list-item"
					>
						{{ capability }}
					</li>
				</ul>
				<div v-if="surface.lastActivityLabel" class="communication-domain-card__footer">
					<StatusIcon :status="status.statusIcon" :label="status.label" size="1rem" />
					<span>{{ surface.lastActivityLabel }}</span>
				</div>
			</div>
		</CardContent>
	</Card>
</template>
