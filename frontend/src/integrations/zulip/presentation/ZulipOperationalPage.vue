<script setup lang="ts">
import ZulipOperationalReadPanel from './ZulipOperationalReadPanel.vue'
import type { ZulipOperationalReadModel } from './zulipOperationalReadModel'
import ZulipOperationalReplayPanel from './ZulipOperationalReplayPanel.vue'
import type { ZulipOperationalReplayModel } from './zulipOperationalReplayModel'
import type {
	ZulipDestination,
	ZulipOperationalPageModel,
} from './zulipOperationalPageModel'
import './zulipOperationalPage.css'

defineProps<{
	model: ZulipOperationalPageModel
	readModel: ZulipOperationalReadModel
	replayModel: ZulipOperationalReplayModel
}>()

const emit = defineEmits<{
	loadMoreConversations: []
	loadMoreEvents: []
	loadMoreMessages: []
	loadMoreReplay: []
	loadMoreSearchResults: []
	readRefresh: []
	refreshStatus: []
	replayRefresh: []
	search: []
	selectConversation: [providerConversationId: string]
	selectReadAccount: [accountId: string]
	selectReplayAccount: [accountId: string]
	send: []
	updateDestination: [value: ZulipDestination]
	updateAccountId: [value: string]
	updateStream: [value: string]
	updateTopic: [value: string]
	updateRecipients: [value: string]
	updateContent: [value: string]
	updateOperationId: [value: string]
	updateSearchQuery: [value: string]
}>()
</script>

<template>
	<section class="zulip-operational-page">
		<header>
			<span>Provider operations</span>
			<h1>Zulip</h1>
			<p>
				Stream and direct delivery stay in this integration. Canonical Communications reads
				only durable evidence emitted after provider execution.
			</p>
		</header>

		<ZulipOperationalReadPanel
			:model="readModel"
			@load-more-conversations="emit('loadMoreConversations')"
			@load-more-events="emit('loadMoreEvents')"
			@load-more-messages="emit('loadMoreMessages')"
			@load-more-search-results="emit('loadMoreSearchResults')"
			@refresh="emit('readRefresh')"
			@search="emit('search')"
			@select-account="emit('selectReadAccount', $event)"
			@select-conversation="emit('selectConversation', $event)"
			@update-search-query="emit('updateSearchQuery', $event)"
		/>

		<ZulipOperationalReplayPanel
			:model="replayModel"
			@load-more="emit('loadMoreReplay')"
			@refresh="emit('replayRefresh')"
			@select-account="emit('selectReplayAccount', $event)"
		/>

		<div class="zulip-operational-grid">
			<form class="zulip-operational-card" @submit.prevent="emit('send')">
				<div>
					<span>Command</span>
					<h2>Send message</h2>
					<p>Dispatch an exact provider command. Acceptance does not mean completion.</p>
				</div>
				<div class="zulip-destination-selector" role="group" aria-label="Destination">
					<button
						type="button"
						:aria-pressed="model.destination === 'stream'"
						@click="emit('updateDestination', 'stream')"
					>
						Stream
					</button>
					<button
						type="button"
						:aria-pressed="model.destination === 'direct'"
						@click="emit('updateDestination', 'direct')"
					>
						Direct
					</button>
				</div>
				<label>
					Account ID
					<input
						autocomplete="off"
						placeholder="zulip-account-id"
						:value="model.accountId"
						@input="emit('updateAccountId', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<template v-if="model.destination === 'stream'">
					<label>
						Stream
						<input
							autocomplete="off"
							placeholder="engineering"
							:value="model.stream"
							@input="emit('updateStream', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Topic
						<input
							autocomplete="off"
							placeholder="clean-room"
							:value="model.topic"
							@input="emit('updateTopic', ($event.target as HTMLInputElement).value)"
						>
					</label>
				</template>
				<label v-else>
					Recipients
					<input
						autocomplete="off"
						placeholder="owner@example.com, team@example.com"
						:value="model.recipients"
						@input="emit('updateRecipients', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					Message
					<textarea
						rows="5"
						placeholder="Message content"
						:value="model.content"
						:disabled="!model.canCommand || model.busy"
						@input="emit('updateContent', ($event.target as HTMLTextAreaElement).value)"
					/>
				</label>
				<button
					type="submit"
					:disabled="
						!model.canCommand
							|| !model.accountId.trim()
							|| !model.content.trim()
							|| (model.destination === 'stream' && (!model.stream.trim() || !model.topic.trim()))
							|| (model.destination === 'direct' && !model.recipients.trim())
							|| model.busy
					"
				>
					{{ model.busy ? 'Working…' : 'Send command' }}
				</button>
				<small v-if="!model.canCommand">Zulip command capability is not admitted.</small>
			</form>

			<section class="zulip-operational-card">
				<div>
					<span>Terminal result</span>
					<h2>Operation status</h2>
					<p>Query the receipt returned by the provider command contract.</p>
				</div>
				<form class="zulip-status-loader" @submit.prevent="emit('refreshStatus')">
					<label for="zulip-operation-id">Operation ID</label>
					<div>
						<input
							id="zulip-operation-id"
							autocomplete="off"
							placeholder="operation-id"
							:value="model.operationId"
							@input="emit('updateOperationId', ($event.target as HTMLInputElement).value)"
						>
						<button type="submit" :disabled="!model.operationId.trim() || model.busy">Refresh</button>
					</div>
				</form>
				<dl v-if="model.status" class="zulip-operation-status">
					<div><dt>Outcome</dt><dd>{{ model.status.outcome }}</dd></div>
					<div><dt>Operation</dt><dd>{{ model.status.operationId }}</dd></div>
					<div><dt>Account</dt><dd>{{ model.status.accountId }}</dd></div>
					<div><dt>Provider message</dt><dd>{{ model.status.providerMessageId }}</dd></div>
					<div><dt>Requested</dt><dd>{{ model.status.requestedAt }}</dd></div>
					<div><dt>Completed</dt><dd>{{ model.status.completedAt }}</dd></div>
				</dl>
				<p v-else class="zulip-operational-empty">No operation selected.</p>
			</section>
		</div>

		<p v-if="model.notice" class="zulip-operational-notice" role="status">{{ model.notice }}</p>
	</section>
</template>
