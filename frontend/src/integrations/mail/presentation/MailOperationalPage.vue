<script setup lang="ts">
import type { MailOperationalPageModel } from './mailOperationalPageModel'
import './mailOperationalPage.css'

defineProps<{ model: MailOperationalPageModel }>()

const emit = defineEmits<{
	deliver: []
	refreshStatus: []
	sync: []
	updateOperationId: [value: string]
	updateProviderConversationId: [value: string]
	updateRecipients: [value: string]
	updateSubject: [value: string]
	updateTextBody: [value: string]
}>()
</script>

<template>
	<section class="mail-operational-page">
		<header class="mail-operational-page__header">
			<div>
				<span>Provider operations</span>
				<h1>Mail</h1>
				<p>Bounded inbox sync, delivery commands and terminal delivery receipts.</p>
			</div>
			<section class="mail-sync-card">
				<div><strong>Inbox sync</strong><small>{{ model.syncSummary || 'No sync run in this session.' }}</small></div>
				<button type="button" :disabled="!model.canSync || model.busyAction !== null" @click="emit('sync')">
					{{ model.busyAction === 'sync' ? 'Syncing…' : 'Sync now' }}
				</button>
			</section>
		</header>

		<div class="mail-operational-grid">
			<form class="mail-operational-card" @submit.prevent="emit('deliver')">
				<div>
					<span>Delivery command</span>
					<h2>Compose mail</h2>
					<p>The integration owns provider delivery. Canonical Communications receives evidence later.</p>
				</div>
				<label>
					Recipients
					<input
						autocomplete="off"
						placeholder="owner@example.com, team@example.com"
						:value="model.recipients"
						@input="emit('updateRecipients', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					Provider conversation ID <small>Optional reply context</small>
					<input
						autocomplete="off"
						placeholder="provider-conversation-id"
						:value="model.providerConversationId"
						@input="emit('updateProviderConversationId', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					Subject
					<input
						autocomplete="off"
						placeholder="Subject"
						:value="model.subject"
						@input="emit('updateSubject', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					Message
					<textarea
						rows="6"
						placeholder="Plain text body"
						:value="model.textBody"
						:disabled="!model.canDeliver"
						@input="emit('updateTextBody', ($event.target as HTMLTextAreaElement).value)"
					/>
				</label>
				<button
					type="submit"
					:disabled="!model.canDeliver || !model.recipients.trim() || !model.textBody.trim() || model.busyAction !== null"
				>
					{{ model.busyAction === 'delivery' ? 'Sending…' : 'Send mail' }}
				</button>
				<small v-if="!model.canDeliver">Mail delivery capability is not admitted.</small>
			</form>

			<section class="mail-operational-card">
				<div>
					<span>Terminal result</span>
					<h2>Delivery status</h2>
					<p>Accepted is asynchronous. Query the Mail-owned operation receipt for completion.</p>
				</div>
				<form class="mail-status-loader" @submit.prevent="emit('refreshStatus')">
					<label for="mail-operation-id">Operation ID</label>
					<div>
						<input
							id="mail-operation-id"
							autocomplete="off"
							placeholder="operation-id"
							:value="model.operationId"
							@input="emit('updateOperationId', ($event.target as HTMLInputElement).value)"
						>
						<button type="submit" :disabled="!model.operationId.trim() || model.busyAction !== null">Refresh</button>
					</div>
				</form>
				<dl v-if="model.status" class="mail-delivery-status">
					<div><dt>Outcome</dt><dd>{{ model.status.outcome }}</dd></div>
					<div><dt>Operation</dt><dd>{{ model.status.operationId }}</dd></div>
					<div><dt>Connection</dt><dd>{{ model.status.connectionId }}</dd></div>
					<div><dt>Requested</dt><dd>{{ model.status.requestedAt }}</dd></div>
					<div><dt>Completed</dt><dd>{{ model.status.completedAt }}</dd></div>
					<div><dt>Response</dt><dd>{{ model.status.responseCode }}</dd></div>
				</dl>
				<p v-else class="mail-operational-empty">No delivery selected.</p>
			</section>
		</div>

		<p v-if="model.notice" class="mail-operational-notice" role="status">{{ model.notice }}</p>
	</section>
</template>
