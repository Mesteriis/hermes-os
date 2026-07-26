<script setup lang="ts">
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'
import './telegramOperationalPage.css'

defineProps<{ model: TelegramOperationalPageModel }>()

const emit = defineEmits<{
	load: []
	selectChat: [providerChatId: string]
	send: []
	updateAccountId: [value: string]
	updateDraft: [value: string]
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
				<article
					v-for="message in model.messages"
					:key="message.id"
					class="telegram-message-row"
					:class="{ outgoing: message.outgoing }"
				>
					<div><strong>{{ message.sender }}</strong><small>{{ message.meta }}</small></div>
					<p>{{ message.body }}</p>
				</article>
			</main>
		</div>

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
