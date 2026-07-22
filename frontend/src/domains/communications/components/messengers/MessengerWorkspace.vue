<script setup lang="ts">
import '../communicationDomainElements.css'
import MessengerInspector from './MessengerInspector.vue'
import MessengerList from './MessengerList.vue'
import MessengerMessage from './MessengerMessage.vue'
import { useMessengerWorkspaceController } from '../../queries/useMessengerWorkspaceController'
import type { MessengerAttachmentModel, MessengerConversationModel, MessengerInspectorModel, MessengerListItemModel } from './messengerElements'
import type { MessengerConversationRuntimeAction, MessengerConversationRuntimeActionRunner } from '@/shared/communications/types/messengerRuntimeActions'

const props = defineProps<{
  isListLoading?: boolean
  isListRefreshing?: boolean
  listError?: string
  items: readonly MessengerListItemModel[]
  conversation: MessengerConversationModel
  inspector: MessengerInspectorModel
  isActionRunning?: boolean
  isLoadingOlder?: boolean
  selectedMessageId?: string
  runtimeActionRunner?: MessengerConversationRuntimeActionRunner
}>()

const emit = defineEmits<{
  'conversation-action': [action: MessengerConversationRuntimeAction]
  refresh: []
  'select-message': [messageId: string]
  'select-conversation': [item: MessengerListItemModel]
  'submit': [value: string]
  'upload-file': [file: File, caption: string]
  'download-attachment': [attachment: MessengerAttachmentModel]
  'load-older': []
  'messages-visible': []
}>()

const controller = useMessengerWorkspaceController(
  props,
  {
    refresh: () => emit('refresh'),
    selectConversation: (item) => emit('select-conversation', item),
    conversationAction: (action) => emit('conversation-action', action),
    selectMessage: (messageId) => emit('select-message', messageId),
    submit: (value) => emit('submit', value),
    uploadFile: (file, caption) => emit('upload-file', file, caption),
    downloadAttachment: (attachment) => emit('download-attachment', attachment),
    loadOlder: () => emit('load-older'),
    messagesVisible: () => emit('messages-visible'),
  },
)

const {
  isInspectorVisible,
  handleConversationAction,
  handleDownloadAttachment,
  handleLoadOlder,
  handleMessagesVisible,
  handleRefresh,
  handleSelectConversation,
  handleSelectMessage,
  handleSubmit,
  handleToggleInspector,
  handleUploadFile,
} = controller
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
				@refresh="handleRefresh"
				@select="handleSelectConversation"
			/>
			<section class="communication-messenger-workspace-reader" aria-label="Open dialog">
		<MessengerMessage
				:conversation="conversation"
				:inspector-visible="isInspectorVisible"
				:is-action-running="isActionRunning"
				:is-loading-older="isLoadingOlder"
				:selected-message-id="selectedMessageId"
				@conversation-action="handleConversationAction"
				@select-message="handleSelectMessage"
				@submit="handleSubmit"
				@toggle-inspector="handleToggleInspector"
				@upload-file="handleUploadFile"
				@download-attachment="handleDownloadAttachment"
				@load-older="handleLoadOlder"
				@messages-visible="handleMessagesVisible"
			/>
			</section>
		<MessengerInspector
			v-if="isInspectorVisible"
			:model="inspector"
			:runtime-action-runner="runtimeActionRunner"
		/>
	</section>
</template>
