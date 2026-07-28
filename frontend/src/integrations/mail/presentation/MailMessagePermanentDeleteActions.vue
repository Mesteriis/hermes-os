<script setup lang="ts">
import type { MailMessagePermanentDeleteModel } from './mailMessagePermanentDeleteModel'

defineProps<{ model: MailMessagePermanentDeleteModel }>()

const emit = defineEmits<{
	delete: []
	refreshStatus: []
	setConfirmed: [confirmed: boolean]
}>()
</script>

<template>
	<section
		v-if="model.hasTrashSelection || model.operationId"
		class="mail-message-flag-actions mail-message-permanent-delete-actions"
		aria-label="Permanent provider deletion"
	>
		<div>
			<span>Destructive provider action</span>
			<strong>Delete permanently</strong>
			<small>Removes the provider message. Canonical Communications evidence remains.</small>
		</div>
		<label v-if="model.hasTrashSelection">
			<input
				type="checkbox"
				:checked="model.confirmed"
				:disabled="model.busy"
				@change="emit('setConfirmed', ($event.target as HTMLInputElement).checked)"
			>
			I understand this cannot be undone at the provider
		</label>
		<div class="mail-message-flag-actions__controls">
			<button
				v-if="model.hasTrashSelection"
				type="button"
				:disabled="!model.canDelete || !model.canQueryStatus || !model.confirmed || model.busy"
				@click="emit('delete')"
			>
				Delete permanently
			</button>
			<button
				v-if="model.operationId"
				type="button"
				:disabled="!model.canQueryStatus || model.busy"
				@click="emit('refreshStatus')"
			>
				Refresh delete status
			</button>
		</div>
		<p v-if="model.statusMessage" :role="model.status === 'error' ? 'alert' : 'status'">
			{{ model.statusMessage }}
		</p>
	</section>
</template>
