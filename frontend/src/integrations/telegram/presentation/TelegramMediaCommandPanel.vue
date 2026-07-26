<script setup lang="ts">
import type { TelegramMediaCommandModel } from '../queries/useTelegramMediaCommands'

defineProps<{ model: TelegramMediaCommandModel }>()

const emit = defineEmits<{
	downloadFile: []
	sendMedia: []
	updateBackupClass: [value: string]
	updateBlobRef: [value: string]
	updateCaption: [value: string]
	updateDeclaredSize: [value: string]
	updateFilename: [value: string]
	updateMediaKind: [value: string]
	updateProviderFileId: [value: string]
	updateReferenceIdHex: [value: string]
}>()
</script>

<template>
	<section class="telegram-command-panel">
		<header><h3>Media & files</h3></header>
		<label for="telegram-media-kind">Media kind</label>
		<input id="telegram-media-kind" :value="model.mediaKind" @input="emit('updateMediaKind', ($event.target as HTMLInputElement).value)">
		<label for="telegram-media-blob-ref">Admitted Blob reference</label>
		<input id="telegram-media-blob-ref" :value="model.blobRef" @input="emit('updateBlobRef', ($event.target as HTMLInputElement).value)">
		<label for="telegram-media-reference-id">Reference ID (hex)</label>
		<input id="telegram-media-reference-id" :value="model.referenceIdHex" @input="emit('updateReferenceIdHex', ($event.target as HTMLInputElement).value)">
		<label for="telegram-media-size">Declared size</label>
		<input id="telegram-media-size" inputmode="numeric" :value="model.declaredSize" @input="emit('updateDeclaredSize', ($event.target as HTMLInputElement).value)">
		<label for="telegram-media-backup-class">Backup class from Blob receipt</label>
		<input id="telegram-media-backup-class" inputmode="numeric" :value="model.backupClass" @input="emit('updateBackupClass', ($event.target as HTMLInputElement).value)">
		<label for="telegram-media-caption">Caption</label>
		<input id="telegram-media-caption" :value="model.caption" @input="emit('updateCaption', ($event.target as HTMLInputElement).value)">
		<label for="telegram-media-filename">Filename</label>
		<input id="telegram-media-filename" :value="model.filename" @input="emit('updateFilename', ($event.target as HTMLInputElement).value)">
		<button
			type="button"
			:disabled="!model.hasChat || !model.blobRef.trim() || !model.referenceIdHex.trim() || !model.declaredSize.trim() || !model.backupClass.trim() || !model.canCommand || model.pending"
			@click="emit('sendMedia')"
		>
			Send media
		</button>
		<label for="telegram-provider-file-id">Provider file ID</label>
		<div>
			<input id="telegram-provider-file-id" :value="model.providerFileId" @input="emit('updateProviderFileId', ($event.target as HTMLInputElement).value)">
			<button type="button" :disabled="!model.providerFileId.trim() || !model.canCommand || model.pending" @click="emit('downloadFile')">Download</button>
		</div>
		<small role="status">{{ model.statusMessage }}</small>
	</section>
</template>
