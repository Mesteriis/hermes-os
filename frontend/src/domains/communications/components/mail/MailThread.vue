<script setup lang="ts">
import { Badge, ProviderIcon } from '@/shared/ui'
import type { CommunicationConversationModel } from '../communicationDomainElements'
import {
  communicationChannelLabel,
  communicationChannelProviderIcon,
  communicationWorkflowStatusPresentation
} from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailMessage from './MailMessage.vue'
import MailReplyComposer from './MailReplyComposer.vue'

withDefaults(defineProps<{
  conversation: CommunicationConversationModel
  inspectorVisible?: boolean
}>(), {
  inspectorVisible: true
})

const emit = defineEmits<{
  'toggle-inspector': []
}>()

function handleToggleInspector(): void {
  emit('toggle-inspector')
}
</script>

<template>
	<section class="communication-workspace-panel communication-conversation" aria-label="Mail thread">
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

		<div class="communication-email-thread">
			<MailMessage
				v-for="message in conversation.messages"
				:key="message.id"
				:message="message"
				:fallback-subject="conversation.title"
				:inspector-visible="inspectorVisible"
				@toggle-inspector="handleToggleInspector"
			/>
		</div>

		<div class="communication-conversation__composer">
			<MailReplyComposer
				:draft-preview="conversation.draftPreview"
				:reply-original="conversation.replyOriginal"
			/>
		</div>
	</section>
</template>
