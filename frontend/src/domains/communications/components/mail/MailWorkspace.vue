<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '@/platform/i18n'
import type { CommunicationConversationModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailInspector from './MailInspector.vue'
import MailList from './MailList.vue'
import MailMessage from './MailMessage.vue'
import type { MailListItemModel } from './mailElements'
import type { MailInspectorModel } from './mailInspector'

const props = defineProps<{
  items: readonly MailListItemModel[]
  conversation: CommunicationConversationModel
  inspector: MailInspectorModel
}>()

const { t } = useI18n()
const isInspectorVisible = ref(true)
const activeMessage = computed(() => props.conversation.messages[props.conversation.messages.length - 1])

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
}
</script>

<template>
	<section
		:class="[
			'communication-workspace-shell communication-workspace-shell--mail',
			!isInspectorVisible && 'communication-workspace-shell--mail-inspector-hidden'
		]"
	>
		<MailList :items="items" />
		<section class="communication-mail-workspace-reader" :aria-label="t('Open message')">
			<MailMessage
				v-if="activeMessage"
				:message="activeMessage"
				:fallback-subject="conversation.title"
				:inspector-visible="isInspectorVisible"
				@toggle-inspector="handleToggleInspector"
			/>
			<p v-else class="communication-mail-workspace-reader__empty">{{ t('No message selected.') }}</p>
		</section>
		<MailInspector v-if="isInspectorVisible" :model="inspector" />
	</section>
</template>
