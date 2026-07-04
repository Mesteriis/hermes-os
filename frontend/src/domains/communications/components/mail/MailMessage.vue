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
  showInspectorToggle?: boolean
}>(), {
  inspectorVisible: true,
  showInspectorToggle: true
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
		<MailAction
			:action-groups="message.actionGroups"
			:inspector-visible="inspectorVisible"
			:show-inspector-toggle="showInspectorToggle"
			@toggle-inspector="handleToggleInspector"
		/>
		<section class="communication-email-preview communication-email-preview--structured" :aria-label="t('Open message')">
			<MailViewer :message="message" :fallback-subject="fallbackSubject" />
			<MailFooter :attachments="message.attachments" />
		</section>
	</article>
</template>
