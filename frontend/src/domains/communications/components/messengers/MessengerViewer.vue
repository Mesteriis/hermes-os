<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import { AttachmentChip, Badge, Button, MessageBubble, MessageStatus, ProviderIcon, ReactionBadge } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { MessengerAttachmentModel, MessengerConversationModel } from './messengerElements'
import {
  messengerChannelLabel,
  messengerChannelProviderIcon,
  messengerConversationKindLabel,
  messengerMessageAuthor,
  messengerMessageMeta,
  messengerMessageTimestamp,
  messengerWorkflowStatusPresentation,
} from './messengerElements'
import MessengerRichEditor from './MessengerRichEditor.vue'
import { useMessengerViewerController, type MessengerViewerControllerActions } from '../../queries/useMessengerViewerController'

const props = defineProps<{
  conversation: MessengerConversationModel
  isActionRunning?: boolean
  isLoadingOlder?: boolean
  selectedMessageId?: string
}>()

const emit = defineEmits<{
  'select-message': [messageId: string]
  'submit': [value: string]
  'download-attachment': [attachment: MessengerAttachmentModel]
  'load-older': []
  'messages-visible': []
}>()

const { t } = useI18n()

const controller = useMessengerViewerController(
  props,
  {
    submitMessage: (value) => emit('submit', value),
    selectMessage: (messageId) => emit('select-message', messageId),
    downloadAttachment: (attachment) => emit('download-attachment', attachment),
    loadOlder: () => emit('load-older'),
    messagesVisible: () => emit('messages-visible'),
  } satisfies MessengerViewerControllerActions,
)

const {
  messagesContainer,
  isConversationEmpty,
  handleComposerSubmit,
  handleMessageScroll,
  handleSelectMessage,
  handleDownloadAttachment,
} = controller
</script>

<template>
	<section class="messenger-viewer" :aria-label="t('Open dialog')">
		<header class="messenger-viewer__header">
			<div class="messenger-viewer__identity">
				<ProviderIcon
					:provider="messengerChannelProviderIcon(conversation.channelKind)"
					:label="messengerChannelLabel(conversation.channelKind)"
					class="messenger-viewer__provider"
				/>
				<div class="messenger-viewer__title-group">
					<h2 class="messenger-viewer__title">{{ conversation.title }}</h2>
					<p class="messenger-viewer__subtitle">
						{{ messengerChannelLabel(conversation.channelKind) }} · {{ t(messengerConversationKindLabel(conversation.kind)) }}
					</p>
				</div>
			</div>
			<Badge :variant="messengerWorkflowStatusPresentation(conversation.workflowState).tone">
				{{ t(messengerWorkflowStatusPresentation(conversation.workflowState).label) }}
			</Badge>
		</header>

		<div class="messenger-viewer__facts" :aria-label="t('Dialog facts')">
			<span class="messenger-viewer__fact">{{ conversation.participantsLabel }}</span>
			<span v-if="conversation.lastSeenLabel" class="messenger-viewer__fact">{{ conversation.lastSeenLabel }}</span>
			<span
				v-for="fact in conversation.facts"
				:key="fact.label"
				:class="['messenger-viewer__fact', fact.tone && `messenger-viewer__fact--${fact.tone}`]"
			>
				{{ t(fact.label) }}: {{ fact.value }}
			</span>
		</div>

		<div v-if="isConversationEmpty" class="messenger-viewer__empty-state">
			<ProviderIcon
				:provider="messengerChannelProviderIcon(conversation.channelKind)"
				:label="messengerChannelLabel(conversation.channelKind)"
				class="messenger-viewer__empty-icon"
			/>
			<h3>{{ messengerChannelLabel(conversation.channelKind) }} is not connected</h3>
			<p>Open Settings → Accounts to configure a local account or resume QR login.</p>
		</div>
		<div ref="messagesContainer" v-else class="messenger-viewer__messages" @scroll.passive="handleMessageScroll">
			<p v-if="isLoadingOlder" class="messenger-viewer__history-status" role="status">Loading older messages…</p>
			<MessageBubble
				v-for="message in conversation.messages"
				:key="message.id"
				:author="messengerMessageAuthor(message)"
				:direction="message.direction"
				:meta="messengerMessageMeta(message)"
				:timestamp="messengerMessageTimestamp(message)"
				:tone="message.tone"
				:selected="message.id === selectedMessageId"
				:pending="message.pending"
					@click="handleSelectMessage(message.id)"
			>
				<p>{{ message.body }}</p>
				<div v-if="message.reactions?.length" class="messenger-viewer__reactions" :aria-label="t('Reactions')">
					<ReactionBadge
						v-for="reaction in message.reactions"
						:key="reaction.emoji"
						:emoji="reaction.emoji"
						:count="reaction.count"
						:active="reaction.active"
						:label="`${reaction.emoji} ${reaction.count}`"
					/>
				</div>
				<div v-if="message.attachments?.length" class="messenger-viewer__attachments">
					<div
						v-for="attachment in message.attachments"
						:key="attachment.id"
						class="messenger-viewer__attachment"
					>
						<AttachmentChip
						:name="attachment.name"
						:meta="attachment.meta"
						:icon="attachment.icon"
						:tone="attachment.tone"
					/>
						<Button
							v-if="attachment.downloadable"
							variant="ghost"
							size="sm"
							icon="tabler:download"
							:aria-label="`${t('Download attachment')}: ${attachment.name}`"
							:title="t('Download attachment')"
							@click.stop="handleDownloadAttachment(attachment)"
						/>
					</div>
				</div>
				<template #footer>
					<MessageStatus
						v-if="message.deliveryStatus"
						:status="message.deliveryStatus"
						:label="message.deliveryStatusLabel"
					/>
				</template>
			</MessageBubble>
		</div>
		<footer v-if="!isConversationEmpty" class="messenger-viewer__composer">
			<MessengerRichEditor
				:conversation="conversation"
				@submit="handleComposerSubmit"
			/>
		</footer>
	</section>
</template>
