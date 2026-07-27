<script setup lang="ts">
import type { MailDeliveryModel } from './mailDeliveryModel'

defineProps<{ model: MailDeliveryModel }>()
const emit = defineEmits<{
	refreshStatus: []
	updateOperationId: [value: string]
}>()
</script>

<template>
	<section class="mail-operational-card">
		<div>
			<span>Terminal result</span>
			<h2>Delivery status</h2>
			<p>Accepted is asynchronous. Query the Mail-owned receipt for completion.</p>
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
				<button type="submit" :disabled="!model.operationId.trim() || model.busy">Refresh</button>
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
		<p v-if="model.notice" class="mail-inline-notice" role="status">{{ model.notice }}</p>
	</section>
</template>
