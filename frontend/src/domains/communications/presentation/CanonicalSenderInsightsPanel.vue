<script setup lang="ts">
import type { CanonicalSenderInsightsPanelModel } from './canonicalSenderInsightsPanelModel'
import './canonicalSenderInsightsPanel.css'

defineProps<{ model: CanonicalSenderInsightsPanelModel }>()

const emit = defineEmits<{
	loadMore: []
	retry: []
	updateScopeCurrentAccount: [value: boolean]
}>()
</script>

<template>
	<section class="canonical-sender-insights" aria-labelledby="canonical-sender-insights-title">
		<header>
			<div>
				<h2 id="canonical-sender-insights-title">Sender insights</h2>
				<p>Provider-neutral activity derived from incoming canonical evidence.</p>
			</div>
			<label>
				<input
					type="checkbox"
					:checked="model.scopeCurrentAccount"
					:disabled="!model.canScopeToCurrentAccount || model.busy"
					@change="emit(
						'updateScopeCurrentAccount',
						($event.target as HTMLInputElement).checked,
					)"
				>
				Selected source
			</label>
		</header>

		<div
			v-if="model.status === 'unavailable'"
			class="canonical-sender-insights__state"
			role="status"
		>
			{{ model.statusMessage }}
		</div>
		<div
			v-else-if="model.status === 'error'"
			class="canonical-sender-insights__state"
			role="alert"
		>
			<span>{{ model.statusMessage }}</span>
			<button type="button" @click="emit('retry')">Retry</button>
		</div>
		<div v-else-if="model.items.length === 0" class="canonical-sender-insights__state">
			{{ model.statusMessage }}
		</div>
		<div v-else class="canonical-sender-insights__list" :aria-busy="model.busy">
			<article v-for="sender in model.items" :key="sender.key">
				<div>
					<strong>{{ sender.displayLabel }}</strong>
					<small>{{ sender.referenceLabel }}</small>
				</div>
				<dl>
					<div><dt>Messages</dt><dd>{{ sender.messageCountLabel }}</dd></div>
					<div><dt>Conversations</dt><dd>{{ sender.conversationCountLabel }}</dd></div>
				</dl>
				<small>{{ sender.observedRangeLabel }}</small>
			</article>
		</div>
		<button
			v-if="model.hasMore"
			type="button"
			class="canonical-sender-insights__more"
			:disabled="model.busy"
			@click="emit('loadMore')"
		>Load more senders</button>
	</section>
</template>
