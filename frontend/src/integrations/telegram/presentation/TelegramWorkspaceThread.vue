<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'

defineProps<{ model: TelegramOperationalPageModel }>()

const emit = defineEmits<{
	refreshContext: []
	selectMessage: [messageId: string, providerMessageId: string]
	send: []
	updateDraft: [value: string]
}>()
</script>

<template>
	<main class="telegram-workspace-thread" aria-label="Telegram message thread">
		<template v-if="model.selectedChatId">
			<header class="telegram-thread-header">
				<div class="telegram-thread-header__avatar">
					{{ model.selectedChatTitle.slice(0, 1).toLocaleUpperCase() }}
				</div>
				<div>
					<h2>{{ model.selectedChatTitle }}</h2>
					<p>{{ model.selectedChatId }}</p>
				</div>
				<nav aria-label="Chat actions">
					<button type="button" title="Search"><Icon icon="tabler:search" size="1rem" /></button>
					<button type="button" title="Refresh context" @click="emit('refreshContext')">
						<Icon icon="tabler:refresh" size="1rem" />
					</button>
					<button type="button" title="More"><Icon icon="tabler:dots-vertical" size="1rem" /></button>
				</nav>
			</header>

			<nav class="telegram-thread-tabs" aria-label="Telegram thread sections">
				<button type="button" class="active">Messages <span>{{ model.messages.length }}</span></button>
				<button type="button" disabled>Files</button>
				<button type="button" disabled>Links</button>
				<button type="button" disabled>Voice</button>
				<button type="button" disabled>Topics</button>
				<button type="button" disabled>Pinned</button>
				<button type="button" disabled>Timeline</button>
			</nav>

			<section class="telegram-thread-messages">
				<button
					v-for="message in model.messages"
					:key="message.id"
					type="button"
					class="telegram-thread-message"
					:class="{ outgoing: message.outgoing, selected: message.selected }"
					:aria-pressed="message.selected"
					@click="emit('selectMessage', message.id, message.providerMessageId)"
				>
					<span class="telegram-thread-message__sender">{{ message.sender }}</span>
					<p>{{ message.body }}</p>
					<footer>
						<time>{{ message.meta }}</time>
						<Icon v-if="message.outgoing" icon="tabler:checks" size="0.85rem" />
					</footer>
				</button>
				<p v-if="model.status !== 'loading' && model.messages.length === 0" class="telegram-thread-messages__empty">
					No projected messages in this chat.
				</p>
			</section>

			<form class="telegram-thread-composer" @submit.prevent="emit('send')">
				<button type="button" title="Attach" disabled><Icon icon="tabler:paperclip" size="1.1rem" /></button>
				<textarea
					rows="1"
					placeholder="Write a message…"
					:value="model.draft"
					:disabled="!model.canSend || model.sendPending"
					@input="emit('updateDraft', ($event.target as HTMLTextAreaElement).value)"
				/>
				<button type="button" title="Emoji" disabled><Icon icon="tabler:mood-smile" size="1.1rem" /></button>
				<button
					type="submit"
					class="telegram-thread-composer__send"
					title="Send"
					:disabled="!model.canSend || !model.draft.trim() || model.sendPending"
				>
					<Icon icon="tabler:send" size="1.1rem" />
				</button>
				<small v-if="model.sendMessage">{{ model.sendMessage }}</small>
			</form>
		</template>

		<section v-else class="telegram-workspace-thread__empty">
			<Icon icon="tabler:brand-telegram" size="2.25rem" />
			<h2>Select a Telegram chat</h2>
			<p>Choose a conversation to inspect messages and compose replies.</p>
		</section>
	</main>
</template>
