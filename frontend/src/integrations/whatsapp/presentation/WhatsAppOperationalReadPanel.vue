<script setup lang="ts">
import type { WhatsAppOperationalReadModel } from './whatsAppOperationalReadModel'
import './whatsAppOperationalReadPanel.css'

defineProps<{ model: WhatsAppOperationalReadModel }>()

const emit = defineEmits<{
	loadMoreDialogs: []
	loadMoreEvents: []
	loadMoreMessages: []
	loadMoreParticipants: []
	loadMoreSearchResults: []
	refresh: []
	search: []
	selectAccount: [accountId: string]
	selectDialog: [providerChatId: string]
	updateSearchQuery: [value: string]
}>()
</script>

<template>
	<section class="whatsapp-read-panel" :aria-busy="model.state === 'loading'">
		<header class="whatsapp-read-panel__header">
			<div>
				<span>Operational projection</span>
				<h2>WhatsApp account</h2>
				<p>Integration-owned dialogs, participants, messages and provider event history.</p>
			</div>
			<form @submit.prevent="emit('refresh')">
				<label for="whatsapp-operational-account">Admitted account</label>
				<div>
					<select
						id="whatsapp-operational-account"
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
			class="whatsapp-read-panel__status"
			:role="model.state === 'error' ? 'alert' : 'status'"
		>
			{{ model.statusMessage }}
		</p>

		<dl v-if="model.runtime" class="whatsapp-runtime-summary">
			<div><dt>Runtime</dt><dd>{{ model.runtime.state }}</dd></div>
			<div><dt>Projection</dt><dd>{{ model.runtime.projectionState }}</dd></div>
			<div><dt>Latest sequence</dt><dd>{{ model.runtime.latestSequence }}</dd></div>
		</dl>

		<form class="whatsapp-operational-search" @submit.prevent="emit('search')">
			<label for="whatsapp-message-search">Search provider messages</label>
			<div>
				<input
					id="whatsapp-message-search"
					type="search"
					autocomplete="off"
					placeholder="Search in selected dialog"
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

		<section v-if="model.searchResults.length > 0" class="whatsapp-read-section">
			<header><h3>Search results</h3><span>{{ model.searchResults.length }}</span></header>
			<article v-for="message in model.searchResults" :key="`${message.chatId}:${message.id}`" class="whatsapp-message-row">
				<header><strong>{{ message.sender }}</strong><small>{{ message.meta }}</small></header>
				<p>{{ message.text }}</p>
				<small>{{ message.chatId }} · {{ message.deliveryState }}</small>
			</article>
			<button v-if="model.hasMoreSearchResults" type="button" class="whatsapp-read-more" @click="emit('loadMoreSearchResults')">
				Load more search results
			</button>
		</section>

		<div class="whatsapp-read-workbench">
			<aside class="whatsapp-read-section">
				<header><h3>Dialogs</h3><span>{{ model.dialogs.length }}</span></header>
				<button
					v-for="dialog in model.dialogs"
					:key="dialog.id"
					type="button"
					class="whatsapp-dialog-row"
					:class="{ selected: dialog.selected }"
					:aria-pressed="dialog.selected"
					@click="emit('selectDialog', dialog.id)"
				>
					<strong>{{ dialog.title }}</strong>
					<small>{{ dialog.meta }}</small>
					<small>{{ dialog.flags }}</small>
				</button>
				<button v-if="model.hasMoreDialogs" type="button" class="whatsapp-read-more" @click="emit('loadMoreDialogs')">
					Load more dialogs
				</button>
			</aside>

			<section class="whatsapp-read-section">
				<header><h3>Messages</h3><span>{{ model.messages.length }}</span></header>
				<article v-for="message in model.messages" :key="`${message.chatId}:${message.id}`" class="whatsapp-message-row">
					<header><strong>{{ message.sender }}</strong><small>{{ message.meta }}</small></header>
					<p>{{ message.text }}</p>
					<small>{{ message.deliveryState }}</small>
				</article>
				<button v-if="model.hasMoreMessages" type="button" class="whatsapp-read-more" @click="emit('loadMoreMessages')">
					Load more messages
				</button>
			</section>

			<section class="whatsapp-read-section">
				<header><h3>Participants</h3><span>{{ model.participants.length }}</span></header>
				<article v-for="participant in model.participants" :key="participant.id" class="whatsapp-participant-row">
					<strong>{{ participant.displayName }}<small v-if="participant.isSelf">You</small></strong>
					<small>{{ participant.meta }}</small>
				</article>
				<button v-if="model.hasMoreParticipants" type="button" class="whatsapp-read-more" @click="emit('loadMoreParticipants')">
					Load more participants
				</button>
			</section>
		</div>

		<section class="whatsapp-read-section whatsapp-event-journal">
			<header><h3>Provider event journal</h3><span>{{ model.events.length }}</span></header>
			<article v-for="event in model.events" :key="event.id">
				<strong>{{ event.kind }}</strong>
				<p>{{ event.summary }}</p>
			</article>
			<button v-if="model.hasMoreEvents" type="button" class="whatsapp-read-more" @click="emit('loadMoreEvents')">
				Load more events
			</button>
		</section>
	</section>
</template>
