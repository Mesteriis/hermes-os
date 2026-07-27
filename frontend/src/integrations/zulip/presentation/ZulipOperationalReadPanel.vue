<script setup lang="ts">
import ZulipMessageRow from './ZulipMessageRow.vue'
import type { ZulipOperationalReadModel } from './zulipOperationalReadModel'
import './zulipOperationalReadPanel.css'

defineProps<{ model: ZulipOperationalReadModel }>()

const emit = defineEmits<{
	loadMoreConversations: []
	loadMoreEvents: []
	loadMoreMessages: []
	loadMoreSearchResults: []
	refresh: []
	search: []
	selectAccount: [accountId: string]
	selectConversation: [providerConversationId: string]
	updateSearchQuery: [value: string]
}>()
</script>

<template>
	<section class="zulip-read-panel" :aria-busy="model.state === 'loading'">
		<header class="zulip-read-panel__header">
			<div>
				<span>Operational projection</span>
				<h2>Zulip account</h2>
				<p>Provider-owned history, stream topics, direct conversations and event state.</p>
			</div>
			<form @submit.prevent="emit('refresh')">
				<label for="zulip-operational-account">Admitted account</label>
				<div>
					<select
						id="zulip-operational-account"
						:value="model.selectedAccountId"
						:disabled="!model.canQuery || model.accounts.length === 0"
						@change="emit('selectAccount', ($event.target as HTMLSelectElement).value)"
					>
						<option v-if="model.accounts.length === 0" value="">No account</option>
						<option v-for="account in model.accounts" :key="account.id" :value="account.id">
							{{ account.label }}
						</option>
					</select>
					<button
						type="submit"
						:disabled="!model.canQuery || !model.selectedAccountId || model.state === 'loading'"
					>
						{{ model.state === 'loading' ? 'Loading…' : 'Refresh' }}
					</button>
				</div>
			</form>
		</header>

		<p
			v-if="model.statusMessage"
			class="zulip-read-panel__status"
			:role="model.state === 'error' ? 'alert' : 'status'"
		>
			{{ model.statusMessage }}
		</p>

		<dl v-if="model.accountStatus" class="zulip-account-summary">
			<div><dt>Projection</dt><dd>{{ model.accountStatus.projectionState }}</dd></div>
			<div><dt>History</dt><dd>{{ model.accountStatus.historyState }}</dd></div>
			<div><dt>Credential</dt><dd>{{ model.accountStatus.credentialState }}</dd></div>
			<div><dt>Latest sequence</dt><dd>{{ model.accountStatus.latestSequence }}</dd></div>
			<div><dt>Binding revision</dt><dd>{{ model.accountStatus.bindingRevision }}</dd></div>
			<div><dt>Runtime generation</dt><dd>{{ model.accountStatus.runtimeGeneration }}</dd></div>
		</dl>

		<form class="zulip-operational-search" @submit.prevent="emit('search')">
			<label for="zulip-message-search">Search provider history</label>
			<div>
				<input
					id="zulip-message-search"
					type="search"
					autocomplete="off"
					placeholder="Search raw Markdown in the selected conversation"
					:value="model.searchQuery"
					:disabled="!model.canQuery"
					@input="emit('updateSearchQuery', ($event.target as HTMLInputElement).value)"
				>
				<button
					type="submit"
					:disabled="!model.canQuery || !model.searchQuery.trim() || model.state === 'loading'"
				>
					Search
				</button>
			</div>
		</form>

		<section v-if="model.searchResults.length > 0" class="zulip-read-section">
			<header><h3>Search results</h3><span>{{ model.searchResults.length }}</span></header>
			<ZulipMessageRow v-for="message in model.searchResults" :key="message.id" :message="message" />
			<button v-if="model.hasMoreSearchResults" type="button" class="zulip-read-more" @click="emit('loadMoreSearchResults')">
				Load more search results
			</button>
		</section>

		<div class="zulip-read-workbench">
			<aside class="zulip-read-section">
				<header><h3>Conversations</h3><span>{{ model.conversations.length }}</span></header>
				<button
					v-for="conversation in model.conversations"
					:key="conversation.id"
					type="button"
					class="zulip-conversation-row"
					:class="{ selected: conversation.selected }"
					:aria-pressed="conversation.selected"
					@click="emit('selectConversation', conversation.id)"
				>
					<strong>{{ conversation.title }}</strong>
					<small>{{ conversation.kind }}</small>
					<small>{{ conversation.meta }}</small>
				</button>
				<button v-if="model.hasMoreConversations" type="button" class="zulip-read-more" @click="emit('loadMoreConversations')">
					Load more conversations
				</button>
			</aside>

			<section class="zulip-read-section">
				<header><h3>Messages</h3><span>{{ model.messages.length }}</span></header>
				<ZulipMessageRow v-for="message in model.messages" :key="message.id" :message="message" />
				<button v-if="model.hasMoreMessages" type="button" class="zulip-read-more" @click="emit('loadMoreMessages')">
					Load more messages
				</button>
			</section>
		</div>

		<section class="zulip-read-section zulip-event-journal">
			<header><h3>Provider event journal</h3><span>{{ model.events.length }}</span></header>
			<article v-for="event in model.events" :key="event.id">
				<header><strong>{{ event.kind }}</strong><small>{{ event.meta }}</small></header>
				<p>{{ event.summary }}</p>
			</article>
			<button v-if="model.hasMoreEvents" type="button" class="zulip-read-more" @click="emit('loadMoreEvents')">
				Load more events
			</button>
		</section>
	</section>
</template>
