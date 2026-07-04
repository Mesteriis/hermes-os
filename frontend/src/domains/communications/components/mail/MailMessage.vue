<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import type { CommunicationConversationMessageModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailAction from './MailAction.vue'
import MailFooter from './MailFooter.vue'
import MailViewer from './MailViewer.vue'

withDefaults(defineProps<{
  message: CommunicationConversationMessageModel
  fallbackSubject: string
  inspectorVisible?: boolean
}>(), {
  inspectorVisible: true
})

const emit = defineEmits<{
  'toggle-inspector': []
}>()

const { t } = useI18n()

function handleToggleInspector(): void {
  emit('toggle-inspector')
}
</script>

<template>
	<article
		:class="[
			'communication-email-message',
			message.direction === 'outbound' && 'communication-email-message--outbound',
			message.tone === 'warning' && 'communication-email-message--signal'
		]"
	>
		<section class="communication-email-center communication-email-center--structured" :aria-label="t('Open message')">
			<MailAction
				:action-groups="message.actionGroups"
				:inspector-visible="inspectorVisible"
				@toggle-inspector="handleToggleInspector"
			/>
			<MailViewer :message="message" :fallback-subject="fallbackSubject" />
			<MailFooter :attachments="message.attachments" />
		</section>
	</article>
</template>
