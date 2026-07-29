<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'

defineProps<{ model: TelegramOperationalPageModel }>()

const emit = defineEmits<{
	selectChat: [providerChatId: string]
}>()
</script>

<template>
	<aside class="telegram-workspace-chat-list" aria-label="Telegram chats">
		<header>
			<div>
				<strong>Chats</strong>
				<small>{{ model.chats.length }} conversations</small>
			</div>
			<button type="button" title="Chat list options"><Icon icon="tabler:dots" size="1rem" /></button>
		</header>

		<p v-if="model.statusMessage" class="telegram-workspace-chat-list__status" :role="model.status === 'error' ? 'alert' : 'status'">
			{{ model.statusMessage }}
		</p>

		<div class="telegram-workspace-chat-list__items">
			<button
				v-for="chat in model.chats"
				:key="chat.id"
				type="button"
				class="telegram-workspace-chat"
				:class="{ selected: chat.selected }"
				:aria-pressed="chat.selected"
				@click="emit('selectChat', chat.id)"
			>
				<span class="telegram-workspace-chat__avatar">
					{{ chat.title.slice(0, 1).toLocaleUpperCase() }}
				</span>
				<span class="telegram-workspace-chat__body">
					<strong>{{ chat.title }}</strong>
					<small>{{ chat.detail }}</small>
				</span>
				<span v-if="chat.selected" class="telegram-workspace-chat__selected" />
			</button>

			<section v-if="model.status !== 'loading' && model.chats.length === 0" class="telegram-workspace-chat-list__empty">
				<Icon icon="tabler:messages" size="1.75rem" />
				<strong>No Telegram chats</strong>
				<small>Sync the selected account to load provider conversations.</small>
			</section>
		</div>
	</aside>
</template>
