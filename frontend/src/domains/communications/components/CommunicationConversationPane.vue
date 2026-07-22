<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '@/platform/i18n'
import { AttachmentChip, Badge, ChatInput, MessageBubble, MessageStatus, ProviderIcon } from '@/shared/ui'
import MailMessage from './mail/MailMessage.vue'
import type { CommunicationConversationModel } from './communicationDomainElements'
import {
  communicationChannelLabel,
  communicationChannelProviderIcon,
  communicationConversationIsEmail,
  communicationWorkflowStatusPresentation
} from './communicationDomainElements'
import './communicationDomainElements.css'
import { communicationConversationActiveMessage } from './communicationConversationPresentation'

const props = defineProps<{
  conversation: CommunicationConversationModel
}>()

const { t } = useI18n()
const activeMailMessage = computed(() => communicationConversationActiveMessage(props.conversation.messages))
</script>

<template>
	<template v-if="communicationConversationIsEmail(conversation.channelKind)">
		<MailMessage
			v-if="activeMailMessage"
			:message="activeMailMessage"
			:fallback-subject="conversation.title"
			:show-inspector-toggle="false"
		/>
		<section v-else class="communication-workspace-panel communication-mail-workspace-reader" :aria-label="t('Open message')">
			<p class="communication-mail-workspace-reader__empty">{{ t('No message selected.') }}</p>
		</section>
	</template>
	<section v-else class="communication-workspace-panel communication-conversation" aria-label="Conversation">
		<header class="communication-workspace-panel__header">
			<div class="communication-conversation__toolbar">
				<div class="communication-inbox-item__identity">
					<ProviderIcon
						:provider="communicationChannelProviderIcon(conversation.channelKind)"
						:label="communicationChannelLabel(conversation.channelKind)"
					/>
					<div class="communication-domain-card__title-group">
						<h2 class="communication-conversation__title">{{ conversation.title }}</h2>
						<p class="communication-conversation__subtitle">{{ conversation.subtitle }}</p>
					</div>
				</div>
				<Badge :variant="communicationWorkflowStatusPresentation(conversation.workflowState).badgeTone">
					{{ communicationWorkflowStatusPresentation(conversation.workflowState).label }}
				</Badge>
			</div>
			<div class="communication-conversation__facts">
				<span v-for="fact in conversation.facts" :key="fact.label" class="communication-conversation__fact">
					{{ fact.label }}: {{ fact.value }}
				</span>
			</div>
		</header>

		<div class="communication-conversation__messages">
			<MessageBubble
				v-for="message in conversation.messages"
				:key="message.id"
				:author="message.author"
				:direction="message.direction"
				:meta="message.meta"
				:timestamp="message.timestamp"
				:tone="message.tone"
			>
				<p>{{ message.body }}</p>
				<div v-if="message.attachments?.length" class="communication-conversation__message-attachments">
					<AttachmentChip
						v-for="attachment in message.attachments"
						:key="attachment.id"
						:name="attachment.name"
						:meta="attachment.meta"
						:icon="attachment.icon"
						:tone="attachment.tone"
					/>
				</div>
				<template #footer>
					<MessageStatus v-if="message.direction === 'outbound'" status="delivered" />
				</template>
			</MessageBubble>
		</div>

		<div class="communication-conversation__composer">
			<ChatInput
				id="communication-workspace-reply"
				:model-value="conversation.draftPreview"
				label="Reply"
				placeholder="Write a reply"
				helper="Provider write remains gated by review and account capability."
				send-label="Send"
				attach-label="Attach evidence"
				:max-length="320"
			/>
		</div>
	</section>
</template>
