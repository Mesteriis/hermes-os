<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { AttachmentChip, Badge, Button, MessageBubble, MessageStatus, ProviderIcon, ReactionBadge } from '@/shared/ui'
import { messengerComposerPlainText, type MessengerComposerCapability } from './messengerComposer'
import type { TelegramMessage } from '@/shared/communications/types/telegram'
import '../communicationDomainElements.css'
import type { MessengerAttachmentModel, MessengerConversationModel } from './messengerElements'
import {
  messengerChannelLabel,
  messengerChannelProviderIcon,
  messengerConversationKindLabel,
  messengerMessageAuthor,
  messengerMessageMeta,
  messengerMessageTimestamp,
  messengerWorkflowStatusPresentation
} from './messengerElements'
import MessengerRichEditor from './MessengerRichEditor.vue'

const props = defineProps<{
  conversation: MessengerConversationModel
  isActionRunning?: boolean
  isLoadingOlder?: boolean
  selectedMessageId?: string
  telegramMessage?: TelegramMessage | null
}>()

const emit = defineEmits<{
  'select-message': [messageId: string]
  'submit': [value: string]
  'upload-file': [file: File, caption: string]
  'download-attachment': [attachment: MessengerAttachmentModel]
  'load-older': []
  'messages-visible': []
}>()

const { t } = useI18n()
const fileInput = ref<HTMLInputElement | null>(null)
const pendingAttachment = ref<File | null>(null)
const messagesContainer = ref<HTMLElement | null>(null)
const historyScrollHeight = ref<number | null>(null)
const isTelegramEmptyState = computed(() =>
  props.conversation.channelKind === 'telegram' && props.conversation.id === 'telegram:empty'
)

watch(
  () => props.conversation.id,
  () => {
    pendingAttachment.value = null
    historyScrollHeight.value = null
    void nextTick(() => {
      const container = messagesContainer.value
      if (container) container.scrollTop = container.scrollHeight
    })
  }
)

watch(
  [() => props.conversation.id, () => props.conversation.messages.length],
  () => {
    void nextTick(() => {
      const container = messagesContainer.value
      if (container && props.conversation.messages.length > 0 && container.clientHeight > 0) {
        emit('messages-visible')
      }
    })
  },
  { immediate: true, flush: 'post' }
)

watch(
  () => props.isLoadingOlder,
  (isLoading, wasLoading) => {
    if (!wasLoading || isLoading || historyScrollHeight.value == null) return
    void nextTick(() => {
      const container = messagesContainer.value
      const previousHeight = historyScrollHeight.value
      historyScrollHeight.value = null
      if (container && previousHeight != null) {
        container.scrollTop += container.scrollHeight - previousHeight
      }
    })
  }
)

function handleComposerCapability(capability: MessengerComposerCapability): void {
  if (capability.id === 'telegram-file' && !props.isActionRunning) {
    fileInput.value?.click()
  }
}

function handleFileChange(event: Event): void {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (file) pendingAttachment.value = file
}

function handleComposerSubmit(value: string): void {
  if (props.isActionRunning) return
  const file = pendingAttachment.value
  if (!file) {
    emit('submit', value)
    return
  }

  pendingAttachment.value = null
  emit('upload-file', file, messengerComposerPlainText(value))
}

function handleMessageScroll(event: Event): void {
  const target = event.currentTarget as HTMLElement
  if (target.scrollTop > 80 || props.isLoadingOlder || historyScrollHeight.value != null) return
  historyScrollHeight.value = target.scrollHeight
  emit('load-older')
}
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

		<div v-if="isTelegramEmptyState" class="messenger-viewer__empty-state">
			<ProviderIcon provider="telegram" label="Telegram" class="messenger-viewer__empty-icon" />
			<h3>Telegram is not connected</h3>
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
				@click="emit('select-message', message.id)"
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
							@click.stop="emit('download-attachment', attachment)"
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
		<footer v-if="!isTelegramEmptyState" class="messenger-viewer__composer">
			<div v-if="pendingAttachment" class="messenger-viewer__pending-attachment" role="status">
				<AttachmentChip
					:name="pendingAttachment.name"
					:meta="t('Attachment ready to send')"
					icon="tabler:paperclip"
				/>
				<Button
					variant="ghost"
					size="sm"
					icon="tabler:x"
					:aria-label="t('Remove attachment')"
					:title="t('Remove attachment')"
					@click="pendingAttachment = null"
				/>
			</div>
			<MessengerRichEditor
				:conversation="conversation"
				@select-capability="handleComposerCapability"
				@submit="handleComposerSubmit"
			/>
			<input
				ref="fileInput"
				class="messenger-viewer__file-input"
				type="file"
				@change="handleFileChange"
			/>
		</footer>
	</section>
</template>
