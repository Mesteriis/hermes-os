<script setup lang="ts">
import type { MailMessageLocationModel } from './mailMessageLocationModel'

defineProps<{ model: MailMessageLocationModel }>()

const emit = defineEmits<{
	archive: []
	move: []
	refreshStatus: []
	restore: []
	selectTargetFolder: [folderId: string]
	trash: []
}>()
</script>

<template>
	<section class="mail-message-flag-actions" aria-label="Provider message location">
		<div>
			<span>Provider location</span>
			<strong>Archive, trash, restore or move</strong>
		</div>
		<div class="mail-message-flag-actions__controls">
			<button
				type="button"
				:disabled="!model.canMutate || !model.canQueryStatus || !model.hasSelection || model.busy"
				@click="emit('archive')"
			>
				Archive
			</button>
			<button
				v-if="!model.isTrashed"
				type="button"
				:disabled="!model.canMutate || !model.canQueryStatus || !model.hasSelection || model.busy"
				@click="emit('trash')"
			>
				Move to trash
			</button>
			<button
				v-else
				type="button"
				:disabled="!model.canMutate || !model.canQueryStatus || !model.hasSelection || model.busy"
				@click="emit('restore')"
			>
				Restore
			</button>
			<select
				:value="model.targetFolderId"
				:disabled="!model.hasSelection || model.busy || model.targetFolders.length === 0"
				aria-label="Move message to folder"
				@change="emit('selectTargetFolder', ($event.target as HTMLSelectElement).value)"
			>
				<option value="">Select folder</option>
				<option v-for="folder in model.targetFolders" :key="folder.id" :value="folder.id">
					{{ folder.label }}
				</option>
			</select>
			<button
				type="button"
				:disabled="!model.canMutate || !model.canQueryStatus || !model.hasSelection || !model.targetFolderId || model.busy"
				@click="emit('move')"
			>
				Move
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
