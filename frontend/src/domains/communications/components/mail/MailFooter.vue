<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import { AttachmentChip, Icon } from '@/shared/ui'
import type { CommunicationConversationAttachmentModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailAttachmentTextExtraction from './MailAttachmentTextExtraction.vue'

defineProps<{
  attachments?: readonly CommunicationConversationAttachmentModel[]
}>()

const { t } = useI18n()
</script>

<template>
	<footer
		v-if="attachments?.length"
		class="communication-email-footer"
		:aria-label="t('Attachments')"
	>
		<span class="communication-email-footer__label">
			<Icon icon="tabler:paperclip" size="0.95rem" aria-hidden="true" />
			{{ t('Attachments') }}
		</span>
		<div v-for="attachment in attachments" :key="attachment.id" class="communication-email-footer__attachment">
			<AttachmentChip
				:name="attachment.name"
				:meta="attachment.meta"
				:icon="attachment.icon"
				:tone="attachment.tone"
			/>
			<MailAttachmentTextExtraction :attachment="attachment" />
		</div>
	</footer>
	<footer v-else class="communication-email-footer" aria-hidden="true"></footer>
</template>
