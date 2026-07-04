<script setup lang="ts">
import { Badge, EntityIcon, Icon, ProviderIcon } from '@/shared/ui'
import type { CommunicationCallsSurfaceModel } from './communicationDomainElements'
import {
  communicationCallStateTone,
  communicationChannelLabel,
  communicationChannelProviderIcon
} from './communicationDomainElements'
import './communicationDomainElements.css'

defineProps<{
  surface: CommunicationCallsSurfaceModel
}>()
</script>

<template>
	<section class="communication-calls-surface">
		<aside class="communication-calls-list" aria-label="Calls">
			<header class="communication-workspace-panel__header">
				<h2 class="communication-workspace-panel__title">{{ surface.title }}</h2>
				<p class="communication-workspace-panel__meta">{{ surface.subtitle }}</p>
			</header>
			<div class="communication-calls-list__body">
				<button
					v-for="call in surface.calls"
					:key="call.id"
					type="button"
					:class="['communication-call-item', call.selected && 'communication-call-item--selected']"
				>
					<div class="communication-inbox-item__identity">
						<ProviderIcon
							:provider="communicationChannelProviderIcon(call.channelKind)"
							:label="communicationChannelLabel(call.channelKind)"
						/>
						<div class="communication-inbox-item__title-block">
							<h3 class="communication-inbox-item__title">{{ call.title }}</h3>
							<p class="communication-inbox-item__meta">{{ call.participants }}</p>
						</div>
					</div>
					<p class="communication-inbox-item__preview">{{ call.summary }}</p>
					<div class="communication-inbox-item__badges">
						<Badge :variant="communicationCallStateTone(call.state)">{{ call.state }}</Badge>
						<Badge variant="neutral">{{ call.durationLabel }}</Badge>
						<span class="communication-inbox-item__meta">{{ call.startedAt }}</span>
					</div>
				</button>
			</div>
		</aside>

		<section class="communication-call-detail" aria-label="Call detail">
			<header class="communication-workspace-panel__header">
				<div class="communication-workspace-panel__title-row">
					<div>
						<h2 class="communication-conversation__title">Call transcript</h2>
						<p class="communication-conversation__subtitle">Moments, speaker turns and extracted follow-ups.</p>
					</div>
					<Icon icon="tabler:phone-call" size="1.25rem" />
				</div>
			</header>
			<div class="communication-call-timeline">
				<article
					v-for="moment in surface.moments"
					:key="moment.id"
					class="communication-call-moment"
				>
					<span class="communication-call-moment__time">{{ moment.timestamp }}</span>
					<div class="communication-call-moment__body">
						<strong>{{ moment.speaker }}</strong>
						<p>{{ moment.text }}</p>
					</div>
					<Badge v-if="moment.tone" :variant="moment.tone">{{ moment.tone }}</Badge>
				</article>
			</div>
		</section>

		<aside class="communication-workspace-panel communication-workspace-panel--inspector" aria-label="Call inspector">
			<header class="communication-workspace-panel__header">
				<h2 class="communication-workspace-panel__title">Hermes inspector</h2>
				<p class="communication-workspace-panel__meta">Calls become evidence before they become obligations.</p>
			</header>
			<div class="communication-inspector">
				<section
					v-for="section in surface.inspectorSections"
					:key="section.id"
					class="communication-inspector-section"
				>
					<h3 class="communication-inspector-section__title">{{ section.title }}</h3>
					<article
						v-for="item in section.items"
						:key="item.id"
						class="communication-inspector-entity"
					>
						<div class="communication-inspector-entity__identity">
							<EntityIcon :entity="item.entity" :label="item.title" />
							<div class="communication-inspector-entity__body">
								<h4 class="communication-inspector-entity__title">{{ item.title }}</h4>
								<p class="communication-inspector-entity__description">{{ item.description }}</p>
								<p class="communication-inspector-entity__evidence">{{ item.evidenceLabel }}</p>
							</div>
						</div>
					</article>
				</section>
			</div>
		</aside>
	</section>
</template>
