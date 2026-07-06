<script setup lang="ts">
import type { CommunicationOutboxItem } from '../types/communications'
import CommunicationCapabilityCard from './CommunicationCapabilityCard.vue'
import CommunicationChannelSurfaceCard from './CommunicationChannelSurfaceCard.vue'
import CommunicationOutboxStatusCard from './CommunicationOutboxStatusCard.vue'
import CommunicationThreadSignalCard from './CommunicationThreadSignalCard.vue'
import type {
  CommunicationCapabilityCardModel,
  CommunicationChannelSurfaceCardModel,
  CommunicationThreadSignalCardModel
} from './communicationDomainElements'
import './communicationDomainElements.css'

defineProps<{
  title: string
  description: string
  channels: readonly CommunicationChannelSurfaceCardModel[]
  capabilities: readonly CommunicationCapabilityCardModel[]
  threads: readonly CommunicationThreadSignalCardModel[]
  outboxItems: readonly CommunicationOutboxItem[]
  now: string
}>()
</script>

<template>
	<section class="communication-domain-overview">
		<header class="communication-domain-overview__header">
			<h2>{{ title }}</h2>
			<p>{{ description }}</p>
		</header>

		<section
			v-if="channels.length > 0"
			class="communication-domain-overview__section"
			aria-labelledby="communication-channels-title"
		>
			<h3 id="communication-channels-title">Channel surfaces</h3>
			<p>Provider channels stay under one Communications product surface.</p>
			<div class="communication-domain-overview__grid">
				<CommunicationChannelSurfaceCard
					v-for="channel in channels"
					:key="channel.channelId"
					:surface="channel"
				/>
			</div>
		</section>

		<section
			v-if="capabilities.length > 0"
			class="communication-domain-overview__section"
			aria-labelledby="communication-capabilities-title"
		>
			<h3 id="communication-capabilities-title">Workspace tools</h3>
			<p>Communication actions stay grouped by owner workflow, not by transport details.</p>
			<div class="communication-domain-overview__grid">
				<CommunicationCapabilityCard
					v-for="capability in capabilities"
					:key="capability.id"
					:capability="capability"
				/>
			</div>
		</section>

		<section
			v-if="threads.length > 0"
			class="communication-domain-overview__section"
			aria-labelledby="communication-threads-title"
		>
			<h3 id="communication-threads-title">Thread signals</h3>
			<p>Fresh events can signal review pressure without becoming durable truth by themselves.</p>
			<div class="communication-domain-overview__wide-grid">
				<CommunicationThreadSignalCard
					v-for="thread in threads"
					:key="thread.thread.thread_id"
					:model="thread"
				/>
			</div>
		</section>

		<section
			v-if="outboxItems.length > 0"
			class="communication-domain-overview__section"
			aria-labelledby="communication-outbox-title"
		>
			<h3 id="communication-outbox-title">Outbox state</h3>
			<p>Provider writes stay observable through durable outbox and delivery state.</p>
			<div class="communication-domain-overview__wide-grid">
				<CommunicationOutboxStatusCard
					v-for="item in outboxItems"
					:key="item.outbox_id"
					:item="item"
					:now="now"
				/>
			</div>
		</section>
	</section>
</template>
