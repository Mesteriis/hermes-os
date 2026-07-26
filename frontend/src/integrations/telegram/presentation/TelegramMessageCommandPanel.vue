<script setup lang="ts">
import type { TelegramMessageCommandModel } from '../queries/useTelegramMessageCommands'

defineProps<{ model: TelegramMessageCommandModel }>()

const emit = defineEmits<{
	delete: []
	edit: []
	forward: []
	pin: [active: boolean]
	react: [active: boolean]
	reply: []
	restore: []
	updateEmoji: [value: string]
	updateRestoreReason: [value: string]
	updateTargetChatId: [value: string]
	updateText: [value: string]
}>()
</script>

<template>
	<section class="telegram-command-panel">
		<header>
			<h3>Selected message</h3>
			<small>{{ model.selectedMessageId || 'Select a message above' }}</small>
		</header>
		<label for="telegram-command-text">Reply or edit text</label>
		<textarea
			id="telegram-command-text"
			rows="2"
			:value="model.text"
			@input="emit('updateText', ($event.target as HTMLTextAreaElement).value)"
		/>
		<div class="telegram-command-panel__actions">
			<button type="button" :disabled="!model.selectedMessageId || !model.text.trim() || !model.canCommand || model.pending" @click="emit('reply')">Reply</button>
			<button type="button" :disabled="!model.selectedMessageId || !model.text.trim() || !model.canCommand || model.pending" @click="emit('edit')">Edit</button>
			<button class="danger" type="button" :disabled="!model.selectedMessageId || !model.canCommand || model.pending" @click="emit('delete')">Delete</button>
		</div>
		<label for="telegram-forward-chat-id">Forward to chat ID</label>
		<div>
			<input
				id="telegram-forward-chat-id"
				:value="model.targetChatId"
				@input="emit('updateTargetChatId', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.selectedMessageId || !model.targetChatId.trim() || !model.canCommand || model.pending" @click="emit('forward')">Forward</button>
		</div>
		<label for="telegram-reaction-emoji">Reaction</label>
		<div>
			<input
				id="telegram-reaction-emoji"
				:value="model.emoji"
				@input="emit('updateEmoji', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.selectedMessageId || !model.emoji.trim() || !model.canCommand || model.pending" @click="emit('react', true)">Add</button>
			<button type="button" :disabled="!model.selectedMessageId || !model.emoji.trim() || !model.canCommand || model.pending" @click="emit('react', false)">Remove</button>
		</div>
		<label for="telegram-restore-reason">Restore reason</label>
		<div>
			<input
				id="telegram-restore-reason"
				:value="model.restoreReason"
				@input="emit('updateRestoreReason', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.selectedMessageId || !model.restoreReason.trim() || !model.canCommand || model.pending" @click="emit('restore')">Restore</button>
		</div>
		<div class="telegram-command-panel__actions">
			<button type="button" :disabled="!model.selectedMessageId || !model.canCommand || model.pending" @click="emit('pin', true)">Pin</button>
			<button type="button" :disabled="!model.selectedMessageId || !model.canCommand || model.pending" @click="emit('pin', false)">Unpin</button>
		</div>
		<small role="status">{{ model.statusMessage }}</small>
	</section>
</template>
