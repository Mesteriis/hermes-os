<script setup lang="ts">
import TelegramAccountAccessPanel from './TelegramAccountAccessPanel.vue'
import type { TelegramAccountAccessModel } from './telegramAccountAccessModel'
import TelegramDiscoveryPanel from './TelegramDiscoveryPanel.vue'
import type { TelegramDiscoveryModel } from './telegramDiscoveryModel'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'
import './telegramOperationalPage.css'

defineProps<{
	accountAccess: TelegramAccountAccessModel
	discovery: TelegramDiscoveryModel
	model: TelegramOperationalPageModel
}>()

const emit = defineEmits<{
	load: []
	provisionAccount: []
	refreshAccounts: []
	refreshChatContext: []
	replayAccount: []
	retireAccount: []
	search: []
	selectAccount: [accountId: string]
	selectChat: [providerChatId: string]
	selectMessage: [messageId: string, providerMessageId: string]
	send: []
	startAccount: []
	stopAccount: []
	submitAuthorizationPassword: []
	updateAccountId: [value: string]
	updateAuthorizationPassword: [value: string]
	updateDraft: [value: string]
	updateSearchQuery: [value: string]
	updateProvisionAccountId: [value: string]
	updateProvisionDisplayName: [value: string]
	updateProvisionExternalAccountId: [value: string]
}>()
</script>

<template>
	<section class="telegram-operational-page">
		<header class="telegram-operational-page__header">
			<div>
				<span>Provider operations</span>
				<h1>Telegram</h1>
				<p>Telegram-owned chats, message projections and delivery commands.</p>
			</div>
			<form class="telegram-account-loader" @submit.prevent="emit('load')">
				<label for="telegram-account-id">Admitted account ID</label>
				<div>
					<input
						id="telegram-account-id"
						autocomplete="off"
						placeholder="telegram-account-id"
						:value="model.accountId"
						@input="emit('updateAccountId', ($event.target as HTMLInputElement).value)"
					>
					<button type="submit" :disabled="model.status === 'loading'">
						{{ model.status === 'loading' ? 'Loading…' : 'Open account' }}
					</button>
				</div>
			</form>
		</header>

		<p v-if="model.statusMessage" class="telegram-operational-page__status" :role="model.status === 'error' ? 'alert' : 'status'">
			{{ model.statusMessage }}
		</p>

		<TelegramAccountAccessPanel
			:model="accountAccess"
			@provision="emit('provisionAccount')"
			@refresh="emit('refreshAccounts')"
			@replay="emit('replayAccount')"
			@retire="emit('retireAccount')"
			@select-account="emit('selectAccount', $event)"
			@start="emit('startAccount')"
			@stop="emit('stopAccount')"
			@submit-password="emit('submitAuthorizationPassword')"
			@update-password="emit('updateAuthorizationPassword', $event)"
			@update-provision-account-id="emit('updateProvisionAccountId', $event)"
			@update-provision-display-name="emit('updateProvisionDisplayName', $event)"
			@update-provision-external-account-id="emit('updateProvisionExternalAccountId', $event)"
		/>

		<div class="telegram-operational-workbench" :aria-busy="model.status === 'loading'">
			<aside class="telegram-operational-pane telegram-operational-pane--chats">
				<header><h2>Chats</h2><span>{{ model.chats.length }}</span></header>
				<button
					v-for="chat in model.chats"
					:key="chat.id"
					type="button"
					class="telegram-chat-row"
					:class="{ selected: chat.selected }"
					:aria-pressed="chat.selected"
					@click="emit('selectChat', chat.id)"
				>
					<strong>{{ chat.title }}</strong>
					<small>{{ chat.detail }}</small>
				</button>
			</aside>

			<main class="telegram-operational-pane telegram-operational-pane--messages">
				<header>
					<div><h2>{{ model.selectedChatTitle || 'Messages' }}</h2><small>{{ model.selectedChatId }}</small></div>
					<span>{{ model.messages.length }}</span>
				</header>
				<button
					v-for="message in model.messages"
					:key="message.id"
					type="button"
					class="telegram-message-row"
					:class="{ outgoing: message.outgoing, selected: message.selected }"
					:aria-pressed="message.selected"
					@click="emit('selectMessage', message.id, message.providerMessageId)"
				>
					<div><strong>{{ message.sender }}</strong><small>{{ message.meta }}</small></div>
					<p>{{ message.body }}</p>
				</button>
			</main>
		</div>

		<TelegramDiscoveryPanel
			:model="discovery"
			@refresh-context="emit('refreshChatContext')"
			@search="emit('search')"
			@select-chat="emit('selectChat', $event)"
			@update-query="emit('updateSearchQuery', $event)"
		/>

		<form class="telegram-composer" @submit.prevent="emit('send')">
			<label for="telegram-message-draft">Send to selected Telegram chat</label>
			<textarea
				id="telegram-message-draft"
				rows="3"
				placeholder="Message text"
				:value="model.draft"
				:disabled="!model.selectedChatId || !model.canSend || model.sendPending"
				@input="emit('updateDraft', ($event.target as HTMLTextAreaElement).value)"
			/>
			<footer>
				<small>
					{{ model.sendMessage || (model.canSend
						? 'Accepted means queued; provider completion remains asynchronous.'
						: 'Telegram command capability is not admitted.') }}
				</small>
				<button type="submit" :disabled="!model.selectedChatId || !model.canSend || !model.draft.trim() || model.sendPending">
					{{ model.sendPending ? 'Sending…' : 'Send' }}
				</button>
			</footer>
		</form>
	</section>
</template>
	search: []
