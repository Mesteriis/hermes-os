<script setup lang="ts">
import type { TelegramAutomationModel } from './telegramAutomationModel'
import './telegramAutomationPanel.css'

defineProps<{ model: TelegramAutomationModel }>()

const emit = defineEmits<{
	newPolicy: []
	newTemplate: []
	preview: []
	refresh: []
	savePolicy: []
	saveTemplate: []
	selectPolicy: [id: string]
	selectTemplate: [id: string]
	updatePolicyAccountId: [value: string]
	updatePolicyChatIds: [value: string]
	updatePolicyEnabled: [value: boolean]
	updatePolicyExpiresAt: [value: string]
	updatePolicyId: [value: string]
	updatePolicyName: [value: string]
	updatePolicyTemplateId: [value: string]
	updatePreviewAccountId: [value: string]
	updatePreviewChatId: [value: string]
	updatePreviewPolicyId: [value: string]
	updatePreviewVariables: [value: string]
	updateTemplateBody: [value: string]
	updateTemplateId: [value: string]
	updateTemplateName: [value: string]
	updateTemplateVariables: [value: string]
}>()
</script>

<template>
	<section class="telegram-automation">
		<header class="telegram-automation__header">
			<div>
				<h2>Automation policies</h2>
				<p>Telegram-owned templates, exact chat scopes and persisted dry-run previews.</p>
			</div>
			<button type="button" :disabled="!model.canQuery || model.pending" @click="emit('refresh')">
				{{ model.pending ? 'Working…' : 'Refresh' }}
			</button>
		</header>

		<p
			v-if="model.statusMessage"
			class="telegram-automation__status"
			role="status"
		>
			{{ model.statusMessage }}
		</p>

		<div class="telegram-automation__grid">
			<section class="telegram-automation__card">
				<header class="telegram-automation__section-header">
					<h3>Templates</h3>
					<button type="button" @click="emit('newTemplate')">New</button>
				</header>
				<div class="telegram-automation__list">
					<button
						v-for="template in model.templates"
						:key="template.id"
						type="button"
						@click="emit('selectTemplate', template.id)"
					>
						<strong>{{ template.name }}</strong>
						<small>{{ template.id }} · r{{ template.revision }}</small>
					</button>
				</div>
				<form class="telegram-automation__form" @submit.prevent="emit('saveTemplate')">
					<label>
						Template ID
						<input
							:value="model.template.id"
							autocomplete="off"
							@input="emit('updateTemplateId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Name
						<input
							:value="model.template.name"
							@input="emit('updateTemplateName', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Body
						<textarea
							rows="5"
							:value="model.template.body"
							placeholder="Hello {{name}}"
							@input="emit('updateTemplateBody', ($event.target as HTMLTextAreaElement).value)"
						/>
					</label>
					<label>
						Required variables
						<input
							:value="model.template.requiredVariables"
							placeholder="name, project"
							@input="emit('updateTemplateVariables', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<div class="telegram-automation__actions">
						<small>Expected revision {{ model.template.revision }}</small>
						<button type="submit" :disabled="!model.canCommand || model.pending">Save template</button>
					</div>
				</form>
			</section>

			<section class="telegram-automation__card">
				<header class="telegram-automation__section-header">
					<h3>Policies</h3>
					<button type="button" @click="emit('newPolicy')">New</button>
				</header>
				<div class="telegram-automation__list">
					<button
						v-for="policy in model.policies"
						:key="policy.id"
						type="button"
						@click="emit('selectPolicy', policy.id)"
					>
						<strong>{{ policy.name }}</strong>
						<small>{{ policy.accountId }} · r{{ policy.revision }} · {{ policy.enabled ? 'enabled' : 'disabled' }}</small>
					</button>
				</div>
				<form class="telegram-automation__form" @submit.prevent="emit('savePolicy')">
					<label>
						Policy ID
						<input
							:value="model.policy.id"
							@input="emit('updatePolicyId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Template ID
						<input
							:value="model.policy.templateId"
							@input="emit('updatePolicyTemplateId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Name
						<input
							:value="model.policy.name"
							@input="emit('updatePolicyName', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Account ID
						<input
							:value="model.policy.accountId"
							@input="emit('updatePolicyAccountId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Allowed chat IDs
						<textarea
							rows="3"
							:value="model.policy.providerChatIds"
							placeholder="chat-1, chat-2"
							@input="emit('updatePolicyChatIds', ($event.target as HTMLTextAreaElement).value)"
						/>
					</label>
					<label>
						Expires at (Unix seconds, optional)
						<input
							inputmode="numeric"
							:value="model.policy.expiresAtUnixSeconds"
							@input="emit('updatePolicyExpiresAt', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label class="telegram-automation__checkbox">
						<input
							type="checkbox"
							:checked="model.policy.enabled"
							@change="emit('updatePolicyEnabled', ($event.target as HTMLInputElement).checked)"
						>
						Enabled
					</label>
					<div class="telegram-automation__actions">
						<small>Expected revision {{ model.policy.revision }}</small>
						<button type="submit" :disabled="!model.canCommand || model.pending">Save policy</button>
					</div>
				</form>
			</section>
		</div>

		<section class="telegram-automation__card">
			<header class="telegram-automation__section-header">
				<div>
					<h3>Dry-run preview</h3>
					<small>No TDLib command or Communications event is created.</small>
				</div>
			</header>
			<form class="telegram-automation__form" @submit.prevent="emit('preview')">
				<div class="telegram-automation__grid">
					<label>
						Policy ID
						<input
							:value="model.preview.policyId"
							@input="emit('updatePreviewPolicyId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Account ID
						<input
							:value="model.preview.accountId"
							@input="emit('updatePreviewAccountId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Chat ID
						<input
							:value="model.preview.providerChatId"
							@input="emit('updatePreviewChatId', ($event.target as HTMLInputElement).value)"
						>
					</label>
					<label>
						Variables (one name=value per line)
						<textarea
							rows="3"
							:value="model.preview.variables"
							placeholder="name=Ada"
							@input="emit('updatePreviewVariables', ($event.target as HTMLTextAreaElement).value)"
						/>
					</label>
				</div>
				<div class="telegram-automation__actions">
					<small>Preview is persisted for exact idempotent replay.</small>
					<button type="submit" :disabled="!model.canCommand || model.pending">Render preview</button>
				</div>
			</form>
			<pre v-if="model.preview.renderedText" class="telegram-automation__preview-output">{{ model.preview.renderedText }}</pre>
			<small v-if="model.preview.renderedSha256" class="telegram-automation__digest">
				SHA-256 {{ model.preview.renderedSha256 }}
			</small>
		</section>
	</section>
</template>
