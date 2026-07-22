<script setup lang="ts">
import '../communicationDomainElements.css'
import MessengerAction from './MessengerAction.vue'
import type { MessengerAttachmentModel, MessengerConversationModel } from './messengerElements'
import MessengerViewer from './MessengerViewer.vue'
import type { MessengerConversationRuntimeAction } from '@/shared/communications/types/messengerRuntimeActions'

withDefaults(defineProps<{
  conversation: MessengerConversationModel
  inspectorVisible?: boolean
  isActionRunning?: boolean
  isLoadingOlder?: boolean
  selectedMessageId?: string
  showInspectorToggle?: boolean
}>(), {
  inspectorVisible: true,
  isActionRunning: false,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'conversation-action': [action: MessengerConversationRuntimeAction]
  'select-message': [messageId: string]
  'toggle-inspector': []
  'submit': [value: string]
  'upload-file': [file: File, caption: string]
  'download-attachment': [attachment: MessengerAttachmentModel]
  'load-older': []
  'messages-visible': []
}>()

function handleSubmit(value: string): void {
  emit('submit', value)
}

function handleUploadFile(file: File, caption: string): void {
  emit('upload-file', file, caption)
}

function handleDownloadAttachment(attachment: MessengerAttachmentModel): void {
  emit('download-attachment', attachment)
}

function handleSelectMessage(messageId: string): void {
  emit('select-message', messageId)
}

function handleConversationAction(action: MessengerConversationRuntimeAction): void {
  emit('conversation-action', action)
}

function handleToggleInspector(): void {
  emit('toggle-inspector')
}

function handleLoadOlder(): void {
  emit('load-older')
}

function handleMessagesVisible(): void {
  emit('messages-visible')
}
</script>

<template>
	<article class="messenger-message">
		<MessengerAction
			:inspector-visible="inspectorVisible"
			:is-action-running="isActionRunning"
			:is-loading-older="isLoadingOlder"
			:selected-message-id="selectedMessageId"
			@select-message="handleSelectMessage"
			:show-inspector-toggle="showInspectorToggle"
			@conversation-action="handleConversationAction"
			@toggle-inspector="handleToggleInspector"
		/>
		<MessengerViewer
			:conversation="conversation"
			:is-action-running="isActionRunning"
			:is-loading-older="isLoadingOlder"
			@submit="handleSubmit"
			@upload-file="handleUploadFile"
			@download-attachment="handleDownloadAttachment"
			@load-older="handleLoadOlder"
			@messages-visible="handleMessagesVisible"
		/>
	</article>
</template>
