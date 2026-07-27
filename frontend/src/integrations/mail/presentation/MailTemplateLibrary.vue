<script setup lang="ts">
import type {
	MailCompositionModel,
	MailTemplateEditorPatch,
} from './mailCompositionModel'

defineProps<{ model: MailCompositionModel }>()
const emit = defineEmits<{
	applyTemplate: []
	newTemplate: []
	removeTemplate: []
	saveTemplate: []
	selectTemplate: [templateId: string]
	updateTemplate: [patch: MailTemplateEditorPatch]
}>()
</script>

<template>
	<section class="mail-operational-card">
		<div class="mail-card-heading">
			<div>
				<span>Reusable content</span>
				<h2>Templates</h2>
				<p>Typed <code v-pre>{{variable}}</code> substitution is previewed before use.</p>
			</div>
			<button type="button" class="mail-button-secondary" @click="emit('newTemplate')">New</button>
		</div>
		<label>
			Saved templates
			<select
				:value="model.template.templateId"
				@change="emit('selectTemplate', ($event.target as HTMLSelectElement).value)"
			>
				<option value="">Unsaved template</option>
				<option v-for="template in model.templates" :key="template.id" :value="template.id">
					{{ template.label }} — {{ template.detail }}
				</option>
			</select>
		</label>
		<label>
			Name
			<input
				:value="model.template.name"
				@input="emit('updateTemplate', {
					name: ($event.target as HTMLInputElement).value,
				})"
			>
		</label>
		<label>
			Subject template
			<input
				:value="model.template.subjectTemplate"
				@input="emit('updateTemplate', {
					subjectTemplate: ($event.target as HTMLInputElement).value,
				})"
			>
		</label>
		<label>
			Body template
			<textarea
				rows="5"
				:value="model.template.textBodyTemplate"
				@input="emit('updateTemplate', {
					textBodyTemplate: ($event.target as HTMLTextAreaElement).value,
				})"
			/>
		</label>
		<div class="mail-composition-inline-fields">
			<label>
				Variables <small>One per line</small>
				<textarea
					rows="3"
					:value="model.template.variables"
					@input="emit('updateTemplate', {
						variables: ($event.target as HTMLTextAreaElement).value,
					})"
				/>
			</label>
			<label>
				Locale <small>Optional</small>
				<input
					:value="model.template.locale"
					@input="emit('updateTemplate', {
						locale: ($event.target as HTMLInputElement).value,
					})"
				>
			</label>
		</div>
		<label>
			Preview values <small><code>name=value</code>, one per line</small>
			<textarea
				rows="3"
				:value="model.template.previewValues"
				@input="emit('updateTemplate', {
					previewValues: ($event.target as HTMLTextAreaElement).value,
				})"
			/>
		</label>
		<p v-if="model.template.previewSummary" class="mail-inline-notice" role="status">
			{{ model.template.previewSummary }}
		</p>
		<div class="mail-composition-actions">
			<button
				type="button"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('saveTemplate')"
			>
				Save
			</button>
			<button
				type="button"
				:disabled="!model.template.templateId || model.busyAction !== null"
				@click="emit('applyTemplate')"
			>
				Preview &amp; apply
			</button>
			<button
				v-if="model.template.templateId"
				type="button"
				class="mail-button-danger"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('removeTemplate')"
			>
				Delete
			</button>
		</div>
	</section>
</template>
