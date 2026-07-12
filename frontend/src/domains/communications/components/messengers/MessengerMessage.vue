<script setup lang="ts">
import '../communicationDomainElements.css'
import MessengerAction from './MessengerAction.vue'
import type { MessengerAttachmentModel, MessengerConversationModel } from './messengerElements'
import MessengerViewer from './MessengerViewer.vue'
import type { TelegramConversationRuntimeAction } from '@/shared/communications/types/telegramRuntimeActions'
import type { TelegramMessage } from '@/shared/communications/types/telegram'

withDefaults(defineProps<{
  conversation: MessengerConversationModel
  inspectorVisible?: boolean
  isActionRunning?: boolean
  isLoadingOlder?: boolean
  selectedMessageId?: string
  showInspectorToggle?: boolean
  telegramMessage?: TelegramMessage | null
}>(), {
  inspectorVisible: true,
  isActionRunning: false,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'conversation-action': [action: TelegramConversationRuntimeAction]
  'select-message': [messageId: string]
  'toggle-inspector': []
  'submit': [value: string]
  'upload-file': [file: File, caption: string]
  'download-attachment': [attachment: MessengerAttachmentModel]
  'load-older': []
  'messages-visible': []
}>()
</script>

<template>
	<article class="messenger-message">
		<MessengerAction
			:channel-kind="conversation.channelKind"
			:inspector-visible="inspectorVisible"
			:is-action-running="isActionRunning"
			:is-loading-older="isLoadingOlder"
			:selected-message-id="selectedMessageId"
			:telegram-message="telegramMessage"
			@select-message="emit('select-message', $event)"
			:show-inspector-toggle="showInspectorToggle"
			@conversation-action="emit('conversation-action', $event)"
			@toggle-inspector="emit('toggle-inspector')"
		/>
		<MessengerViewer
			:conversation="conversation"
			:is-action-running="isActionRunning"
			:is-loading-older="isLoadingOlder"
			@submit="emit('submit', $event)"
			@upload-file="(file, caption) => emit('upload-file', file, caption)"
			@download-attachment="emit('download-attachment', $event)"
			@load-older="emit('load-older')"
			@messages-visible="emit('messages-visible')"
		/>
	</article>
</template>
