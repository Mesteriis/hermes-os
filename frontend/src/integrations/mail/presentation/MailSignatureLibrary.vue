<script setup lang="ts">
import type {
	MailCompositionModel,
	MailSignatureEditorPatch,
} from './mailCompositionModel'

defineProps<{ model: MailCompositionModel }>()
const emit = defineEmits<{
	newSignature: []
	removeSignature: []
	saveSignature: []
	selectSignature: [signatureId: string]
	updateSignature: [patch: MailSignatureEditorPatch]
	useSignature: [signatureId: string]
}>()
</script>

<template>
	<section class="mail-operational-card">
		<div class="mail-card-heading">
			<div>
				<span>Sender identity</span>
				<h2>Signatures</h2>
				<p>The selected signature is composed into delivery without changing the saved body.</p>
			</div>
			<button type="button" class="mail-button-secondary" @click="emit('newSignature')">New</button>
		</div>
		<label>
			Saved signatures
			<select
				:value="model.signature.signatureId"
				@change="emit('selectSignature', ($event.target as HTMLSelectElement).value)"
			>
				<option value="">Unsaved signature</option>
				<option v-for="signature in model.signatures" :key="signature.id" :value="signature.id">
					{{ signature.label }} — {{ signature.detail }}
				</option>
			</select>
		</label>
		<label>
			Name
			<input
				:value="model.signature.name"
				@input="emit('updateSignature', {
					name: ($event.target as HTMLInputElement).value,
				})"
			>
		</label>
		<label>
			Signature body
			<textarea
				rows="5"
				:value="model.signature.textBody"
				@input="emit('updateSignature', {
					textBody: ($event.target as HTMLTextAreaElement).value,
				})"
			/>
		</label>
		<label class="mail-checkbox-field">
			<input
				type="checkbox"
				:checked="model.signature.isDefault"
				@change="emit('updateSignature', {
					isDefault: ($event.target as HTMLInputElement).checked,
				})"
			>
			Default for this Mail connection
		</label>
		<div class="mail-composition-actions">
			<button
				type="button"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('saveSignature')"
			>
				Save
			</button>
			<button
				type="button"
				:disabled="!model.signature.signatureId"
				@click="emit('useSignature', model.signature.signatureId)"
			>
				Use in draft
			</button>
			<button
				v-if="model.signature.signatureId"
				type="button"
				class="mail-button-danger"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('removeSignature')"
			>
				Delete
			</button>
		</div>
	</section>
</template>
