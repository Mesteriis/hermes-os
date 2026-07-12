<script setup lang="ts">
import { ref } from 'vue'
import '../communicationDomainElements.css'
import MessengerInspector from './MessengerInspector.vue'
import MessengerList from './MessengerList.vue'
import MessengerMessage from './MessengerMessage.vue'
import type { MessengerAttachmentModel, MessengerConversationModel, MessengerInspectorModel, MessengerListItemModel } from './messengerElements'
import type { TelegramConversationRuntimeAction } from '@/shared/communications/types/telegramRuntimeActions'
import type { TelegramMessage } from '@/shared/communications/types/telegram'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import type { TelegramConversationRuntimeActionRunner } from '@/shared/communications/types/telegramRuntimeActions'

defineProps<{
  isListLoading?: boolean
  isListRefreshing?: boolean
  listError?: string
  items: readonly MessengerListItemModel[]
  conversation: MessengerConversationModel
  inspector: MessengerInspectorModel
  isActionRunning?: boolean
  isLoadingOlder?: boolean
  selectedMessageId?: string
  runtimeActionRunner?: TelegramConversationRuntimeActionRunner
  telegramChat?: TelegramChat | null
  telegramMessage?: TelegramMessage | null
}>()

const emit = defineEmits<{
  'conversation-action': [action: TelegramConversationRuntimeAction]
  refresh: []
  'select-message': [messageId: string]
  'select-conversation': [item: MessengerListItemModel]
  'submit': [value: string]
  'upload-file': [file: File, caption: string]
  'download-attachment': [attachment: MessengerAttachmentModel]
  'load-older': []
  'messages-visible': []
}>()

const isInspectorVisible = ref(true)

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
}
</script>

<template>
	<section
		:class="[
			'communication-workspace-shell communication-workspace-shell--messenger',
			!isInspectorVisible && 'communication-workspace-shell--messenger-inspector-hidden'
		]"
	>
		<MessengerList
			:items="items"
			:is-loading="isListLoading"
			:is-refreshing="isListRefreshing"
			:error-message="listError"
			:selected-id="conversation.id"
			@refresh="emit('refresh')"
			@select="emit('select-conversation', $event)"
		/>
		<section class="communication-messenger-workspace-reader" aria-label="Open dialog">
		<MessengerMessage
				:conversation="conversation"
				:inspector-visible="isInspectorVisible"
				:is-action-running="isActionRunning"
				:is-loading-older="isLoadingOlder"
				:selected-message-id="selectedMessageId"
				:telegram-message="telegramMessage"
				@conversation-action="emit('conversation-action', $event)"
				@select-message="emit('select-message', $event)"
				@submit="emit('submit', $event)"
			@toggle-inspector="handleToggleInspector"
			@upload-file="(file, caption) => emit('upload-file', file, caption)"
			@download-attachment="emit('download-attachment', $event)"
			@load-older="emit('load-older')"
			@messages-visible="emit('messages-visible')"
		/>
		</section>
		<MessengerInspector
			v-if="isInspectorVisible"
			:model="inspector"
			:runtime-action-runner="runtimeActionRunner"
			:telegram-chat="telegramChat"
			:telegram-message="telegramMessage"
		/>
	</section>
</template>
