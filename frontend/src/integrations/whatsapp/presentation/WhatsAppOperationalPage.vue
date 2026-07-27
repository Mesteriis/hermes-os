<script setup lang="ts">
import WhatsAppOperationalReadPanel from './WhatsAppOperationalReadPanel.vue'
import type { WhatsAppOperationalReadModel } from './whatsAppOperationalReadModel'
import WhatsAppOperationalReplayPanel from './WhatsAppOperationalReplayPanel.vue'
import type { WhatsAppOperationalReplayModel } from './whatsAppOperationalReplayModel'
import type { WhatsAppOperationalPageModel } from './whatsAppOperationalPageModel'
import './whatsAppOperationalPage.css'

defineProps<{
	model: WhatsAppOperationalPageModel
	readModel: WhatsAppOperationalReadModel
	replayModel: WhatsAppOperationalReplayModel
}>()

const emit = defineEmits<{
	loadMoreDialogs: []
	loadMoreEvents: []
	loadMoreMessages: []
	loadMoreParticipants: []
	loadMoreReplay: []
	loadMoreSearchResults: []
	readRefresh: []
	refreshStatus: []
	replayRefresh: []
	search: []
	selectReadAccount: [accountId: string]
	selectReplayAccount: [accountId: string]
	selectDialog: [providerChatId: string]
	send: []
	updateAccountId: [value: string]
	updateProviderChatId: [value: string]
	updateDraft: [value: string]
	updateOperationId: [value: string]
	updateSearchQuery: [value: string]
}>()
</script>

<template>
	<section class="whatsapp-operational-page">
		<header>
			<span>Provider operations</span>
			<h1>WhatsApp</h1>
			<p>
				Commands and terminal receipts stay in this integration. Provider browser execution
				remains isolated in the first-party host WebView.
			</p>
		</header>

		<WhatsAppOperationalReadPanel
			:model="readModel"
			@load-more-dialogs="emit('loadMoreDialogs')"
			@load-more-events="emit('loadMoreEvents')"
			@load-more-messages="emit('loadMoreMessages')"
			@load-more-participants="emit('loadMoreParticipants')"
			@load-more-search-results="emit('loadMoreSearchResults')"
			@refresh="emit('readRefresh')"
			@search="emit('search')"
			@select-account="emit('selectReadAccount', $event)"
			@select-dialog="emit('selectDialog', $event)"
			@update-search-query="emit('updateSearchQuery', $event)"
		/>

		<WhatsAppOperationalReplayPanel
			:model="replayModel"
			@load-more="emit('loadMoreReplay')"
			@refresh="emit('replayRefresh')"
			@select-account="emit('selectReplayAccount', $event)"
		/>

		<div class="whatsapp-operational-grid">
			<form class="whatsapp-operational-card" @submit.prevent="emit('send')">
				<div>
					<span>Command</span>
					<h2>Send text</h2>
					<p>Dispatch an exact provider command. Acceptance does not mean completion.</p>
				</div>
				<label>
					Account ID
					<input
						autocomplete="off"
						placeholder="whatsapp-account-id"
						:value="model.accountId"
						@input="emit('updateAccountId', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					Provider chat ID
					<input
						autocomplete="off"
						placeholder="provider-chat-id"
						:value="model.providerChatId"
						@input="emit('updateProviderChatId', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					Message
					<textarea
						rows="5"
						placeholder="Message text"
						:value="model.draft"
						:disabled="!model.canSend || model.busy"
						@input="emit('updateDraft', ($event.target as HTMLTextAreaElement).value)"
					/>
				</label>
				<button
					type="submit"
					:disabled="!model.canSend || !model.accountId.trim() || !model.providerChatId.trim() || !model.draft.trim() || model.busy"
				>
					{{ model.busy ? 'Working…' : 'Send command' }}
				</button>
				<small v-if="!model.canSend">WhatsApp command capability is not admitted.</small>
			</form>

			<section class="whatsapp-operational-card">
				<div>
					<span>Terminal result</span>
					<h2>Operation status</h2>
					<p>Query a receipt by the operation ID returned from the command contract.</p>
				</div>
				<form class="whatsapp-status-loader" @submit.prevent="emit('refreshStatus')">
					<label for="whatsapp-operation-id">Operation ID</label>
					<div>
						<input
							id="whatsapp-operation-id"
							autocomplete="off"
							placeholder="operation-id"
							:value="model.operationId"
							@input="emit('updateOperationId', ($event.target as HTMLInputElement).value)"
						>
						<button type="submit" :disabled="!model.operationId.trim() || model.busy">Refresh</button>
					</div>
				</form>
				<dl v-if="model.status" class="whatsapp-operation-status">
					<div><dt>State</dt><dd>{{ model.status.state }}</dd></div>
					<div><dt>Operation</dt><dd>{{ model.status.operationId }}</dd></div>
					<div><dt>Account</dt><dd>{{ model.status.accountId }}</dd></div>
					<div><dt>Requested</dt><dd>{{ model.status.requestedAt }}</dd></div>
					<div><dt>Completed</dt><dd>{{ model.status.completedAt }}</dd></div>
				</dl>
				<p v-else class="whatsapp-operational-empty">No operation selected.</p>
			</section>
		</div>

		<p v-if="model.notice" class="whatsapp-operational-notice" role="status">{{ model.notice }}</p>
	</section>
</template>
