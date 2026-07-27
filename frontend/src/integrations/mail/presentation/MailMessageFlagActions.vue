<script setup lang="ts">
import type { MailMessageFlagModel } from './mailMessageFlagModel'

defineProps<{ model: MailMessageFlagModel }>()

const emit = defineEmits<{
	refreshStatus: []
	setRead: [targetValue: boolean]
	setStarred: [targetValue: boolean]
}>()
</script>

<template>
	<section class="mail-message-flag-actions" aria-label="Provider message flags">
		<div>
			<span>Provider flags</span>
			<strong>Read and starred state</strong>
		</div>
		<div class="mail-message-flag-actions__controls">
			<button
				type="button"
				:disabled="!model.canMutate || !model.canQueryStatus || !model.hasSelection || model.busy"
				@click="emit('setRead', !model.isRead)"
			>
				{{ model.isRead ? 'Mark unread' : 'Mark read' }}
			</button>
			<button
				type="button"
				:disabled="!model.canMutate || !model.canQueryStatus || !model.hasSelection || model.busy"
				@click="emit('setStarred', !model.isStarred)"
			>
				{{ model.isStarred ? 'Remove star' : 'Add star' }}
			</button>
			<button
				v-if="model.operationId"
				type="button"
				:disabled="!model.canQueryStatus || model.busy"
				@click="emit('refreshStatus')"
			>
				Refresh status
			</button>
		</div>
		<p v-if="model.statusMessage" :role="model.status === 'error' ? 'alert' : 'status'">
			{{ model.statusMessage }}
		</p>
	</section>
</template>
