<script setup lang="ts">
import type { TelegramChatCommandModel } from '../queries/useTelegramChatCommands'

defineProps<{ model: TelegramChatCommandModel }>()

const emit = defineEmits<{
	addToFolder: []
	archive: [active: boolean]
	join: []
	leave: []
	markUnread: [active: boolean]
	mute: [active: boolean]
	removeFromFolder: []
	updateFolderId: [value: string]
}>()
</script>

<template>
	<section class="telegram-command-panel">
		<header><h3>Selected chat</h3></header>
		<div class="telegram-command-panel__actions">
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('markUnread', true)">Mark unread</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('markUnread', false)">Mark read</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('archive', true)">Archive</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('archive', false)">Unarchive</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('mute', true)">Mute</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('mute', false)">Unmute</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('join')">Join</button>
			<button class="danger" type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('leave')">Leave</button>
		</div>
		<label for="telegram-folder-id">Folder ID</label>
		<div>
			<input
				id="telegram-folder-id"
				inputmode="numeric"
				:value="model.folderId"
				@input="emit('updateFolderId', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.hasChat || !model.folderId.trim() || !model.canCommand || model.pending" @click="emit('addToFolder')">Add</button>
			<button type="button" :disabled="!model.hasChat || !model.folderId.trim() || !model.canCommand || model.pending" @click="emit('removeFromFolder')">Remove</button>
		</div>
		<small role="status">{{ model.statusMessage }}</small>
	</section>
</template>
