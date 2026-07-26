<script setup lang="ts">
import type { CanonicalCommunicationsPageModel } from './canonicalCommunicationsPageModel'
import './canonicalCommunicationsPage.css'

defineProps<{ model: CanonicalCommunicationsPageModel }>()

const emit = defineEmits<{
	retry: []
	search: []
	selectAccount: [accountKey: string]
	selectConversation: [conversationKey: string]
	updateSearchText: [value: string]
}>()
</script>

<template>
	<section class="canonical-communications-page">
		<header class="canonical-communications-page__header">
			<div>
				<span class="canonical-communications-page__eyebrow">Canonical evidence</span>
				<h1>Communications</h1>
				<p>Provider-neutral observations, conversations and message evidence.</p>
			</div>
			<form class="canonical-communications-search" role="search" @submit.prevent="emit('search')">
				<label for="canonical-communications-search">Search evidence</label>
				<div>
					<input
						id="canonical-communications-search"
						type="search"
						autocomplete="off"
						placeholder="Exact tokens"
						:value="model.searchText"
						@input="emit('updateSearchText', ($event.target as HTMLInputElement).value)"
					>
					<button type="submit" :disabled="model.searchStatus === 'loading'">
						{{ model.searchStatus === 'loading' ? 'Searching…' : 'Search' }}
					</button>
				</div>
			</form>
		</header>

		<div v-if="model.status === 'error'" class="canonical-communications-state" role="alert">
			<strong>Communications unavailable</strong>
			<p>{{ model.statusMessage }}</p>
			<button type="button" @click="emit('retry')">Retry</button>
		</div>

		<div v-else class="canonical-communications-workbench" :aria-busy="model.status === 'loading'">
			<aside class="canonical-communications-pane canonical-communications-pane--accounts">
				<header><h2>Sources</h2><span>{{ model.accounts.length }}</span></header>
				<button
					v-for="account in model.accounts"
					:key="account.key"
					type="button"
					class="canonical-communications-row"
					:class="{ selected: account.selected }"
					:aria-pressed="account.selected"
					@click="emit('selectAccount', account.key)"
				>
					<strong>{{ account.sourceLabel }}</strong>
					<span>{{ account.identityLabel }}</span>
					<small>{{ account.observedRangeLabel }}</small>
				</button>
				<p v-if="model.accounts.length === 0" class="canonical-communications-empty">
					{{ model.statusMessage }}
				</p>
			</aside>

			<section class="canonical-communications-pane canonical-communications-pane--conversations">
				<header><h2>Conversations</h2><span>{{ model.conversations.length }}</span></header>
				<button
					v-for="conversation in model.conversations"
					:key="conversation.key"
					type="button"
					class="canonical-communications-row"
					:class="{ selected: conversation.selected }"
					:aria-pressed="conversation.selected"
					@click="emit('selectConversation', conversation.key)"
				>
					<strong>{{ conversation.identityLabel }}</strong>
					<span>{{ conversation.sourceLabel }}</span>
					<small>{{ conversation.observedRangeLabel }}</small>
				</button>
				<p v-if="model.conversations.length === 0" class="canonical-communications-empty">
					{{ model.statusMessage }}
				</p>
			</section>

			<main class="canonical-communications-pane canonical-communications-pane--messages">
				<header><h2>Message evidence</h2><span>{{ model.messages.length }}</span></header>
				<article
					v-for="message in model.messages"
					:key="message.key"
					class="canonical-communications-message"
				>
					<div><strong>{{ message.identityLabel }}</strong><span>{{ message.directionLabel }}</span></div>
					<p>{{ message.stateLabel }}</p>
					<small>{{ message.observedRangeLabel }}</small>
				</article>
				<p v-if="model.messages.length === 0" class="canonical-communications-empty">
					{{ model.statusMessage || 'Select a canonical conversation.' }}
				</p>
			</main>
		</div>

		<section class="canonical-communications-results" aria-labelledby="canonical-search-results-title">
			<header>
				<div>
					<h2 id="canonical-search-results-title">Search results</h2>
					<p>Metadata-only matches from the rebuildable owner index.</p>
				</div>
				<span>{{ model.searchResults.length }}</span>
			</header>
			<p v-if="model.searchMessage" class="canonical-communications-empty">{{ model.searchMessage }}</p>
			<div v-else class="canonical-communications-results__grid">
				<article v-for="result in model.searchResults" :key="result.key">
					<strong>{{ result.evidenceLabel }}</strong>
					<span>{{ result.messageLabel }}</span>
					<span>{{ result.conversationLabel }}</span>
					<footer><small>{{ result.observedAtLabel }}</small><em>{{ result.matchLabel }}</em></footer>
				</article>
			</div>
		</section>
	</section>
</template>
