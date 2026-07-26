<script setup lang="ts">
import type { TelegramAccountAccessModel } from './telegramAccountAccessModel'
import './telegramAccountAccessPanel.css'

defineProps<{ model: TelegramAccountAccessModel }>()

const emit = defineEmits<{
	provision: []
	refresh: []
	replay: []
	retire: []
	selectAccount: [accountId: string]
	start: []
	stop: []
	submitPassword: []
	updatePassword: [value: string]
	updateProvisionAccountId: [value: string]
	updateProvisionDisplayName: [value: string]
	updateProvisionExternalAccountId: [value: string]
}>()
</script>

<template>
	<section class="telegram-account-access">
		<header>
			<div>
				<span>Account access</span>
				<h2>Runtime & authorization</h2>
			</div>
			<button type="button" :disabled="model.pending" @click="emit('refresh')">
				{{ model.pending ? 'Refreshing…' : 'Refresh' }}
			</button>
		</header>

		<p class="telegram-account-access__status" role="status">
			{{ model.statusMessage || `Authorization: ${model.authorizationState}` }}
		</p>

		<div class="telegram-account-access__grid">
			<div class="telegram-account-access__accounts">
				<button
					v-for="account in model.accounts"
					:key="account.id"
					type="button"
					:class="{ selected: account.selected }"
					:aria-pressed="account.selected"
					@click="emit('selectAccount', account.id)"
				>
					<strong>{{ account.title }}</strong>
					<small>{{ account.detail }}</small>
				</button>
			</div>

			<div class="telegram-account-access__actions">
				<div class="telegram-account-access__action-row">
					<button type="button" :disabled="!model.selectedAccountId || !model.canManageLifecycle || model.pending" @click="emit('start')">Start</button>
					<button type="button" :disabled="!model.selectedAccountId || !model.canManageLifecycle || model.pending" @click="emit('stop')">Stop</button>
					<button type="button" :disabled="!model.selectedAccountId || !model.canManageLifecycle || model.pending" @click="emit('replay')">Replay</button>
					<button class="danger" type="button" :disabled="!model.selectedAccountId || !model.canManageLifecycle || model.pending" @click="emit('retire')">Retire</button>
				</div>

				<a
					v-if="model.authorizationQrLink"
					:href="model.authorizationQrLink"
					rel="noreferrer"
					target="_blank"
				>
					Open Telegram authorization QR
				</a>

				<form v-if="model.canAuthorize" @submit.prevent="emit('submitPassword')">
					<label for="telegram-authorization-password">
						2FA password
						<small v-if="model.authorizationPasswordHint">{{ model.authorizationPasswordHint }}</small>
					</label>
					<div>
						<input
							id="telegram-authorization-password"
							type="password"
							autocomplete="current-password"
							:value="model.password"
							@input="emit('updatePassword', ($event.target as HTMLInputElement).value)"
						>
						<button type="submit" :disabled="!model.password.trim() || model.pending">Submit</button>
					</div>
				</form>
			</div>
		</div>

		<details>
			<summary>Provision Telegram account</summary>
			<form class="telegram-account-access__provision" @submit.prevent="emit('provision')">
				<label for="telegram-provision-account-id">Account ID</label>
				<input
					id="telegram-provision-account-id"
					autocomplete="off"
					:value="model.provisionAccountId"
					@input="emit('updateProvisionAccountId', ($event.target as HTMLInputElement).value)"
				>
				<label for="telegram-provision-display-name">Display name</label>
				<input
					id="telegram-provision-display-name"
					autocomplete="off"
					:value="model.provisionDisplayName"
					@input="emit('updateProvisionDisplayName', ($event.target as HTMLInputElement).value)"
				>
				<label for="telegram-provision-external-id">External account ID</label>
				<input
					id="telegram-provision-external-id"
					autocomplete="off"
					:value="model.provisionExternalAccountId"
					@input="emit('updateProvisionExternalAccountId', ($event.target as HTMLInputElement).value)"
				>
				<button
					type="submit"
					:disabled="!model.canManageLifecycle || !model.provisionAccountId.trim() || !model.provisionDisplayName.trim() || model.pending"
				>
					Provision
				</button>
			</form>
		</details>
	</section>
</template>
