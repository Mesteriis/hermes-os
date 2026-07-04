<script setup lang="ts">
import { computed } from 'vue'
import { Badge, Card, CardContent, CardHeader, Icon, StatusIcon } from '@/shared/ui'
import type { CommunicationOutboxItem } from '../types/communications'
import { communicationOutboxCardPresentation } from './communicationDomainElements'
import './communicationDomainElements.css'

const props = defineProps<{
  item: CommunicationOutboxItem
  now: string
}>()

const presentation = computed(() => communicationOutboxCardPresentation(props.item, new Date(props.now)))
</script>

<template>
	<Card
		as="article"
		class="communication-domain-card"
		variant="muted"
		density="compact"
		:signal="item.status === 'failed'"
		signal-tone="danger"
	>
		<CardHeader>
			<div class="communication-domain-card__header">
				<div class="communication-domain-card__identity">
					<StatusIcon :status="presentation.statusIcon" :label="presentation.title" />
					<div class="communication-domain-card__title-group">
						<h3 class="communication-domain-card__title">{{ presentation.title }}</h3>
						<p class="communication-domain-card__subtitle">{{ item.subject }}</p>
					</div>
				</div>
				<Badge :variant="presentation.badgeTone">{{ item.status }}</Badge>
			</div>
		</CardHeader>
		<CardContent>
			<div class="communication-domain-card__body">
				<p class="communication-domain-card__description">{{ presentation.detail }}</p>
				<div class="communication-domain-card__footer">
					<Icon icon="tabler:send-2" size="1rem" />
					<span>{{ item.to_recipients.length }} recipients</span>
					<span v-if="presentation.canUndo">Undo available</span>
				</div>
			</div>
		</CardContent>
	</Card>
</template>
